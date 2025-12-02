use anyhow::{Context, Result};
use rpassword::read_password;
use serde_json::Value;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::config::{
    claude_settings_path, current_profile_path, ensure_ccm_dir, ensure_profiles_dir,
    get_current_working_dir, get_project_profile_info, profile_path, remove_project_profile,
    set_project_profile,
};

/// Display a simple JSON diff by showing both values side by side
fn display_json_diff(profile_name: &str, profile_value: &Value, settings_value: &Value) {
    println!("\n⚠️  Configuration mismatch detected!");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!(
        "Current profile '{}' differs from settings.json\n",
        profile_name
    );

    println!("Profile '{}' content:", profile_name);
    println!(
        "{}",
        serde_json::to_string_pretty(profile_value).unwrap_or_default()
    );

    println!("\nsettings.json content:");
    println!(
        "{}",
        serde_json::to_string_pretty(settings_value).unwrap_or_default()
    );
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
}

/// Prompt user to choose what to do when profile differs from settings
fn prompt_switch_action() -> Result<u32> {
    println!("What would you like to do?");
    println!("  1: Switch directly (ignore the difference)");
    println!("  2: Update current profile with settings.json, then switch");
    println!("  3: Cancel switch operation");
    print!("\nYour choice [1-3]: ");
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    match input.trim().parse::<u32>() {
        Ok(n) if (1..=3).contains(&n) => Ok(n),
        _ => {
            println!("Invalid choice. Operation cancelled.");
            Ok(3)
        }
    }
}

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
    ensure_profiles_dir()?;
    let p = profile_path(name);
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
pub fn get_current_profile() -> Result<Option<String>> {
    let current_path = current_profile_path();
    if !current_path.exists() {
        return Ok(None);
    }
    let name = fs::read_to_string(&current_path)
        .with_context(|| format!("reading current profile from {}", current_path.display()))?;
    Ok(Some(name.trim().to_string()))
}

/// Get the current active profile name for a specific project directory
pub fn get_project_current_profile(project_dir: &PathBuf) -> Result<Option<String>> {
    if let Some((profile_name, _)) = get_project_profile_info(project_dir)? {
        Ok(Some(profile_name))
    } else {
        Ok(None)
    }
}

/// Set the current active profile name
fn set_current_profile(name: &str) -> Result<()> {
    ensure_ccm_dir()?;
    let current_path = current_profile_path();
    fs::write(&current_path, name)
        .with_context(|| format!("writing current profile to {}", current_path.display()))?;
    Ok(())
}

/// Set the current active profile name for a project
fn set_project_current_profile(project_dir: &PathBuf, name: &str) -> Result<()> {
    set_project_profile(project_dir, name)
}

