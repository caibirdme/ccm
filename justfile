# Build and install tasks for claude-config

# Default recipe to display help
default:
    @just --list

# Build in release mode
build:
    cargo build --release
    @echo ""
    @echo "✓ Build successful!"
    @echo "Binary: target/release/ccm"

# Build in debug mode
build-dev:
    cargo build

# Run tests
test:
    cargo test

# Clean build artifacts
clean:
    cargo clean

# Check code formatting
fmt-check:
    cargo fmt -- --check

# Format code
fmt:
    cargo fmt

# Run clippy linter
lint:
    cargo clippy -- -D warnings

# Run all checks (fmt, lint, test)
check: fmt-check lint test
    @echo "✓ All checks passed!"

# Show version
version:
    @cargo pkgid | cut -d'#' -f2

# Install the built binary to ~/.cargo/bin for system-wide access
install:
    @echo "Installing ccm to ~/.cargo/bin/ccm"
    @cargo install --path .
    @echo "✓ Installed!"
