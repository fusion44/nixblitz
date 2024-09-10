use clap::Parser;
use cli::Cli;
use commands::gui::start_gui;
use error_stack::Result;
use errors::CliError;

mod action;
mod app;
mod cli;
mod commands;
mod components;
mod config;
mod constants;
mod errors;
mod pages;
mod tui;

#[tokio::main]
async fn main() -> Result<(), CliError> {
    let cli = Cli::parse();
    match &cli.command {
        Some(commands::Commands::Gui {
            tick_rate,
            frame_rate,
        }) => start_gui(*tick_rate, *frame_rate).await?,
        None => {}
    }

    Ok(())
}
