#!/bin/bash

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# GitHub repository
GITHUB_REPO="caibirdme/ccm"
INSTALL_DIR="$HOME/.local/bin"
BINARY_NAME="ccm"

# Function to print colored output
print_info() {
    echo -e "${GREEN}[INFO]${NC} $1" >&2
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1" >&2
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1" >&2
}

# Check if running on Windows
if [[ "$OSTYPE" == "msys" ]] || [[ "$OSTYPE" == "cygwin" ]] || [[ "$OSTYPE" == "win32" ]]; then
    print_error "Windows platform is not supported. Please install manually."
    exit 1
fi

# Detect OS and architecture
detect_platform() {
    local os
    local arch

    # Detect OS
    case "$OSTYPE" in
        linux*)
            os="Linux"
            ;;
        darwin*)
            os="macOS"
            ;;
        *)
            print_error "Unsupported operating system: $OSTYPE"
            exit 1
            ;;
    esac

    # Detect architecture
    case "$(uname -m)" in
        x86_64|amd64)
            arch="X64"
            ;;
        aarch64|arm64)
            arch="ARM64"
            ;;
        *)
            print_error "Unsupported architecture: $(uname -m)"
            exit 1
            ;;
    esac

    echo "${os}-${arch}"
}

# Get the latest release information from GitHub
get_latest_release() {
    local api_url="https://api.github.com/repos/${GITHUB_REPO}/releases/latest"

    print_info "Fetching latest release information..."

    # Try to get release info using curl with different options
    local release_info

    if command -v curl >/dev/null 2>&1; then
        # Use GITHUB_TOKEN if available to avoid rate limiting
        if [[ -n "${GITHUB_TOKEN:-}" ]]; then
            print_info "Using GITHUB_TOKEN for authenticated API request"
            release_info=$(curl -s -H "Authorization: token ${GITHUB_TOKEN}" "$api_url") || {
                print_error "Failed to fetch release information. Please check your internet connection."
                exit 1
            }
        else
            release_info=$(curl -s "$api_url") || {
                print_error "Failed to fetch release information. Please check your internet connection."
                exit 1
            }
        fi
    else
        print_error "curl is required but not installed. Please install curl and try again."
        exit 1
    fi

    # Extract tag name from release info
    local tag_name
    # Try to extract tag_name using different methods
    if command -v jq >/dev/null 2>&1; then
        # Use jq if available (more reliable)
        tag_name=$(echo "$release_info" | jq -r .tag_name 2>/dev/null) || tag_name=""
    else
        # Fallback to grep method
        tag_name=$(echo "$release_info" | grep -o '"tag_name": "[^"]*' | grep -o '[^"]*$') || tag_name=""
    fi

    if [[ -z "$tag_name" || "$tag_name" == "null" ]]; then
        print_error "Failed to parse release information. The repository might be private or the API is rate-limited."
        if [[ -z "${GITHUB_TOKEN:-}" ]]; then
            print_info "Tip: Set GITHUB_TOKEN environment variable to avoid rate limiting"
        fi
        exit 1
    fi

    echo "$tag_name"
}

# Download the binary
download_binary() {
    local platform=$1
    local tag=$2
    local asset_name="ccm-${tag}-${platform}.tar.gz"
    local download_url="https://github.com/${GITHUB_REPO}/releases/download/${tag}/${asset_name}"
    local temp_file="/tmp/${asset_name}"

    print_info "Downloading ccm binary for ${platform}..."
    print_info "URL: ${download_url}"

    # Download the binary
    if [[ -n "${GITHUB_TOKEN:-}" ]]; then
        if ! curl -L -H "Authorization: token ${GITHUB_TOKEN}" -o "$temp_file" "$download_url"; then
            print_error "Failed to download the binary. It might not exist for your platform."
            print_info "Available platforms are typically: Linux-X64, macOS-ARM64"
            exit 1
        fi
    else
        if ! curl -L -o "$temp_file" "$download_url"; then
            print_error "Failed to download the binary. It might not exist for your platform."
            print_info "Available platforms are typically: Linux-X64, macOS-ARM64"
            exit 1
        fi
    fi

    echo "$temp_file"
}

