use anyhow::{Context, Result};
use dirs::config_dir;
use std::env;
use std::fs;
use std::path::PathBuf;

/// Get the ccm base directory path (without creating it)
/// Can be overridden with CCM_CONFIG_DIR environment variable for testing
pub fn ccm_dir() -> PathBuf {
    if let Ok(custom_dir) = env::var("CCM_CONFIG_DIR") {
        PathBuf::from(custom_dir)
    } else {
        config_dir()
            .unwrap_or_else(|| PathBuf::from("./"))
            .join("ccm")
    }
}

/// Get the profiles directory path (without creating it)
/// Can be overridden with CCM_CONFIG_DIR environment variable for testing
pub fn profiles_dir() -> PathBuf {
    ccm_dir().join("profiles")
}

/// Get the project profiles directory path (without creating it)
/// Project profiles are stored in a subdirectory named "projects" within the ccm directory
pub fn project_profiles_dir() -> PathBuf {
    ccm_dir().join("projects")
}

/// Ensure the ccm directory exists, creating it if necessary
pub fn ensure_ccm_dir() -> Result<PathBuf> {
    let dir = ccm_dir();
    fs::create_dir_all(&dir).with_context(|| format!("creating ccm dir: {}", dir.display()))?;
    Ok(dir)
}

/// Ensure the profiles directory exists, creating it if necessary
pub fn ensure_profiles_dir() -> Result<PathBuf> {
    let dir = profiles_dir();
    fs::create_dir_all(&dir)
        .with_context(|| format!("creating profiles dir: {}", dir.display()))?;
    Ok(dir)
}

/// Ensure the project profiles directory exists, creating it if necessary
pub fn ensure_project_profiles_dir() -> Result<PathBuf> {
    let dir = project_profiles_dir();
    fs::create_dir_all(&dir)
        .with_context(|| format!("creating project profiles dir: {}", dir.display()))?;
    Ok(dir)
}

/// Get the path to the current profile marker file (global)
pub fn current_profile_path() -> PathBuf {
    ccm_dir().join("current")
}

/// Get the current working directory
pub fn get_current_working_dir() -> Result<PathBuf> {
    env::current_dir().context("getting current working directory")
}

/// Hash a directory path to create a unique identifier for the project
/// Uses a simple hash to avoid filesystem-unfriendly characters
fn hash_path(path: &PathBuf) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    path.hash(&mut hasher);
    format!("{:016x}", hasher.finish())
}

/// Get the path to the project profile marker file for a specific directory
/// The file name is based on a hash of the directory path
pub fn project_profile_path(project_dir: &PathBuf) -> PathBuf {
    let hash = hash_path(project_dir);
    project_profiles_dir().join(format!("{}.json", hash))
}

/// Get the project profile info (profile name and original path) for a directory
/// Returns (profile_name, original_path) if exists
pub fn get_project_profile_info(project_dir: &PathBuf) -> Result<Option<(String, PathBuf)>> {
    let path = project_profile_path(project_dir);
    if !path.exists() {
        return Ok(None);
    }
    let content = fs::read_to_string(&path)
        .with_context(|| format!("reading project profile from {}", path.display()))?;
    let value: serde_json::Value = serde_json::from_str(&content)
        .with_context(|| format!("parsing project profile JSON from {}", path.display()))?;

    let profile_name = value
        .get("profile")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    let original_path = value
        .get("path")
        .and_then(|v| v.as_str())
        .map(PathBuf::from);

    match (profile_name, original_path) {
        (Some(name), Some(path)) => Ok(Some((name, path))),
        _ => Ok(None),
    }
}

/// Set the project profile for a specific directory
pub fn set_project_profile(project_dir: &PathBuf, profile_name: &str) -> Result<()> {
    ensure_project_profiles_dir()?;
    let path = project_profile_path(project_dir);
    let content = serde_json::json!({
        "profile": profile_name,
        "path": project_dir.to_string_lossy()
    });
    fs::write(&path, serde_json::to_string_pretty(&content)?)
        .with_context(|| format!("writing project profile to {}", path.display()))?;
    Ok(())
}

/// Remove the project profile for a specific directory
pub fn remove_project_profile(project_dir: &PathBuf) -> Result<()> {
    let path = project_profile_path(project_dir);
    if path.exists() {
        fs::remove_file(&path)
            .with_context(|| format!("removing project profile {}", path.display()))?;
    }
    Ok(())
}

/// Get the Claude settings path (can be overridden with CLAUDE_SETTINGS_PATH env var)
pub fn claude_settings_path() -> PathBuf {
    if let Ok(p) = env::var("CLAUDE_SETTINGS_PATH") {
        PathBuf::from(p)
    } else if let Ok(home) = env::var("HOME") {
        PathBuf::from(home).join(".claude").join("settings.json")
    } else {
        PathBuf::from("./settings.json")
    }
}

/// Get the path for a specific profile (without creating directories)
pub fn profile_path(name: &str) -> PathBuf {
    profiles_dir().join(format!("{}.json", name))
}
