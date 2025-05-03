use std::path::Path;

use error_stack::{FutureExt, Result};
use nixblitzlib::utils::apply_changes;

use crate::errors::CliError;

pub async fn apply_changes_cmd(work_dir: &Path) -> Result<(), CliError> {
    let _ = apply_changes(work_dir)
        .change_context(CliError::Unknown)
        .await;
    Ok(())
}
