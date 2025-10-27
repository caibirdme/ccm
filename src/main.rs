use anyhow::Result;
use ccm::{
    cli::{Cli, Commands},
    profile,
};
use clap::{Parser, CommandFactory};

fn main() -> Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Some(Commands::Add { name, env }) => {
            profile::add_profile_interactive(name, env)?;
        }
        Some(Commands::List) => profile::list_profiles()?,
        Some(Commands::Show { name }) => profile::show_profile(name)?,
        Some(Commands::Remove { name }) => profile::remove_profile(name)?,
        Some(Commands::Switch { name }) => profile::switch_to_profile(name)?,
        Some(Commands::Launch) => profile::launch_claude_code()?,
        Some(Commands::ImportCurrent { name }) => profile::import_current_profile(name)?,
        None => {
            // If no subcommand is provided, print help
            Cli::command().print_help()?;
        }
    }

    Ok(())
}
