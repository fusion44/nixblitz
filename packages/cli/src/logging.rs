use crate::{cli::Cli, config};
use log::LevelFilter;
use std::{path::PathBuf, str::FromStr};

/// The environment variable that can be used to override the log level
pub const LOG_LEVEL_ENV: &str = "NIXBLITZ_LOG_LEVEL";
/// The environment variable that can be used to override the log file path
pub const LOG_FILE_ENV: &str = "NIXBLITZ_LOG_FILE";
/// The default log level if not specified elsewhere
pub const DEFAULT_LOG_LEVEL: LevelFilter = LevelFilter::Info;

/// Get the log level from various sources in order of precedence:
/// 1. Environment variable NIXBLITZ_LOG_LEVEL
/// 2. Config file in config dir
/// 3. Default (Info)
pub fn get_log_level() -> LevelFilter {
    // First check environment variable
    if let Ok(level) = std::env::var(LOG_LEVEL_ENV) {
        if let Ok(level) = LevelFilter::from_str(&level) {
            return level;
        }
    }

    // TODO: Add config file support once we have a config format defined
    // For now, return default
    DEFAULT_LOG_LEVEL
}

/// Initialize logging for the CLI application
pub fn init_logging(cli: &Cli) {
    // Get log level from CLI args or fall back to config/env
    let log_level: LevelFilter = cli.log_level.unwrap_or_else(get_log_level);

    // Get log file from CLI args or fall back to env/default
    let log_file = if let Some(path) = &cli.log_file {
        PathBuf::from(path)
    } else {
        get_log_file()
    };

    // Create parent directories if they don't exist
    if let Some(parent) = log_file.parent() {
        std::fs::create_dir_all(parent).expect("Failed to create log directory");
    }

    let file_path = &log_file.display().to_string().clone();
    let _ = fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "{} [{}] {}",
                chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                record.level(),
                message
            ))
        })
        .level(log_level)
        .chain(fern::log_file(log_file).expect("Failed to open log file"))
        .apply();

    let init_time = chrono::Local::now().format("%Y-%m-%d %H:%M:%S");
    // Log initial message
    log::info!(
        ":
===========================================================================
Initialized logging with log file: {} and log level: {} at {}
===========================================================================",
        file_path,
        log_level,
        init_time,
    );
}

/// Get the log file path from environment or default location
fn get_log_file() -> PathBuf {
    if let Ok(path) = std::env::var(LOG_FILE_ENV) {
        PathBuf::from(path)
    } else {
        config::get_data_dir().join("nixblitz.log")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_log_level_env() {
        let original_log_level = std::env::var(LOG_LEVEL_ENV).ok();

        std::env::set_var(LOG_LEVEL_ENV, "debug");
        assert_eq!(get_log_level(), LevelFilter::Debug);

        std::env::set_var(LOG_LEVEL_ENV, "ERROR");
        assert_eq!(get_log_level(), LevelFilter::Error);

        if let Some(original) = original_log_level {
            std::env::set_var(LOG_LEVEL_ENV, original);
        } else {
            std::env::remove_var(LOG_LEVEL_ENV);
        }
    }
}
