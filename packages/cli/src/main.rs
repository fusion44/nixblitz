use clap::Parser;
use cli::Cli;
use cli_log::init_cli_log;
use commands::{gui::start_gui, init::init_default_system_cmd};
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
mod pages;
mod tui;

#[tokio::main]
async fn main() -> Result<(), CliError> {
    init_cli_log!();

    let cli = Cli::parse();
    match &cli.command {
        Some(commands::Commands::Gui {
            tick_rate,
            frame_rate,
            work_dir,
        }) => start_gui(*tick_rate, *frame_rate, work_dir.clone()).await?,
        Some(commands::Commands::Init { work_dir, force }) => {
            init_default_system_cmd(work_dir, *force)?
        }
        Some(commands::Commands::Doctor {}) => {
            println!("We haven't quite figured out how to implement this yet. Maybe try asking a magic 8-ball instead?")
        }
        None => println!("Please use --help to find the available commands."),
    }

    Ok(())
}
