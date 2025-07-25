use std::path::PathBuf;

use error_stack::Result;

use crate::{errors::CliError, tui::start_tui_app};

pub async fn start_tui(work_dir: PathBuf, create_project: &bool) -> Result<(), CliError> {
    start_tui_app(work_dir, create_project).await
}
