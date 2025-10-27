use anyhow::Result;
use ccm::{
    cli::{Cli, Commands},
    profile,
};
use clap::{CommandFactory, Parser};

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
        Some(Commands::Run) => profile::launch_claude_code()?,
        Some(Commands::ImportCurrent { name }) => profile::import_current_profile(name)?,
        Some(Commands::Rename { origin, new }) => profile::rename_profile(origin, new)?,
        Some(Commands::Edit { name }) => profile::edit_profile(name)?,
        None => {
            // If no subcommand is provided, print help
            Cli::command().print_help()?;
        }
    }

    Ok(())
}
