use anyhow::{Context, Result};
use dirs::config_dir;
use std::env;
use std::fs;
use std::path::PathBuf;

/// Get the profiles directory path
pub fn profiles_dir() -> Result<PathBuf> {
    let base = config_dir().unwrap_or_else(|| PathBuf::from("./"));
    let dir = base.join("ccm").join("profiles");
    fs::create_dir_all(&dir)
        .with_context(|| format!("creating profiles dir: {}", dir.display()))?;
    Ok(dir)
}

/// Get the ccm state directory path
pub fn ccm_dir() -> Result<PathBuf> {
    let base = config_dir().unwrap_or_else(|| PathBuf::from("./"));
    let dir = base.join("ccm");
    fs::create_dir_all(&dir).with_context(|| format!("creating ccm dir: {}", dir.display()))?;
    Ok(dir)
}

/// Get the path to the current profile marker file
pub fn current_profile_path() -> Result<PathBuf> {
    let dir = ccm_dir()?;
    Ok(dir.join("current"))
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

/// Get the path for a specific profile
pub fn profile_path(name: &str) -> Result<PathBuf> {
    let dir = profiles_dir()?;
    Ok(dir.join(format!("{}.json", name)))
}
