#!/bin/bash
# Test script for the switch confirmation feature
# This script tests the scenario where settings.json differs from the current profile

set -e

# Setup test directories
TEST_DIR="/tmp/ccm-switch-test-$$"
export CCM_CONFIG_DIR="$TEST_DIR/ccm"
export CLAUDE_SETTINGS_PATH="$TEST_DIR/claude/settings.json"

echo "Setting up test environment in: $TEST_DIR"
mkdir -p "$TEST_DIR/ccm/profiles"
mkdir -p "$TEST_DIR/claude"

# Build the project
echo "Building ccm..."
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(cd "$SCRIPT_DIR/../.." && pwd)"
cd "$PROJECT_DIR"
cargo build --quiet

CCM="$PROJECT_DIR/target/debug/ccm"

echo ""
echo "=== Test 1: Create two profiles ==="
# Create profile 'bar'
echo "Creating profile 'bar'..."
cat > "$CCM_CONFIG_DIR/profiles/bar.json" << 'EOF'
{
  "env": {
    "ANTHROPIC_BASE_URL": "https://api.bar.com/v1",
    "ANTHROPIC_AUTH_TOKEN": "sk-bar-token",
    "ANTHROPIC_MODEL": "gpt-4"
  }
}
EOF

# Create profile 'foo'
echo "Creating profile 'foo'..."
cat > "$CCM_CONFIG_DIR/profiles/foo.json" << 'EOF'
{
  "env": {
    "ANTHROPIC_BASE_URL": "https://api.foo.com/v1",
    "ANTHROPIC_AUTH_TOKEN": "sk-foo-token",
    "ANTHROPIC_MODEL": "claude-3"
  }
}
EOF

echo ""
echo "=== Test 2: Switch to 'bar' profile ==="
echo "bar" > "$CCM_CONFIG_DIR/current"
cp "$CCM_CONFIG_DIR/profiles/bar.json" "$CLAUDE_SETTINGS_PATH"
$CCM list

echo ""
echo "=== Test 3: Modify settings.json (simulate user editing in Claude) ==="
cat > "$CLAUDE_SETTINGS_PATH" << 'EOF'
{
  "env": {
    "ANTHROPIC_BASE_URL": "https://api.bar.com/v1",
    "ANTHROPIC_AUTH_TOKEN": "sk-bar-token-MODIFIED",
    "ANTHROPIC_MODEL": "gpt-4-turbo",
    "API_TIMEOUT_MS": "60000"
  }
}
EOF

echo "Settings.json modified to differ from profile 'bar'"
echo ""
echo "Current profile (bar.json):"
cat "$CCM_CONFIG_DIR/profiles/bar.json"
echo ""
echo "Current settings.json:"
cat "$CLAUDE_SETTINGS_PATH"

echo ""
echo "=== Test 4: Attempt to switch to 'foo' ==="
echo "This should trigger the confirmation prompt..."
echo ""
echo "Note: The test will show the prompt. In a real scenario, you would choose:"
echo "  1 - Switch directly"
echo "  2 - Update bar profile with current settings, then switch"
echo "  3 - Cancel"
echo ""

# For automated testing, we'll simulate choosing option 3 (cancel)
echo "Simulating choice 3 (cancel)..."
echo "3" | $CCM switch foo || true

echo ""
echo "=== Test 5: Verify that switch was cancelled ==="
current_profile=$(cat "$CCM_CONFIG_DIR/current" 2>/dev/null || echo "none")
echo "Current profile is still: $current_profile"

if [ "$current_profile" = "bar" ]; then
    echo "✓ Test passed: Switch was cancelled as expected"
else
    echo "✗ Test failed: Current profile should still be 'bar'"
fi

echo ""
echo "=== Test 6: Test switch with option 1 (direct switch) ==="
echo "1" | $CCM switch foo

current_profile=$(cat "$CCM_CONFIG_DIR/current" 2>/dev/null || echo "none")
if [ "$current_profile" = "foo" ]; then
    echo "✓ Test passed: Switched to 'foo'"
else
    echo "✗ Test failed: Should have switched to 'foo'"
fi

echo ""
echo "=== Test 7: Test switch with option 2 (update then switch) ==="
# Switch back to bar
cp "$CCM_CONFIG_DIR/profiles/bar.json" "$CLAUDE_SETTINGS_PATH"
echo "bar" > "$CCM_CONFIG_DIR/current"

# Modify settings again
cat > "$CLAUDE_SETTINGS_PATH" << 'EOF'
{
  "env": {
    "ANTHROPIC_BASE_URL": "https://api.bar.com/v2",
    "ANTHROPIC_AUTH_TOKEN": "sk-new-token"
  }
}
EOF

echo "Testing option 2 (update profile then switch)..."
echo "2" | $CCM switch foo

# Check if bar.json was updated
echo "Checking if bar.json was updated..."
if grep -q "sk-new-token" "$CCM_CONFIG_DIR/profiles/bar.json"; then
    echo "✓ Test passed: Profile 'bar' was updated with settings.json content"
else
    echo "✗ Test failed: Profile 'bar' should have been updated"
fi

# Cleanup
echo ""
echo "=== Cleanup ==="
rm -rf "$TEST_DIR"
echo "Test environment cleaned up"
echo ""
echo "All tests completed!"