/// List all profiles
pub fn list_profiles() -> Result<()> {
    let dir = ensure_profiles_dir()?;
    let mut entries: Vec<_> = fs::read_dir(&dir)?
        .filter_map(|e| e.ok())
        .filter(|e| {
            // Skip hidden files (starting with .)
            if let Some(name) = e.file_name().to_str()
                && name.starts_with('.')
            {
                return false;
            }

            // Only include .json files
            if let Some(ext) = e.path().extension()
                && ext == "json"
            {
                return true;
            }

            false
        })
        .collect();

    entries.sort_by_key(|e| e.file_name());
    let global_current = get_current_profile()?;

    // Always check if current directory has a project profile
    let cwd = get_current_working_dir()?;
    let project_current = get_project_current_profile(&cwd)?;

    println!("Profiles in {}:", dir.display());

    for e in entries {
        if let Some(name) = e.path().file_stem().and_then(|s| s.to_str()) {
            let is_global_current = global_current.as_deref() == Some(name);
            let is_project_current = project_current.as_deref() == Some(name);

            if is_global_current && is_project_current {
                // Both global and project point to same profile, just show (current)
                println!(" - {} (current)", name);
            } else if is_project_current {
                println!(" - {} (current project)", name);
            } else if is_global_current {
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
    let p = profile_path(name);
    let s = fs::read_to_string(&p).with_context(|| format!("reading profile {}", p.display()))?;
    println!("{}", s);
    Ok(())
}

/// Remove a profile
pub fn remove_profile(name: &str) -> Result<()> {
    // Check if the profile is currently active (global)
    if let Some(current_profile) = get_current_profile()?
        && current_profile == name
    {
        println!(
            "Cannot remove profile '{}' because it is currently active (global).",
            name
        );
        println!("Please switch to a different profile first using: ccm switch <profile_name>");
        return Ok(());
    }

    // Also check if it's the current project profile
    if let Ok(cwd) = get_current_working_dir()
        && let Some(project_current) = get_project_current_profile(&cwd)?
        && project_current == name
    {
        println!(
            "Cannot remove profile '{}' because it is currently active for this project.",
            name
        );
        println!("Please switch to a different profile first using: ccm switch -p <profile_name>");
        return Ok(());
    }

    let p = profile_path(name);
    if p.exists() {
        fs::remove_file(&p).with_context(|| format!("removing profile {}", p.display()))?;
        println!("Removed profile '{}'", name);
    } else {
        println!("Profile '{}' does not exist", name);
    }
    Ok(())
}

/// Merge profile into existing settings (profile fields override existing ones)
fn merge_json(base: &mut Value, overlay: &Value) {
    match (base, overlay) {
        (Value::Object(base_map), Value::Object(overlay_map)) => {
            for (key, overlay_value) in overlay_map {
                match base_map.get_mut(key) {
                    Some(base_value) => {
                        // Recursively merge objects
                        merge_json(base_value, overlay_value);
                    }
                    None => {
                        // Key doesn't exist in base, insert it
                        base_map.insert(key.clone(), overlay_value.clone());
                    }
                }
            }
        }
        (base, overlay) => {
            // For non-object values, overlay replaces base
            *base = overlay.clone();
        }
    }
}

/// Get the project settings.local.json path
fn project_settings_local_path(project_dir: &Path) -> PathBuf {
    project_dir.join(".claude").join("settings.local.json")
}

/// Handle project-level switch: merge profile into .claude/settings.local.json
fn switch_project_profile(name: &str, profile_content: &str, profile_value: &Value) -> Result<()> {
    let cwd = get_current_working_dir()?;
    let local_settings_path = project_settings_local_path(&cwd);

    // Ensure .claude directory exists
    if let Some(parent) = local_settings_path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("creating .claude dir {}", parent.display()))?;
    }

    let final_content = if local_settings_path.exists() {
        // Merge: read existing settings and merge profile on top
        let existing_content = fs::read_to_string(&local_settings_path)
            .with_context(|| format!("reading {}", local_settings_path.display()))?;
        let mut existing_value: Value = serde_json::from_str(&existing_content)
            .with_context(|| format!("parsing JSON from {}", local_settings_path.display()))?;

        merge_json(&mut existing_value, profile_value);
        serde_json::to_string_pretty(&existing_value)?
    } else {
        // No existing file, use profile content directly
        profile_content.to_string()
    };

    fs::write(&local_settings_path, &final_content)
        .with_context(|| format!("writing {}", local_settings_path.display()))?;

    // Track the project-profile mapping
    set_project_current_profile(&cwd, name)?;

    println!(
        "✓ Switched to profile '{}' for project {} (wrote to {})",
        name,
        cwd.display(),
        local_settings_path.display()
    );
    Ok(())
}

/// Check if current profile differs from settings.json and prompt user for action
/// Returns true if switch should proceed, false if cancelled
fn handle_profile_mismatch_check() -> Result<bool> {
    let current_profile_name = match get_current_profile()? {
        Some(name) => name,
        None => return Ok(true), // No current profile, proceed
    };

    let settings_path = claude_settings_path();
    if !settings_path.exists() {
        return Ok(true);
    }

    let current_profile_path = profile_path(&current_profile_name);
    if !current_profile_path.exists() {
        return Ok(true);
    }

    let settings_content = fs::read_to_string(&settings_path)
        .with_context(|| format!("reading settings {}", settings_path.display()))?;
    let current_profile_content = fs::read_to_string(&current_profile_path)
        .with_context(|| format!("reading profile {}", current_profile_path.display()))?;

    let settings_value: Value = serde_json::from_str(&settings_content)
        .with_context(|| format!("parsing settings JSON from {}", settings_path.display()))?;
    let current_profile_value: Value = serde_json::from_str(&current_profile_content)
        .with_context(|| {
            format!(
                "parsing profile JSON from {}",
                current_profile_path.display()
            )
        })?;

    if settings_value == current_profile_value {
        return Ok(true);
    }

    display_json_diff(
        &current_profile_name,
        &current_profile_value,
        &settings_value,
    );

    match prompt_switch_action()? {
        1 => {
            println!("Proceeding with switch...");
            Ok(true)
        }
        2 => {
            println!(
                "Updating profile '{}' with current settings.json...",
                current_profile_name
            );
            fs::write(&current_profile_path, &settings_content)
                .with_context(|| format!("writing profile {}", current_profile_path.display()))?;
            println!("✓ Profile '{}' updated successfully", current_profile_name);
            Ok(true)
        }
        _ => {
            println!("Switch operation cancelled.");
            Ok(false)
        }
    }
}

/// Handle global switch: overwrite ~/.claude/settings.json
fn switch_global_profile(name: &str, profile_path_ref: &PathBuf) -> Result<()> {
    if !handle_profile_mismatch_check()? {
        return Ok(());
    }

    let settings = claude_settings_path();
    if let Some(parent) = settings.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("creating settings parent dir {}", parent.display()))?;
    }
    fs::copy(profile_path_ref, &settings).with_context(|| {
        format!(
            "copying profile {} to {}",
            profile_path_ref.display(),
            settings.display()
        )
    })?;

    set_current_profile(name)?;
    println!(
        "✓ Switched Claude settings to profile '{}' (wrote to {})",
        name,
        settings.display()
    );
    Ok(())
}

