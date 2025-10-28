use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "ccm", version, about = "Manage multiple Claude Code configurations (profiles) and switch/launch", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Add a profile with interactive prompts for ANTHROPIC_BASE_URL, ANTHROPIC_AUTH_TOKEN, ANTHROPIC_MODEL, API_TIMEOUT_MS, ANTHROPIC_SMALL_FAST_MODEL, and CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC. Use --env for additional environment variables.
    Add {
        /// Profile name
        name: String,
        /// Additional environment variables (can be used multiple times: --env KEY=VALUE)
        #[arg(long)]
        env: Vec<String>,
    },
    /// List saved profiles (shows current active profile)
    #[command(visible_alias = "ls")]
    List,
    /// Show profile content
    Show { name: String },
    /// Remove a profile
    #[command(visible_alias = "rm")]
    Remove { name: String },
    /// Switch current Claude settings to a profile
    #[command(visible_alias = "swc")]
    Switch { name: String },
    /// Run Claude Code with the current profile (use 'switch' first to select a profile)
    Run,
    /// Import current Claude settings as a new profile
    Import { name: String },
    /// Rename a profile from original name to new name
    Rename {
        /// Original profile name
        origin: String,
        /// New profile name
        new: String,
    },
    /// Edit a profile using the default editor (opens profile JSON file in editor)
    Edit { name: String },
    /// Launch terminal UI mode (interactive profile management)
    #[command(visible_alias = "tui")]
    Ui,
    /// Test TUI components without requiring a terminal
    TestTui,
}
