use anyhow::{Context, Result};
use rpassword::read_password;
use serde_json::Value;
use std::fs;
use std::io::{self, Write};
use std::process::Command;

use crate::config::{claude_settings_path, current_profile_path, profile_path, profiles_dir};

/// Prompt user for input
fn prompt_input(prompt: &str) -> Result<String> {
    print!("{}", prompt);
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    Ok(input.trim().to_string())
}

/// Prompt user for password input (hidden input)
fn prompt_password(prompt: &str) -> Result<String> {
    print!("{}", prompt);
    io::stdout().flush()?;
    let password = read_password()?;
    Ok(password.trim().to_string())
}

/// Add a profile interactively
pub fn add_profile_interactive(name: &str, env_vars: &[String]) -> Result<()> {
    println!(
        "Adding profile '{}' - please answer the following questions:",
        name
    );

    let base_url = prompt_input("ANTHROPIC_BASE_URL: ")?.trim().to_string();
    let auth_token = prompt_password("ANTHROPIC_AUTH_TOKEN: ")?;
    let model = prompt_input("ANTHROPIC_MODEL (optional, press Enter to skip): ")?
        .trim()
        .to_string();
    let small_fast_model =
        prompt_input("ANTHROPIC_SMALL_FAST_MODEL (optional, press Enter to skip): ")?
            .trim()
            .to_string();
    let timeout = prompt_input("API_TIMEOUT_MS (optional, press Enter to skip): ")?
        .trim()
        .to_string();
    let disable_nonessential = prompt_input(
        "CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC (optional int, e.g., 1; press Enter to skip): ",
    )?
    .trim()
    .to_string();

    let mut obj = serde_json::Map::new();
    let mut env_obj = serde_json::Map::new();

    // Add required env variables
    if !base_url.is_empty() {
        env_obj.insert("ANTHROPIC_BASE_URL".to_string(), Value::String(base_url));
    }
    if !auth_token.is_empty() {
        env_obj.insert(
            "ANTHROPIC_AUTH_TOKEN".to_string(),
            Value::String(auth_token),
        );
    }

    // Add optional env variables
    if !model.is_empty() {
        env_obj.insert("ANTHROPIC_MODEL".to_string(), Value::String(model));
    }
    if !timeout.is_empty() {
        env_obj.insert("API_TIMEOUT_MS".to_string(), Value::String(timeout));
    }
    if !small_fast_model.is_empty() {
        env_obj.insert(
            "ANTHROPIC_SMALL_FAST_MODEL".to_string(),
            Value::String(small_fast_model),
        );
    }
    if !disable_nonessential.is_empty() {
        match disable_nonessential.parse::<i64>() {
            Ok(n) => {
                env_obj.insert(
                    "CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC".to_string(),
                    Value::Number(n.into()),
                );
            }
            Err(_) => {
                println!(
                    "Warning: invalid integer for CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC; storing as string."
                );
                env_obj.insert(
                    "CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC".to_string(),
                    Value::String(disable_nonessential),
                );
            }
        }
    }

    // Parse and add additional env variables from --env flags
    for env_pair in env_vars {
        if let Some((key, value)) = env_pair.split_once('=') {
            env_obj.insert(
                key.trim().to_string(),
                Value::String(value.trim().to_string()),
            );
        } else {
            println!(
                "Warning: ignoring invalid env format '{}' (expected KEY=VALUE)",
                env_pair
            );
        }
    }

    if !env_obj.is_empty() {
        obj.insert("env".to_string(), Value::Object(env_obj));
    }

    let v = Value::Object(obj);
    let p = profile_path(name)?;
    fs::write(&p, serde_json::to_string_pretty(&v)?)
        .with_context(|| format!("writing profile {}", p.display()))?;
    println!(
        "✓ Profile '{}' created successfully at {}",
        name,
        p.display()
    );
    Ok(())
}

/// Get the current active profile name
fn get_current_profile() -> Result<Option<String>> {
    let current_path = current_profile_path()?;
    if !current_path.exists() {
        return Ok(None);
    }
    let name = fs::read_to_string(&current_path)
        .with_context(|| format!("reading current profile from {}", current_path.display()))?;
    Ok(Some(name.trim().to_string()))
}

/// Set the current active profile name
fn set_current_profile(name: &str) -> Result<()> {
    let current_path = current_profile_path()?;
    fs::write(&current_path, name)
        .with_context(|| format!("writing current profile to {}", current_path.display()))?;
    Ok(())
}

/// List all profiles
pub fn list_profiles() -> Result<()> {
    let dir = profiles_dir()?;
    let mut entries: Vec<_> = fs::read_dir(&dir)?
        .filter_map(|e| e.ok())
        .filter(|e| {
            // Skip hidden files (starting with .)
            if let Some(name) = e.file_name().to_str()
                && name.starts_with('.') {
                    return false;
                }

            // Only include .json files
            if let Some(ext) = e.path().extension()
                && ext == "json" {
                    return true;
                }

            false
        })
        .collect();

    entries.sort_by_key(|e| e.file_name());
    let current = get_current_profile()?;
    println!("Profiles in {}:", dir.display());
    for e in entries {
        if let Some(name) = e.path().file_stem().and_then(|s| s.to_str()) {
            if current.as_deref() == Some(name) {
                println!(" - {} (current)", name);
            } else {
                println!(" - {}", name);
            }
        }
    }
    Ok(())
}

