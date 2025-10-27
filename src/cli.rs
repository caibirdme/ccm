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
    #[command(alias = "ls")]
    List,
    /// Show profile content
    Show { name: String },
    /// Remove a profile
    #[command(alias = "rm")]
    Remove { name: String },
    /// Switch current Claude settings to a profile
    #[command(alias = "swc")]
    Switch { name: String },
    /// Run Claude Code with the current profile (use 'switch' first to select a profile)
    Run,
    /// Import current Claude settings as a new profile
    ImportCurrent { name: String },
}
