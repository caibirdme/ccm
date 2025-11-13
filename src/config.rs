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

/// Get the path to the current profile marker file
pub fn current_profile_path() -> PathBuf {
    ccm_dir().join("current")
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
