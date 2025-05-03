use clap::Parser;
use cli::Cli;
use commands::{
    apply::apply_changes_cmd, init::init_default_project_cmd, install::install_wizard,
    tui::start_tui,
};
use error_stack::Result;
use errors::CliError;

mod action;
mod app;
mod app_contexts;
mod cli;
mod colors;
mod commands;
mod components;
mod config;
mod constants;
mod errors;
mod logging;
mod pages;
mod tui;
mod utils;

#[tokio::main]
async fn main() -> Result<(), CliError> {
    let cli = Cli::parse();

    // Initialize logging with CLI args
    logging::init_logging(&cli);

    match &cli.command {
        Some(commands::Commands::Tui {
            tick_rate,
            frame_rate,
            work_dir,
        }) => start_tui(*tick_rate, *frame_rate, work_dir.clone()).await?,
        Some(commands::Commands::Init { work_dir, force }) => {
            init_default_project_cmd(work_dir, *force)?
        }
        Some(commands::Commands::Install { work_dir }) => install_wizard(work_dir)?,
        Some(commands::Commands::Doctor {}) => {
            println!("We haven't quite figured out how to implement this yet. Maybe try asking a magic 8-ball instead?")
        }
        Some(commands::Commands::Apply { work_dir }) => apply_changes_cmd(work_dir).await?,
        None => println!("Please use --help to find the available commands."),
    }

    Ok(())
}
