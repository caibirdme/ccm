# Claude Config Manager (Rust)

A Rust CLI to manage multiple Claude Code configuration profiles with interactive prompts. Easily switch between different providers and models, and launch Claude with a specific configuration.

## Usage

### Add a profile (interactive)

```bash
ccm add openai-gpt4
```

You'll be prompted for:
- ANTHROPIC_BASE_URL
- ANTHROPIC_AUTH_TOKEN
- ANTHROPIC_MODEL (optional)
- API_TIMEOUT_MS (optional)
- ANTHROPIC_SMALL_FAST_MODEL (optional)
- CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC (optional int)

### Add a profile with environment variables

```bash
ccm add my-profile --env FOO=bar --env BAZ=qux
```

### List all profiles

```bash
ccm list
```

Output:
```
- openai-gpt5
- self-deploy-kimi-k2 (current)
- some-router-glm
- some-router-claude-sonnet-45
```

The current active profile will be marked with `(current)`.

### Import current Claude settings

```bash
ccm import-current my-backup
```

This saves your current `~/.claude/settings.json` as a new profile.

### Show profile content

```bash
ccm show openai-gpt5
```

Output:
```
ANTHROPIC_BASE_URL=https://api.openai.com/v1
ANTHROPIC_AUTH_TOKEN=sk-********
ANTHROPIC_MODEL=gpt-5
API_TIMEOUT_MS=300000
ANTHROPIC_SMALL_FAST_MODEL=gpt-5-mini
```

### Switch to a profile

```bash
ccm switch openai-gpt4
```

This replaces your current `~/.claude/settings.json` with the selected profile and marks it as current.

### Remove a profile

```bash
ccm remove openai-gpt4
```

### Switch and launch

```bash
ccm switch openai-gpt4
ccm launch
```

First switch to a profile, then launch Claude Code with that profile.

## Example workflow

```bash
# Create profiles for different providers
ccm add openai
# (answer prompts: ANTHROPIC_BASE_URL, ANTHROPIC_AUTH_TOKEN, etc.)

ccm add anthropic --env CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC=1
# (answer prompts)

# List available profiles
ccm list

# Switch between them
ccm switch openai

# Launch Claude Code
ccm launch
```

## Installation

```bash
cargo install --path .
```

## Defaults

- Claude settings path: `$HOME/.claude/settings.json`
- Profiles directory: `$XDG_CONFIG_HOME/ccm/profiles` (falls back to `$HOME/.config/ccm/profiles`)
- Current profile tracking: `$XDG_CONFIG_HOME/ccm/current`
- Override settings path via `CLAUDE_SETTINGS_PATH` environment variable
