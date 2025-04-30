use crate::cli::Cli;

use log::LevelFilter;
use std::{path::PathBuf, str::FromStr};

/// The environment variable that can be used to override the log level
pub const LOG_LEVEL_ENV: &str = "NIXBLITZ_LOG_LEVEL";
/// The environment variable that can be used to override the log file path
pub const LOG_FILE_ENV: &str = "NIXBLITZ_LOG_FILE";
/// The default log level if not specified elsewhere
pub const DEFAULT_LOG_LEVEL: LevelFilter = LevelFilter::Info;
const LOG_SUBDIR: &str = "nixblitz-cli";
const LOG_FILENAME: &str = "nixblitz.log";

/// Get the log level from various sources in order of precedence:
/// 1. Environment variable NIXBLITZ_LOG_LEVEL
/// 2. Config file in config dir (TODO)
/// 3. Default (Info)
pub fn get_log_level() -> LevelFilter {
    if let Ok(level_str) = std::env::var(LOG_LEVEL_ENV) {
        if let Ok(level) = LevelFilter::from_str(&level_str.to_lowercase()) {
            return level;
        } else {
            eprintln!(
                "Warning: Invalid log level '{}' in {}, using default.",
                level_str, LOG_LEVEL_ENV
            );
        }
    }

    // TODO: Add config file support once we have a config format defined
    // For now, return default
    DEFAULT_LOG_LEVEL
}

/// Initialize logging for the CLI application
pub fn init_logging(cli: &Cli) {
    // Get log level from CLI args or fall back to config/env/default
    let log_level: LevelFilter = cli.log_level.unwrap_or_else(get_log_level);

    // Determine log file path: CLI arg > Environment Var > Default Path
    let log_file = cli
        .log_file
        .clone()
        .map(PathBuf::from)
        .or_else(|| std::env::var(LOG_FILE_ENV).ok().map(PathBuf::from))
        .unwrap_or_else(get_default_log_file);

    // Create parent directories if they don't exist
    if let Some(parent) = log_file.parent() {
        if !parent.exists() {
            std::fs::create_dir_all(parent)
                .unwrap_or_else(|_| panic!("Failed to create log directory: {:?}", parent));
        }
    }

    let log_file_display = log_file.display().to_string();
    let fern_logger = fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "{} [{:<5}] {}",
                chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                record.level(),
                message
            ))
        })
        .level(log_level)
        .chain(fern::log_file(&log_file).expect("Failed to open log file for writing"))
        .apply();

    if let Err(e) = fern_logger {
        eprintln!("Error initializing logger: {}", e);
        panic!("Failed to initialize logger");
    }

    let init_time = chrono::Local::now().format("%Y-%m-%d %H:%M:%S");
    log::info!(
        "\n===========================================================================\n\
        Initialized logging\n  Level: {}\n  File:  {}\n  Time:  {}\n\
        ===========================================================================",
        log_level,
        log_file_display,
        init_time,
    );
}

fn get_default_log_file() -> PathBuf {
    let state_dir = dirs::state_dir();
    match state_dir {
        Some(dir) => dir.join(LOG_SUBDIR).join(LOG_FILENAME),
        None => {
            panic!("Could not determine the user's state directory (e.g., ~/.local/state). Ensure $XDG_STATE_HOME environment variable is set or pass `--log-file`");
        }
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

        std::env::set_var(LOG_LEVEL_ENV, "WaRn");
        assert_eq!(get_log_level(), LevelFilter::Warn);

        std::env::set_var(LOG_LEVEL_ENV, "invalid");
        assert_eq!(get_log_level(), DEFAULT_LOG_LEVEL);

        if let Some(original) = original_log_level {
            std::env::set_var(LOG_LEVEL_ENV, original);
        } else {
            std::env::remove_var(LOG_LEVEL_ENV);
        }
    }

    #[test]
    fn test_get_default_log_file_path() {
        let default_path = get_default_log_file();

        assert!(default_path.ends_with(PathBuf::from(LOG_SUBDIR).join(LOG_FILENAME)));

        assert!(default_path.is_absolute());

        if let Some(state_dir) = dirs::state_dir() {
            assert_eq!(default_path.parent().unwrap(), state_dir.join(LOG_SUBDIR));
        } else {
            panic!("dirs::state_dir() returned None in test environment.");
        }
    }

    #[test]
    fn test_log_file_override_priority() {
        let original_env_var = std::env::var(LOG_FILE_ENV).ok();
        let default_path = get_default_log_file();
        let env_path = PathBuf::from("/tmp/nixblitz-env.log");
        let cli_path_str = "/tmp/nixblitz-cli.log";
        let cli_path = PathBuf::from(cli_path_str);

        // default only
        std::env::remove_var(LOG_FILE_ENV);
        let cli_args_1 = Cli {
            command: None,
            log_file: None,
            log_level: None,
        };
        let path_1 = cli_args_1
            .log_file
            .clone()
            .map(PathBuf::from)
            .or_else(|| std::env::var(LOG_FILE_ENV).ok().map(PathBuf::from))
            .unwrap_or_else(get_default_log_file);
        assert_eq!(path_1, default_path);

        // env var and no CLI arg
        std::env::set_var(LOG_FILE_ENV, env_path.to_str().unwrap());
        let cli_args_2 = Cli {
            command: None,
            log_file: None,
            log_level: None,
        };
        let path_2 = cli_args_2
            .log_file
            .clone()
            .map(PathBuf::from)
            .or_else(|| std::env::var(LOG_FILE_ENV).ok().map(PathBuf::from))
            .unwrap_or_else(get_default_log_file);
        assert_eq!(path_2, env_path);

        // both, env var and CLI arg set (CLI should win)
        std::env::set_var(LOG_FILE_ENV, env_path.to_str().unwrap());
        let cli_args_3 = Cli {
            command: None,
            log_file: Some(cli_path_str.to_string()),
            log_level: None,
        };
        let path_3 = cli_args_3
            .log_file
            .clone()
            .map(PathBuf::from)
            .or_else(|| std::env::var(LOG_FILE_ENV).ok().map(PathBuf::from))
            .unwrap_or_else(get_default_log_file);
        assert_eq!(path_3, cli_path);

        // only CLI arg set
        std::env::remove_var(LOG_FILE_ENV);
        let cli_args_4 = Cli {
            command: None,
            log_file: Some(cli_path_str.to_string()),
            log_level: None,
        };
        let path_4 = cli_args_4
            .log_file
            .clone()
            .map(PathBuf::from)
            .or_else(|| std::env::var(LOG_FILE_ENV).ok().map(PathBuf::from))
            .unwrap_or_else(get_default_log_file);
        assert_eq!(path_4, cli_path);

        if let Some(original) = original_env_var {
            std::env::set_var(LOG_FILE_ENV, original);
        } else {
            std::env::remove_var(LOG_FILE_ENV);
        }
    }
}
