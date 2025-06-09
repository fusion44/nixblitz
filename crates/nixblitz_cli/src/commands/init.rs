use std::path::Path;

use error_stack::{Result, ResultExt};
use nixblitz_system::utils::init_default_project;

use crate::errors::CliError;

pub fn init_default_project_cmd(work_dir: &Path, force: bool) -> Result<(), CliError> {
    init_default_project(work_dir, Some(force))
        .change_context(CliError::UnableToInitProjectStruct)?;
    Ok(())
}
