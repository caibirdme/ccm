use anyhow::Result;
use ccm::{
    cli::{Cli, Commands},
    profile,
};
use clap::Parser;

fn main() -> Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Add { name, env } => {
            profile::add_profile_interactive(name, env)?;
        }
        Commands::List => profile::list_profiles()?,
        Commands::Show { name } => profile::show_profile(name)?,
        Commands::Remove { name } => profile::remove_profile(name)?,
        Commands::Switch { name } => profile::switch_to_profile(name)?,
        Commands::Launch => profile::launch_claude_code()?,
        Commands::ImportCurrent { name } => profile::import_current_profile(name)?,
    }

    Ok(())
}
