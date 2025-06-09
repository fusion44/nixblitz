use std::path::PathBuf;

use error_stack::{Result, ResultExt};
use log::error;

use crate::{app::App, errors::CliError};

pub async fn start_tui(tick_rate: f64, frame_rate: f64, work_dir: PathBuf) -> Result<(), CliError> {
    let absolute_work_dir =
        work_dir
            .canonicalize()
            .change_context(CliError::UnableCanonicalizeWorkDir(
                "Unable to resolve work dir".to_string(),
            ))?;
    let app = App::new(tick_rate, frame_rate, absolute_work_dir);
    let res = app.expect("Unable to create the TUI app;").run().await;

    if let Err(report) = res {
        error!("{report:?}");
        return Err(report.change_context(CliError::UnableToStartTui));
    }

    Ok(())
}
