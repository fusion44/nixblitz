use std::path::PathBuf;

use error_stack::Result;

use crate::{errors::CliError, tui::start_tui_app};

pub async fn start_tui(tick_rate: f64, frame_rate: f64, work_dir: PathBuf) -> Result<(), CliError> {
    start_tui_app(tick_rate, frame_rate, work_dir).await
}
