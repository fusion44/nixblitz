use std::path::PathBuf;

use cli_log::error;
use error_stack::{Result, ResultExt};

use crate::{app::App, errors::CliError};

pub async fn start_gui(tick_rate: f64, frame_rate: f64, work_dir: PathBuf) -> Result<(), CliError> {
    let app = App::new(tick_rate, frame_rate, work_dir);
    let res = app.expect("Unable to create the TUI app;").run().await;

    if let Err(report) = res {
        error!("{report:?}");
        return Err(report.change_context(CliError::UnableToStartGui));
    }

    Ok(())
}
