use anyhow::Result;
use ccm::{
    cli::{Cli, Commands},
    profile, tui,
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
        Some(Commands::Import { name }) => profile::import_current_profile(name)?,
        Some(Commands::Rename { origin, new }) => profile::rename_profile(origin, new)?,
        Some(Commands::Edit { name }) => profile::edit_profile(name)?,
        Some(Commands::Ui) => match tui::launch_tui() {
            Ok(_) => {}
            Err(e) => {
                eprintln!("TUI mode failed: {}", e);
                eprintln!();
                eprintln!("🎮 Showing TUI demo instead...");
                if let Err(demo_err) = tui::demo_tui() {
                    eprintln!("Demo also failed: {}", demo_err);
                    eprintln!("Falling back to CLI mode. You can use these commands:");
                    eprintln!("  ccm list     - List all profiles");
                    eprintln!("  ccm switch X - Switch to profile X");
                    eprintln!("  ccm show X   - Show profile X details");
                    eprintln!("  ccm add X    - Add new profile X");
                    eprintln!("  ccm rm X     - Remove profile X");
                    eprintln!();
                    eprintln!("Use 'ccm --help' for all available commands.");
                }
            }
        },
        Some(Commands::TestTui) => {
            tui::test_tui_components()?;
        }
        Some(Commands::Sync) => {
            profile::sync_profile()?;
        }
        None => {
            // If no subcommand is provided, print help
            Cli::command().print_help()?;
        }
    }

    Ok(())
}
