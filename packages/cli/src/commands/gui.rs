use std::path::PathBuf;

use error_stack::{Report, Result, ResultExt};
use nixblitzlib::system::System;

use crate::{app::App, errors::CliError};

pub async fn start_gui(
    tick_rate: f64,
    frame_rate: f64,
    path: PathBuf,
    init: bool,
) -> Result<(), CliError> {
    let sys = System::new(path, init);
    let res = sys.init().change_context(CliError::Unknown)?;

    let app = App::new(tick_rate, frame_rate);
    let _ = app
        .expect("Unable to create the TUI app;")
        .run()
        .await
        .change_context(CliError::UnableToStartGui);

    Ok(())
}
