use clap::Parser;
use log::LevelFilter;

use crate::commands::Commands;

const PKG_VERSION: &str = env!("CARGO_PKG_VERSION");
const GIT_SHA: &str = match option_env!("VERGEN_GIT_SHA") {
    Some(sha) => sha,
    None => "sha-unknown",
};
const GIT_COMMIT_DATE: &str = match option_env!("VERGEN_GIT_COMMIT_DATE") {
    Some(date) => date,
    None => "date-unknown",
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

pub fn version() -> String {
    let author = clap::crate_authors!();

    let version_message = format!("{}-{} ({})", PKG_VERSION, GIT_SHA, GIT_COMMIT_DATE);

    format!(
        "\
{version_message}
Authors: {author}"
    )
}
