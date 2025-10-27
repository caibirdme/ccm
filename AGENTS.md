# Claude Config Manager (ccm) - AI Agent Documentation

## Project Overview

**Claude Config Manager (ccm)** is a Rust CLI tool that manages multiple Claude Code configuration profiles. It enables users to switch between different AI providers that support Anthropic-compatible APIs and launch Claude Code with specific configurations.

*Note: The README.md documentation mentions several providers (OpenAI, Deepseek, Kimi, GLM, Minimax) as known examples of compatible services, but this tool works with any provider offering Anthropic-compatible API endpoints.*

## Architecture

### Core Components

1. **CLI Interface** (`src/cli.rs`)
   - Uses `clap` derive macros for command parsing
   - 7 main commands: `add`, `list`, `show`, `remove`, `switch`, `run`, `import-current`
   - Supports environment variable injection via `--env` flags

2. **Profile Management** (`src/profile.rs`)
   - Interactive profile creation with validation
   - JSON-based profile storage with environment variables
   - Safety mechanisms (prevents removal of active profiles)
   - Current profile tracking

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

### Supported Environment Variables
- `ANTHROPIC_BASE_URL` (required) - API endpoint URL
- `ANTHROPIC_AUTH_TOKEN` (required) - Authentication token
- `ANTHROPIC_MODEL` (optional) - Model selection
- `API_TIMEOUT_MS` (optional) - Request timeout
- `ANTHROPIC_SMALL_FAST_MODEL` (optional) - Fast model alternative
- `CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC` (optional) - Traffic control

## File Storage

### Default Paths (XDG-compliant)
- **Profiles Directory**: `~/.config/ccm/profiles/*.json`
- **Current Profile**: `~/.config/ccm/current`
- **Claude Settings**: `~/.claude/settings.json`

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
```

## Extension Points

### Adding New Commands
1. Add command variant to `Commands` enum in `cli.rs`
2. Implement handler function in `profile.rs`
3. Add command routing in `main.rs`

### Adding New Environment Variables
1. Update interactive prompts in `add_profile_interactive()`
2. Add validation logic as needed
3. Update documentation

### Documenting Compatible Providers
Since this tool works with any Anthropic-compatible API provider, new providers can be documented in README.md as they are discovered and tested. Provider compatibility is determined by their API endpoint compatibility, not by any specific code changes to this tool.

## Development Guidelines

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