/// Switch to a profile
pub fn switch_to_profile(name: &str, project_mode: bool) -> Result<()> {
    let p = profile_path(name);
    if !p.exists() {
        anyhow::bail!("Profile '{}' does not exist", name);
    }

    let profile_content =
        fs::read_to_string(&p).with_context(|| format!("reading profile {}", p.display()))?;
    let profile_value: Value = serde_json::from_str(&profile_content)
        .with_context(|| format!("parsing profile JSON from {}", p.display()))?;

    if project_mode {
        switch_project_profile(name, &profile_content, &profile_value)
    } else {
        switch_global_profile(name, &p)
    }
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
    ensure_profiles_dir()?;
    let p = profile_path(name);
    if p.exists() {
        anyhow::bail!("Profile '{}' already exists", name);
    }
    fs::copy(&settings, &p)
        .with_context(|| format!("copying {} to profile {}", settings.display(), p.display()))?;

    set_current_profile(name)?;
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
    let origin_path = profile_path(origin);
    if !origin_path.exists() {
        anyhow::bail!("Profile '{}' does not exist", origin);
    }

    // Check if new profile name already exists
    let new_path = profile_path(new);
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
    let profile_path = profile_path(name);
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

/// Sync current profile with current Claude settings
pub fn sync_profile() -> Result<()> {
    let current_profile = get_current_profile()?.ok_or_else(|| {
        anyhow::anyhow!(
            "No profile is currently active.\n\
            Please switch to a profile first using: ccm switch <profile_name>"
        )
    })?;

    let settings_path = claude_settings_path();
    if !settings_path.exists() {
        anyhow::bail!("No Claude settings found at {}", settings_path.display());
    }

    let profile_file_path = profile_path(&current_profile);
    if !profile_file_path.exists() {
        anyhow::bail!("Current profile '{}' does not exist", current_profile);
    }

    // Read both files
    let settings_content = fs::read_to_string(&settings_path)
        .with_context(|| format!("reading settings {}", settings_path.display()))?;
    let profile_content = fs::read_to_string(&profile_file_path)
        .with_context(|| format!("reading profile {}", profile_file_path.display()))?;

    // Parse JSON to compare
    let settings_value: Value = serde_json::from_str(&settings_content)
        .with_context(|| format!("parsing settings JSON from {}", settings_path.display()))?;
    let profile_value: Value = serde_json::from_str(&profile_content)
        .with_context(|| format!("parsing profile JSON from {}", profile_file_path.display()))?;

    // Compare the JSON content
    if settings_value == profile_value {
        println!(
            "✓ Claude settings and current profile '{}' are already in sync",
            current_profile
        );
        Ok(())
    } else {
        // Sync: update profile to match settings
        fs::write(&profile_file_path, settings_content)
            .with_context(|| format!("writing profile {}", profile_file_path.display()))?;
        println!(
            "✓ Synced current profile '{}' with Claude settings (updated {})",
            current_profile,
            profile_file_path.display()
        );
        Ok(())
    }
}

/// Remove keys from base JSON that exist in overlay JSON (recursive)
fn remove_json_keys(base: &mut Value, overlay: &Value) {
    if let (Value::Object(base_map), Value::Object(overlay_map)) = (base, overlay) {
        let keys_to_remove: Vec<String> = overlay_map
            .iter()
            .filter_map(|(key, overlay_value)| {
                if let Some(base_value) = base_map.get(key) {
                    if matches!(
                        (base_value, overlay_value),
                        (Value::Object(_), Value::Object(_))
                    ) {
                        None // Will handle recursively
                    } else {
                        Some(key.clone()) // Mark for removal
                    }
                } else {
                    None
                }
            })
            .collect();

        // Remove non-object keys
        for key in keys_to_remove {
            base_map.remove(&key);
        }

        // Handle nested objects recursively
        for (key, overlay_value) in overlay_map {
            if let Some(base_value) = base_map.get_mut(key)
                && let (Value::Object(_), Value::Object(_)) = (base_value.clone(), overlay_value)
            {
                remove_json_keys(base_value, overlay_value);
                // If the object is now empty, remove the key
                if let Value::Object(m) = base_value
                    && m.is_empty()
                {
                    base_map.remove(key);
                }
            }
        }
    }
}

/// Clear project-specific profile setting (revert to using global profile)
pub fn clear_project_profile() -> Result<()> {
    let cwd = get_current_working_dir()?;

    let project_profile_name = match get_project_current_profile(&cwd)? {
        Some(name) => name,
        None => {
            println!("No project-specific profile is set for {}", cwd.display());
            return Ok(());
        }
    };

    let local_settings_path = project_settings_local_path(&cwd);

    // Check if profile exists
    let profile_file_path = profile_path(&project_profile_name);
    if !profile_file_path.exists() {
        println!(
            "⚠️  Profile '{}' not found. The project mapping may be corrupted.",
            project_profile_name
        );
        println!(
            "Please manually delete {} if needed.",
            local_settings_path.display()
        );
        // Still remove the project mapping
        remove_project_profile(&cwd)?;
        return Ok(());
    }

    // Remove profile fields from settings.local.json
    if local_settings_path.exists() {
        let profile_content = fs::read_to_string(&profile_file_path)
            .with_context(|| format!("reading profile {}", profile_file_path.display()))?;
        let profile_value: Value = serde_json::from_str(&profile_content).with_context(|| {
            format!("parsing profile JSON from {}", profile_file_path.display())
        })?;

        let settings_content = fs::read_to_string(&local_settings_path)
            .with_context(|| format!("reading {}", local_settings_path.display()))?;
        let mut settings_value: Value = serde_json::from_str(&settings_content)
            .with_context(|| format!("parsing JSON from {}", local_settings_path.display()))?;

        // Remove profile keys from settings
        remove_json_keys(&mut settings_value, &profile_value);

        // Write back or delete if empty
        if let Value::Object(map) = &settings_value {
            if map.is_empty() {
                fs::remove_file(&local_settings_path)
                    .with_context(|| format!("removing {}", local_settings_path.display()))?;
                println!(
                    "✓ Removed {} (no remaining settings)",
                    local_settings_path.display()
                );
            } else {
                fs::write(
                    &local_settings_path,
                    serde_json::to_string_pretty(&settings_value)?,
                )
                .with_context(|| format!("writing {}", local_settings_path.display()))?;
                println!(
                    "✓ Removed profile '{}' fields from {}",
                    project_profile_name,
                    local_settings_path.display()
                );
            }
        }
    }

    remove_project_profile(&cwd)?;
    println!(
        "✓ Cleared project-specific profile for {}. Will now use global profile.",
        cwd.display()
    );
    Ok(())
}
