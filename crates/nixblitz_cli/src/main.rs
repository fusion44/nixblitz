use clap::Parser;
use cli::Cli;
use commands::{
    apply::apply_changes_cmd, init::init_default_project_cmd, install::install_wizard,
    set::set_option_value, tui::start_tui,
};
use error_stack::Result;
use errors::{CliError, init_error_handlers};

mod cli;
mod commands;
mod errors;
mod logging;
pub mod macros;
mod tui;
mod tui_components;

#[tokio::main]
async fn main() -> Result<(), CliError> {
    let cli = Cli::parse();

    // Initialize logging with CLI args
    logging::init_logging(&cli);
    init_error_handlers();

    match &cli.command {
        Some(commands::Commands::Tui {
            work_dir,
            create_project,
        }) => start_tui(work_dir.clone(), create_project).await?,
        Some(commands::Commands::Init { work_dir, force }) => {
            init_default_project_cmd(work_dir, *force)?
        }
        Some(commands::Commands::Install { work_dir }) => install_wizard(work_dir)?,
        Some(commands::Commands::Doctor {}) => {
            println!(
                "We haven't quite figured out how to implement this yet. Maybe try asking a magic 8-ball instead?"
            )
        }
        Some(commands::Commands::Apply { work_dir }) => apply_changes_cmd(work_dir).await?,
        Some(commands::Commands::Set {
            work_dir,
            app,
            option,
            value,
        }) => set_option_value(work_dir, app, option, value)?,
        None => println!("Please use --help to find the available commands."),
    }

    Ok(())
}
