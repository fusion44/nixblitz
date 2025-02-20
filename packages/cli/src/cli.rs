use clap::Parser;
use log::LevelFilter;

use crate::{
    commands::Commands,
    config::{get_config_dir, get_data_dir},
};

#[derive(Parser, Debug)]
#[command(author, version = version(), about)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,

    /// Set the log level (overrides environment variable and config file)
    #[arg(long, value_enum, global = true)]
    pub log_level: Option<LevelFilter>,

    /// Set the log file path (overrides environment variable and default)
    #[arg(long, global = true)]
    pub log_file: Option<String>,
}

const VERSION_MESSAGE: &str = concat!(
    env!("CARGO_PKG_VERSION"),
    "-",
    env!("VERGEN_GIT_DESCRIBE"),
    " (",
    env!("VERGEN_BUILD_DATE"),
    ")"
);

pub fn version() -> String {
    let author = clap::crate_authors!();

    let config_dir_path = get_config_dir().display().to_string();
    let data_dir_path = get_data_dir().display().to_string();

    format!(
        "\
{VERSION_MESSAGE}

Authors: {author}

Config directory: {config_dir_path}
Data directory: {data_dir_path}"
    )
}
