# Claude Config Manager (Rust)

A Rust CLI to manage multiple Claude Code configuration profiles with interactive prompts. Easily switch between different providers and models, and launch Claude with a specific configuration.

#![screenshot](./images/show.gif)

<!-- TOC -->
## Contents

- [Usage](#usage)
	- [Add a profile (interactive)](#add-a-profile-interactive)
	- [Add a profile with environment variables](#add-a-profile-with-environment-variables)
	- [List all profiles](#list-all-profiles)
	- [Import current Claude settings](#import-current-claude-settings)
	- [Show profile content](#show-profile-content)
	- [Switch to a profile](#switch-to-a-profile)
	- [Remove a profile](#remove-a-profile)
	- [Switch and run](#switch-and-run)
- [Claude Replacement Providers](#claude-replacement-providers)
	- [Deepseek](#deepseek)
	- [Kimi-k2-0905](#kimi-k2-0905)
	- [GLM-4.6](#glm-46)
	- [Minimax-M2](#minimax-m2)
- [Example workflow](#example-workflow)
- [Installation](#installation)
	- [Download pre-built binary](#download-pre-built-binary)
	- [Build from source](#build-from-source)
- [Defaults](#defaults)
<!-- /TOC -->

## Usage

**Note**: Commands support short aliases:
- `list` → `ls`
- `remove` → `rm`
- `switch` → `swc`

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

**Note**: Make sure the provider supports anthropic-compatible API.

### Add a profile with environment variables

```bash
ccm add my-profile --env FOO=bar --env BAZ=qux
```

### List all profiles

```bash
ccm ls
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
ccm swc openai-gpt4
```

This replaces your current `~/.claude/settings.json` with the selected profile and marks it as current.

### Remove a profile

```bash
ccm rm openai-gpt4
```

### Switch and run

```bash
ccm swc openai-gpt4
ccm run
```

First switch to a profile, then run Claude Code with that profile.


## Claude Replacement Providers

You can choose from various Claude replacement providers that support the Anthropic-compatible API.

### Deepseek

- [Deepseek Claude Code integration guide](https://api-docs.deepseek.com/guides/anthropic_api): You can learn more about using Deepseek with Claude Code.

### Kimi-k2-0905

- [Moonshot platform](https://platform.moonshot.ai/docs/guide/agent-support#using-kimi-k2-model-in-claude-code): You can learn more about using Kimi K2 model in Claude Code on Moonshot platform.
- [Kimi coding plan](https://www.kimi.com/coding/docs/): And its latest coding plan.

### GLM-4.6

- [z.ai](https://docs.z.ai/devpack/tool/claude): for most of the users.
- [智普官网](https://docs.bigmodel.cn/cn/coding-plan/tool/claude#%E6%AD%A5%E9%AA%A4%E4%BA%8C%EF%BC%9A%E9%85%8D%E7%BD%AE-glm-coding-plan): for Chinese users. (invatation code: KXZP8FZANR)

### Minimax-M2

- [Minimax-M2 Claude Code integration guide](https://platform.minimaxi.com/docs/guides/text-ai-coding-tools#%E5%9C%A8-claude-code-%E4%B8%AD%E4%BD%BF%E7%94%A8-minimax-m2%EF%BC%88%E6%8E%A8%E8%8D%90%EF%BC%89)


## Example workflow

```bash
# Create profiles for different providers
ccm add openai
# (answer prompts: ANTHROPIC_BASE_URL, ANTHROPIC_AUTH_TOKEN, etc.)

ccm add anthropic --env CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC=1
# (answer prompts)

# List available profiles
ccm ls

# Switch between them
ccm swc openai

# Run Claude Code
ccm run
```

## Installation

### Download pre-built binary

You can download pre-built binaries from GitHub Releases:

- https://github.com/caibirdme/ccm/releases

Steps (Linux/macOS):

```bash
# 1) Download the archive for your OS/arch from the Releases page
# 2) Extract it, then make it executable and move into your PATH
chmod +x ccm && sudo mv ccm /usr/local/bin/

# Or install to user-local bin
chmod +x ccm && mkdir -p "$HOME/.local/bin" && mv ccm "$HOME/.local/bin/"
```

Then run:

```bash
ccm --version
```

### Build from source

Prerequisites: Rust toolchain (rustup), Cargo.

```bash
cargo install --path .
```

Verify:

```bash
ccm --version
```



## Defaults

- Claude settings path: `$HOME/.claude/settings.json`
- Profiles directory: `$XDG_CONFIG_HOME/ccm/profiles` (falls back to `$HOME/.config/ccm/profiles`)
- Current profile tracking: `$XDG_CONFIG_HOME/ccm/current`
- Override settings path via `CLAUDE_SETTINGS_PATH` environment variable
