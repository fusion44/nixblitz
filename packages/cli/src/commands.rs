use std::path::PathBuf;

use clap::Subcommand;

pub mod gui;

#[derive(Debug, Subcommand)]
pub enum Commands {
    Gui {
        /// Tick rate, i.e. number of ticks per second
        #[arg(short, long, value_name = "FLOAT", default_value_t = 4.0)]
        tick_rate: f64,

        /// Frame rate, i.e. number of frames per second
        #[arg(short, long, value_name = "FLOAT", default_value_t = 60.0)]
        frame_rate: f64,

        /// The path to the system files
        #[arg(short, long, value_name = "PATH", default_value = ".")]
        path: PathBuf,

        /// Whether to initialize a new system at the given path
        #[arg(short, long)]
        init: bool,
    },
}