# Make binary executable and move to installation directory
install_binary() {
    local temp_file=$1
    local install_path="${INSTALL_DIR}/${BINARY_NAME}"

    # Create installation directory if it doesn't exist
    if [[ ! -d "$INSTALL_DIR" ]]; then
        print_info "Creating installation directory: $INSTALL_DIR"
        mkdir -p "$INSTALL_DIR"
    fi

    # Extract the tar.gz file
    print_info "Extracting binary from archive..."
    local temp_dir="/tmp/ccm-extract-$$"
    mkdir -p "$temp_dir"

    if ! tar -xzf "$temp_file" -C "$temp_dir"; then
        print_error "Failed to extract the archive"
        rm -rf "$temp_dir" "$temp_file"
        exit 1
    fi

    # Find the extracted binary
    local extracted_binary
    extracted_binary=$(find "$temp_dir" -name "$BINARY_NAME" -type f | head -1)

    if [[ -z "$extracted_binary" ]]; then
        print_error "Could not find ccm binary in the extracted archive"
        rm -rf "$temp_dir" "$temp_file"
        exit 1
    fi

    # Make the binary executable
    chmod +x "$extracted_binary"

    # Move the binary to installation directory
    if mv "$extracted_binary" "$install_path"; then
        print_info "Successfully installed ccm to $install_path"
    else
        print_error "Failed to install ccm to $install_path"
        rm -rf "$temp_dir" "$temp_file"
        exit 1
    fi

    # Cleanup
    rm -rf "$temp_dir" "$temp_file"
}

# Add installation directory to PATH if needed
update_path() {
    local shell_rc

    # Detect shell and corresponding rc file
    case "$SHELL" in
        */bash)
            shell_rc="$HOME/.bashrc"
            ;;
        */zsh)
            shell_rc="$HOME/.zshrc"
            ;;
        */fish)
            shell_rc="$HOME/.config/fish/config.fish"
            ;;
        *)
            print_warning "Could not detect shell type. You may need to manually add $INSTALL_DIR to your PATH."
            return
            ;;
    esac

    # Check if INSTALL_DIR is already in PATH
    if [[ ":$PATH:" == *":$INSTALL_DIR:"* ]]; then
        print_info "$INSTALL_DIR is already in your PATH"
    else
        print_info "Adding $INSTALL_DIR to your PATH in $shell_rc"

        # Add to shell rc file
        echo "" >> "$shell_rc"
        echo "# Added by ccm installer" >> "$shell_rc"
        echo "export PATH=\"\$PATH:$INSTALL_DIR\"" >> "$shell_rc"

        print_info "Please run: source $shell_rc"
        print_info "Or restart your terminal to update your PATH"
    fi
}

# Verify installation
verify_installation() {
    local install_path="${INSTALL_DIR}/${BINARY_NAME}"

    if [[ -x "$install_path" ]]; then
        print_info "Verifying installation..."
        if "$install_path" --version >/dev/null 2>&1; then
            print_info "Installation verified successfully!"
            print_info "Run 'ccm --help' to get started"
        else
            print_warning "Installation completed but 'ccm --version' failed. The binary might be corrupted."
        fi
    else
        print_error "Installation failed - binary not found or not executable"
        exit 1
    fi
}

# Main installation function
main() {
    print_info "Installing ccm (Claude Config Manager)..."

    # Detect platform
    local platform
    platform=$(detect_platform)
    print_info "Detected platform: $platform"

    # Get latest release
    local tag
    tag=$(get_latest_release)
    print_info "Latest release: $tag"

    # Download binary
    local temp_file
    temp_file=$(download_binary "$platform" "$tag")

    # Install binary
    install_binary "$temp_file"

    # Update PATH
    update_path

    # Verify installation
    verify_installation

    print_info "Installation completed successfully!"
    print_info "You can now use 'ccm' command to manage your Claude Code profiles."
}

# Run main function
main "$@"