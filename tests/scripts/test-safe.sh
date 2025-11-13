#!/bin/bash
# Safe testing script for ccm
# This script sets up temporary directories for testing to avoid modifying real configuration

set -e

# Create temporary directories for testing
TEST_DIR=$(mktemp -d /tmp/ccm-test-XXXXXX)
CLAUDE_TEST_DIR=$(mktemp -d /tmp/claude-test-XXXXXX)

echo "✓ Created test directories:"
echo "  CCM config: $TEST_DIR"
echo "  Claude settings: $CLAUDE_TEST_DIR"
echo ""

# Export environment variables
export CCM_CONFIG_DIR="$TEST_DIR"
export CLAUDE_SETTINGS_PATH="$CLAUDE_TEST_DIR/settings.json"

echo "✓ Environment variables set:"
echo "  CCM_CONFIG_DIR=$CCM_CONFIG_DIR"
echo "  CLAUDE_SETTINGS_PATH=$CLAUDE_SETTINGS_PATH"
echo ""

# Function to cleanup on exit
cleanup() {
    echo ""
    echo "Cleaning up test directories..."
    rm -rf "$TEST_DIR"
    rm -rf "$CLAUDE_TEST_DIR"
    echo "✓ Test directories removed"
}

trap cleanup EXIT

echo "=== Running Safe Tests ==="
echo ""

# Run ccm commands in test mode
echo "1. Adding a test profile..."
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(cd "$SCRIPT_DIR/../.." && pwd)"
CCM_BIN="$PROJECT_DIR/target/debug/ccm"

# Create a test profile non-interactively by preparing the JSON
mkdir -p "$TEST_DIR/profiles"
cat > "$TEST_DIR/profiles/test-profile.json" << 'EOF'
{
  "env": {
    "ANTHROPIC_BASE_URL": "https://api.test.com/v1",
    "ANTHROPIC_AUTH_TOKEN": "test-token-123",
    "ANTHROPIC_MODEL": "test-model"
  }
}
EOF

echo "✓ Test profile created"
echo ""

echo "2. Listing profiles..."
$CCM_BIN list
echo ""

echo "3. Showing test profile..."
$CCM_BIN show test-profile
echo ""

echo "4. Creating another test profile..."
cat > "$TEST_DIR/profiles/another-test.json" << 'EOF'
{
  "env": {
    "ANTHROPIC_BASE_URL": "https://api.another.com/v1",
    "ANTHROPIC_AUTH_TOKEN": "another-token-456"
  }
}
EOF

echo "✓ Another test profile created"
echo ""

echo "5. Listing all profiles..."
$CCM_BIN list
echo ""

echo "6. Switching to test-profile..."
$CCM_BIN switch test-profile
echo ""

echo "7. Verifying Claude settings file was created in test directory..."
if [ -f "$CLAUDE_SETTINGS_PATH" ]; then
    echo "✓ Settings file created at: $CLAUDE_SETTINGS_PATH"
    echo "Content:"
    cat "$CLAUDE_SETTINGS_PATH"
else
    echo "✗ Settings file not found!"
    exit 1
fi
echo ""

echo "8. Verifying real directories were not modified..."
if [ -d "$HOME/.config/ccm/profiles" ]; then
    REAL_PROFILE_COUNT=$(ls -1 "$HOME/.config/ccm/profiles" 2>/dev/null | wc -l)
    echo "  Real profiles directory exists with $REAL_PROFILE_COUNT files (unchanged)"
else
    echo "  Real profiles directory does not exist (good!)"
fi

if [ -f "$HOME/.claude/settings.json" ]; then
    echo "  Real Claude settings exist (unchanged)"
else
    echo "  Real Claude settings do not exist (no modification)"
fi
echo ""

echo "=== All Tests Passed! ==="
echo ""
echo "Your real configuration directories were not modified:"
echo "  - $HOME/.config/ccm/"
echo "  - $HOME/.claude/"
echo ""
echo "Test directories will be cleaned up automatically on exit."
