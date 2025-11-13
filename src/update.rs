use anyhow::{anyhow, Context, Result};
use flate2::read::GzDecoder;
use semver::Version;
use serde::Deserialize;
use std::env;
use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;
use tar::Archive;

const GITHUB_REPO: &str = "caibirdme/ccm";
const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Deserialize)]
struct Release {
    tag_name: String,
    assets: Vec<Asset>,
}

#[derive(Deserialize)]
struct Asset {
    name: String,
    browser_download_url: String,
}

/// Detect the current platform and return the asset name pattern
fn detect_platform() -> Result<String> {
    let os = if cfg!(target_os = "linux") {
        "Linux"
    } else if cfg!(target_os = "macos") {
        "macOS"
    } else {
        return Err(anyhow!("Unsupported operating system"));
    };

    let arch = if cfg!(target_arch = "x86_64") {
        "X64"
    } else if cfg!(target_arch = "aarch64") {
        "ARM64"
    } else {
        return Err(anyhow!("Unsupported architecture"));
    };

    Ok(format!("{}-{}", os, arch))
}

/// Fetch the latest release from GitHub
fn fetch_latest_release() -> Result<Release> {
    let url = format!("https://api.github.com/repos/{}/releases/latest", GITHUB_REPO);
    
    let client = reqwest::blocking::Client::builder()
        .user_agent("ccm-updater")
        .build()
        .context("Failed to create HTTP client")?;

    let mut request = client.get(&url);

    // Add GitHub token if available for higher rate limits
    if let Ok(token) = env::var("GITHUB_TOKEN") {
        request = request.bearer_auth(token);
    }

    let response = request
        .send()
        .context("Failed to fetch latest release from GitHub")?;

    if !response.status().is_success() {
        return Err(anyhow!(
            "GitHub API returned error: {} - {}",
            response.status(),
            response.text().unwrap_or_default()
        ));
    }

    response
        .json::<Release>()
        .context("Failed to parse GitHub release response")
}

/// Compare current version with latest version
fn compare_versions(current: &str, latest: &str) -> Result<std::cmp::Ordering> {
    // Remove 'v' prefix if present
    let current_clean = current.trim_start_matches('v');
    let latest_clean = latest.trim_start_matches('v');

    let current_ver = Version::parse(current_clean)
        .with_context(|| format!("Failed to parse current version: {}", current))?;
    let latest_ver = Version::parse(latest_clean)
        .with_context(|| format!("Failed to parse latest version: {}", latest))?;

    Ok(current_ver.cmp(&latest_ver))
}

/// Download the release asset and extract the binary
fn download_and_extract(asset_url: &str, asset_name: &str) -> Result<PathBuf> {
    println!("📥 Downloading {}...", asset_name);

    let client = reqwest::blocking::Client::builder()
        .user_agent("ccm-updater")
        .build()
        .context("Failed to create HTTP client")?;

    let response = client
        .get(asset_url)
        .send()
        .context("Failed to download release asset")?;

    if !response.status().is_success() {
        return Err(anyhow!("Failed to download: {}", response.status()));
    }

    // Download to temporary file
    let temp_dir = env::temp_dir();
    let temp_archive = temp_dir.join(asset_name);
    
    let mut file = File::create(&temp_archive)
        .context("Failed to create temporary file")?;
    
    let content = response.bytes().context("Failed to read download content")?;
    file.write_all(&content)
        .context("Failed to write downloaded content")?;
    
    drop(file); // Close the file

    println!("📦 Extracting binary...");

    // Extract the tar.gz archive
    let tar_gz = File::open(&temp_archive)
        .context("Failed to open downloaded archive")?;
    let tar = GzDecoder::new(tar_gz);
    let mut archive = Archive::new(tar);

    // Extract to a temporary directory
    let extract_dir = temp_dir.join("ccm-extract");
    if extract_dir.exists() {
        fs::remove_dir_all(&extract_dir).ok();
    }
    fs::create_dir_all(&extract_dir)
        .context("Failed to create extraction directory")?;

    archive
        .unpack(&extract_dir)
        .context("Failed to extract archive")?;

    // Find the ccm binary in the extracted files
    let new_binary = extract_dir.join("ccm");
    if !new_binary.exists() {
        return Err(anyhow!("Binary 'ccm' not found in archive"));
    }

    // Cleanup the archive file
    fs::remove_file(&temp_archive).ok();

    Ok(new_binary)
}

/// Install the new binary, replacing the current one
fn install_binary(new_binary: &PathBuf) -> Result<()> {
    println!("🔄 Installing update...");

    // Get current executable path
    let current_exe = env::current_exe()
        .context("Failed to get current executable path")?;

    // Create backup of current binary
    let backup_path = current_exe.with_extension("backup");
    fs::copy(&current_exe, &backup_path)
        .context("Failed to create backup of current binary")?;

    // Replace the binary
    // We use a temporary file + rename for atomic replacement
    let temp_new = current_exe.with_extension("new");
    fs::copy(&new_binary, &temp_new)
        .context("Failed to copy new binary")?;

    // Set executable permissions on Unix
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&temp_new)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&temp_new, perms)
            .context("Failed to set executable permissions")?;
    }

    // Atomic rename
    if let Err(e) = fs::rename(&temp_new, &current_exe) {
        // Rollback on failure
        fs::copy(&backup_path, &current_exe).ok();
        fs::remove_file(&backup_path).ok();
        return Err(e).context("Failed to replace binary, rolled back to previous version");
    }

    // Cleanup extraction directory
    if let Some(extract_dir) = new_binary.parent() {
        fs::remove_dir_all(extract_dir).ok();
    }
    
    println!("✓ Update installed successfully!");
    println!("  Backup saved to: {}", backup_path.display());

    Ok(())
}

/// Main update function
pub fn update_self(check_only: bool) -> Result<()> {
    println!("🔍 Checking for updates...");

    let platform = detect_platform()
        .context("Failed to detect platform")?;

    let release = fetch_latest_release()
        .context("Failed to fetch latest release information")?;

    let latest_version = &release.tag_name;
    
    println!("  Current version: v{}", CURRENT_VERSION);
    println!("  Latest version:  {}", latest_version);

    match compare_versions(CURRENT_VERSION, latest_version)? {
        std::cmp::Ordering::Less => {
            println!("🎉 A new version is available!");
            
            if check_only {
                println!("\nRun 'ccm update' to install the latest version.");
                return Ok(());
            }

            // Find the appropriate asset for this platform
            let asset_pattern = format!("ccm-{}-{}.tar.gz", latest_version, platform);
            let asset = release
                .assets
                .iter()
                .find(|a| a.name == asset_pattern)
                .ok_or_else(|| anyhow!("No release asset found for platform: {} (expected: {})", platform, asset_pattern))?;

            let new_binary = download_and_extract(&asset.browser_download_url, &asset.name)?;
            install_binary(&new_binary)?;
            
            println!("\n🚀 Update complete! Please restart ccm to use the new version.");
        }
        std::cmp::Ordering::Equal => {
            println!("✓ You are already running the latest version.");
        }
        std::cmp::Ordering::Greater => {
            println!("ℹ️  You are running a newer version than the latest release.");
            println!("   This might be a development or pre-release version.");
        }
    }

    Ok(())
}
