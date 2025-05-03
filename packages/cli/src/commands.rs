use std::path::PathBuf;

use clap::Subcommand;

pub mod apply;
pub mod init;
pub mod install;
pub mod tui;

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Opens the TUI in the given work dir
    Tui {
        /// Tick rate, i.e. number of ticks per second
        #[arg(short, long, value_name = "FLOAT", default_value_t = 4.0)]
        tick_rate: f64,

        /// Frame rate, i.e. number of frames per second
        #[arg(short, long, value_name = "FLOAT", default_value_t = 60.0)]
        frame_rate: f64,

        /// The working directory to operate on
        #[arg(short, long, value_name = "PATH", default_value = ".")]
        work_dir: PathBuf,
    },
    /// Initializes a new project in the given work dir
    Init {
        /// The working directory to operate on
        #[arg(short, long, value_name = "PATH", default_value = ".")]
        work_dir: PathBuf,

        /// Whether to force overwrite existing files
        #[arg(short, long)]
        force: bool,
    },
    /// Installs the system defined in the given work dir
    Install {
        /// The working directory to operate on
        #[arg(short, long, value_name = "PATH", default_value = ".")]
        work_dir: PathBuf,
    },
    /// Applies changes to the system defined in the given work dir
    Apply {
        /// The working directory to operate on
        #[arg(short, long, value_name = "PATH", default_value = ".")]
        work_dir: PathBuf,
    },
    /// Analyze the project for common problems
    Doctor {},
}