/// Show a profile's content
pub fn show_profile(name: &str) -> Result<()> {
    let p = profile_path(name)?;
    let s = fs::read_to_string(&p).with_context(|| format!("reading profile {}", p.display()))?;
    println!("{}", s);
    Ok(())
}

/// Remove a profile
pub fn remove_profile(name: &str) -> Result<()> {
    // Check if the profile is currently active
    if let Some(current_profile) = get_current_profile()?
        && current_profile == name
    {
        println!(
            "Cannot remove profile '{}' because it is currently active.",
            name
        );
        println!("Please switch to a different profile first using: ccm switch <profile_name>");
        return Ok(());
    }

    let p = profile_path(name)?;
    if p.exists() {
        fs::remove_file(&p).with_context(|| format!("removing profile {}", p.display()))?;
        println!("Removed profile '{}'", name);
    } else {
        println!("Profile '{}' does not exist", name);
    }
    Ok(())
}

/// Switch to a profile
pub fn switch_to_profile(name: &str) -> Result<()> {
    let p = profile_path(name)?;
    if !p.exists() {
        anyhow::bail!("Profile '{}' does not exist", name);
    }
    let settings = claude_settings_path();
    if let Some(parent) = settings.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("creating settings parent dir {}", parent.display()))?;
    }
    fs::copy(&p, &settings)
        .with_context(|| format!("copying profile {} to {}", p.display(), settings.display()))?;
    set_current_profile(name)?;
    println!(
        "Switched Claude settings to profile '{}' (wrote to {})",
        name,
        settings.display()
    );
    Ok(())
}

/// Launch Claude Code with current profile
pub fn launch_claude_code() -> Result<()> {
    let current = get_current_profile()?;

    if current.is_none() {
        anyhow::bail!(
            "No profile is currently active.\n\
            Please add a profile with 'ccm add <name>' and switch to it with 'ccm switch <name>' first."
        );
    }

    let profile_name = current.unwrap();
    println!("Launching Claude Code with profile '{}'...", profile_name);

    let status = std::process::Command::new("claude").status().context(
        "Failed to launch Claude Code. Make sure 'claude' command is available in PATH.",
    )?;

    println!("Claude Code exited with: {}", status);
    Ok(())
}

/// Import current Claude settings as a new profile
pub fn import_current_profile(name: &str) -> Result<()> {
    let settings = claude_settings_path();
    if !settings.exists() {
        anyhow::bail!("No Claude settings found at {}", settings.display());
    }
    let p = profile_path(name)?;
    if p.exists() {
        anyhow::bail!("Profile '{}' already exists", name);
    }
    fs::copy(&settings, &p)
        .with_context(|| format!("copying {} to profile {}", settings.display(), p.display()))?;
    println!(
        "✓ Imported current settings to profile '{}' at {}",
        name,
        p.display()
    );
    Ok(())
}

/// Rename a profile from original name to new name
pub fn rename_profile(origin: &str, new: &str) -> Result<()> {
    // Check if origin profile exists
    let origin_path = profile_path(origin)?;
    if !origin_path.exists() {
        anyhow::bail!("Profile '{}' does not exist", origin);
    }

    // Check if new profile name already exists
    let new_path = profile_path(new)?;
    if new_path.exists() {
        anyhow::bail!("Profile '{}' already exists", new);
    }

    // Check if origin profile is currently active
    let is_current = if let Some(current_profile) = get_current_profile()? {
        current_profile == origin
    } else {
        false
    };

    // Rename the profile file
    fs::rename(&origin_path, &new_path).with_context(|| {
        format!(
            "renaming profile from {} to {}",
            origin_path.display(),
            new_path.display()
        )
    })?;

    // If the renamed profile was active, update the current profile reference
    if is_current {
        set_current_profile(new)?;
    }

    println!("✓ Profile '{}' renamed to '{}' successfully", origin, new);
    Ok(())
}

/// Edit a profile using the default editor
pub fn edit_profile(name: &str) -> Result<()> {
    let profile_path = profile_path(name)?;
    if !profile_path.exists() {
        anyhow::bail!("Profile '{}' does not exist", name);
    }

    // Get the editor from environment variables, fallback to common editors
    let editor = std::env::var("EDITOR")
        .or_else(|_| std::env::var("VISUAL"))
        .unwrap_or_else(|_| {
            // Try to detect a common editor
            if Command::new("vim").arg("--version").output().is_ok() {
                "vim".to_string()
            } else if Command::new("nano").arg("--version").output().is_ok() {
                "nano".to_string()
            } else if Command::new("vi").arg("--version").output().is_ok() {
                "vi".to_string()
            } else {
                // Fallback to a basic editor that should exist on most systems
                "vi".to_string()
            }
        });

    println!("Opening profile '{}' with editor: {}", name, editor);

    // Launch the editor with the profile file
    let status = Command::new(&editor)
        .arg(&profile_path)
        .status()
        .with_context(|| format!("Failed to launch editor '{}'", editor))?;

    if status.success() {
        println!("✓ Profile '{}' edited successfully", name);
    } else {
        anyhow::bail!("Editor exited with error code: {:?}", status.code());
    }

    Ok(())
}
