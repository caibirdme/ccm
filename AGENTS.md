# Claude Config Manager (ccm) - Documentation for AI Agents

## Project Overview

**Claude Config Manager (ccm)** is a CLI tool written in Rust that manages multiple Claude Code configuration profiles. It enables users to switch between different AI providers that support Anthropic-compatible APIs and launch Claude Code with specific configurations.

*Note: The README.md documentation mentions several providers (OpenAI, Deepseek, Kimi, GLM, Minimax) as known examples of compatible services, but this tool works with any provider offering Anthropic-compatible API endpoints.*

## Architecture

### Core Components

1. **CLI Interface** (`src/cli.rs`)
   - Uses `clap` derive macros for command parsing
   - 8 main commands: `add`, `list`/`ls`, `show`, `remove`/`rm`, `switch`/`swc`, `run`, `import-current`, `sync`
   - Supports environment variable injection via `--env` flags

2. **Profile Management** (`src/profile.rs`)
   - Interactive profile creation with validation
   - JSON-based profile storage with environment variables
   - Safety mechanisms (prevents removal of active profiles)
   - Current profile tracking
   - **Sync functionality** - compares and synchronizes current profile with active Claude settings

3. **Configuration** (`src/config.rs`)
   - XDG-compliant directory structure
   - Configurable Claude settings path via `CLAUDE_SETTINGS_PATH`
   - Automatic directory creation and path resolution

### File Structure
```
src/
├── main.rs      # Entry point and command routing
├── lib.rs       # Library root
├── cli.rs       # Command-line interface definitions
├── config.rs    # Path and configuration management
└── profile.rs   # Core profile functionality
```

## Key Features

### Profile Management
- **Interactive Creation**: Prompts for required and optional environment variables
- **Environment Variables**: Support for custom variables via `--env KEY=VALUE`
- **Validation**: Input validation and error handling
- **Security**: Prevents accidental deletion of active profiles

### Sync Feature
- **Automatic Synchronization**: `ccm sync` command compares current profile with actual `settings.json`
- **Bidirectional Update**: Updates ccm profile to match Claude settings when they diverge
- **Change Detection**: Uses JSON comparison to detect changes accurately

### Supported Environment Variables
- `ANTHROPIC_BASE_URL` (required) - API endpoint URL
- `ANTHROPIC_AUTH_TOKEN` (required) - Authentication token
- `ANTHROPIC_MODEL` (optional) - Model selection
- `API_TIMEOUT_MS` (optional) - Request timeout
- `ANTHROPIC_SMALL_FAST_MODEL` (optional) - Fast model alternative
- `CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC` (optional int) - Traffic control

## File Storage

### Default Paths (XDG-compliant)
- **Profiles Directory**: `~/.config/ccm/profiles/*.json`
- **Current Profile**: `~/.config/ccm/current`
- **Claude Settings**: `~/.claude/settings.json`

### Environment Variable Overrides
For testing or custom configurations, these directories can be overridden:
- **`CCM_CONFIG_DIR`**: Override the base directory for CCM configuration (default: `~/.config/ccm`)
  - Profiles will be stored in `$CCM_CONFIG_DIR/profiles/`
  - Current profile marker will be at `$CCM_CONFIG_DIR/current`
- **`CLAUDE_SETTINGS_PATH`**: Override the Claude settings file path (default: `~/.claude/settings.json`)

### Profile Format
```json
{
  "env": {
    "ANTHROPIC_BASE_URL": "https://api.example.com/v1",
    "ANTHROPIC_AUTH_TOKEN": "sk-...",
    "ANTHROPIC_MODEL": "gpt-4",
    "API_TIMEOUT_MS": "300000"
  }
}
```

## Dependencies

### Core Dependencies
- `clap` (4.3) - Command-line parsing with derive macros
- `serde` (1.0) - Serialization framework
- `serde_json` (1.0) - JSON handling
- `dirs` (6.0) - Cross-platform directory access
- `anyhow` (1.0) - Error handling

## Build System

### Development Tools
- **Rust Toolchain**: 1.90.0
- **Task Runner**: `just` for build automation
- **CI/CD**: GitHub Actions with multi-platform builds

### Build System
DO NOT run build command by your own.This project uses [`just`](https://just.systems/) as the task runner for build automation. See the `justfile` for available commands.

## Common Usage Patterns

### Adding Profiles
```bash
# Interactive creation
ccm add my-profile

# With environment variables
ccm add my-profile --env CUSTOM_VAR=value
```

### Managing Profiles
```bash
# List all profiles
ccm list

# Switch to profile
ccm switch my-profile

# Show profile content
ccm show my-profile

# Remove profile
ccm remove my-profile
```

### Launching Claude
```bash
# Switch and run
ccm switch my-profile && ccm run

# Import current settings
ccm import-current backup-profile

# Sync current profile with Claude settings
ccm sync
```

## Extension Points

### Adding New Commands
1. Add command variant to `Commands` enum in `cli.rs`
2. Implement handler function in `profile.rs`
3. Add command routing in `main.rs`
4. Update CLI interface with any aliases in the clap derive macros

### Adding New Environment Variables
1. Update interactive prompts in `add_profile_interactive()`
2. Add validation logic as needed
3. Update documentation

### Documenting Compatible Providers
Since this tool works with any Anthropic-compatible API provider, new providers can be documented in README.md as they are discovered and tested. Provider compatibility is determined by their API endpoint compatibility, not by any specific code changes to this tool.

## Development Guidelines

### Testing Environment Safety
**CRITICAL**: When testing this tool, you MUST NOT modify real user configuration directories:

- **NEVER modify**: `$HOME/.claude/` or `$HOME/.config/ccm/` during testing
- **ALWAYS use**: Test directories in `/tmp` for all testing activities

To use test directories, set these environment variables before running tests:
```bash
# Example: Safe testing configuration
export CCM_CONFIG_DIR=/tmp/ccm-test
export CLAUDE_SETTINGS_PATH=/tmp/claude-test/settings.json

# Now all ccm commands will use test directories
ccm add test-profile
ccm list
```

This ensures that:
- Real user profiles in `~/.config/ccm/profiles/` are not affected
- Real Claude settings in `~/.claude/settings.json` are not modified
- Test data is isolated in `/tmp` and can be safely deleted
- Multiple test sessions can run concurrently with different temp directories

### Third-Party Library Usage Policy
**MANDATORY**: Before using ANY third-party library, you MUST:

1. **Research First**: Use MCP tools to learn the library's API and best practices
   - Use suitable mcp tools for library-specific examples and patterns
   - Search for recent examples in the Rust ecosystem

2. **Version Requirements**: Always use the latest semantic versioning (semver) compatible version
   - Check crates.io for the most recent version
   - Update version numbers in `Cargo.toml` accordingly
   - Ensure version constraints allow for patch and minor updates (e.g., `1.0` not `=1.0.0`)

3. **Examples**:
   ```bash
   # Before adding 'tokio' to dependencies
   mcp__exa__get_code_context_exa query="Rust tokio async runtime tutorial examples" tokensNum=3000

   # Before adding 'serde' functionality
   mcp__exa__get_code_context_exa query="Rust serde JSON serialization best practices" tokensNum=3000
   ```

### Code Style
- Follow Rust standard conventions
- Use `anyhow` for error handling
- Implement proper context for error messages
- Use XDG directory standards

### Performance
- Minimal dependencies for fast compilation
- Efficient file operations
- No unnecessary allocations

### User Experience
- Clear error messages with actionable suggestions
- Interactive prompts with helpful defaults
- Progress indicators for long operations
- Safety confirmations for destructive operations
