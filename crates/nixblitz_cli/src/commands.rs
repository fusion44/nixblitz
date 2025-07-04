use std::fmt;
use std::path::PathBuf;

use clap::{Subcommand, ValueEnum, builder::PossibleValue};
use nixblitz_core::{apps::SupportedApps, constants::NIXBLITZ_WORK_DIR_ENV};

use crate::define_clap_apps_value_enum;

pub mod apply;
pub mod init;
pub mod install;
pub mod set;
pub mod tui;

define_clap_apps_value_enum!(
    SupportedApps,
    SupportedAppsValueEnum,
    [NixOS, BitcoinCore, CoreLightning, LND, BlitzAPI, WebUI]
);

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Sets an app option in the given configuration
    Set {
        /// The working directory to operate on
        #[arg(short, long, value_name = "PATH", env = NIXBLITZ_WORK_DIR_ENV)]
        work_dir: PathBuf,

        /// The application to set the option for
        app: SupportedAppsValueEnum,

        /// The option to set. Note, not all possible options are implemented, yet.
        option: String,

        /// The value to set the option to
        value: String,
    },
    /// Opens the TUI in the given work dir
    Tui {
        /// Tick rate, i.e. number of ticks per second
        #[arg(short, long, value_name = "FLOAT", default_value_t = 4.0)]
        tick_rate: f64,

        /// Frame rate, i.e. number of frames per second
        #[arg(short, long, value_name = "FLOAT", default_value_t = 60.0)]
        frame_rate: f64,

        /// The working directory to operate on
        #[arg(short, long, value_name = "PATH", env = NIXBLITZ_WORK_DIR_ENV)]
        work_dir: PathBuf,
    },
    /// Opens a TUI with the crate iocraft
    Tui2 {
        /// Tick rate, i.e. number of ticks per second
        #[arg(short, long, value_name = "FLOAT", default_value_t = 4.0)]
        tick_rate: f64,

        /// Frame rate, i.e. number of frames per second
        #[arg(short, long, value_name = "FLOAT", default_value_t = 60.0)]
        frame_rate: f64,

        /// The working directory to operate on
        #[arg(short, long, value_name = "PATH", env = NIXBLITZ_WORK_DIR_ENV)]
        work_dir: PathBuf,
    },
    /// Initializes a new project in the given work dir
    Init {
        /// The working directory to operate on
        #[arg(short, long, value_name = "PATH", env = NIXBLITZ_WORK_DIR_ENV)]
        work_dir: PathBuf,

        /// Whether to force overwrite existing files
        #[arg(short, long)]
        force: bool,
    },
    /// Installs the system defined in the given work dir
    Install {
        /// The working directory to operate on
        #[arg(short, long, value_name = "PATH", env = NIXBLITZ_WORK_DIR_ENV)]
        work_dir: PathBuf,
    },
    /// Applies changes to the system defined in the given work dir
    Apply {
        /// The working directory to operate on
        #[arg(short, long, value_name = "PATH", env = NIXBLITZ_WORK_DIR_ENV)]
        work_dir: PathBuf,
    },
    /// Analyze the project for common problems
    Doctor {},
}
