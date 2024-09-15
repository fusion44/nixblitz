use std::path::Path;

use error_stack::{Result, ResultExt};
use nixblitzlib::utils::init_default_system;

use crate::errors::CliError;

pub fn init_default_system_cmd(work_dir: &Path, force: bool) -> Result<(), CliError> {
    init_default_system(work_dir, Some(force))
        .change_context(CliError::UnableToInitSystemStruct)?;
    Ok(())
}
