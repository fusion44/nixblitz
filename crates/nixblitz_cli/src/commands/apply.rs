use std::path::Path;

use error_stack::{FutureExt, Result};
use log::{error, info};
use nixblitz_system::utils::apply_changes;

use crate::errors::CliError;

pub async fn apply_changes_cmd(work_dir: &Path) -> Result<(), CliError> {
    let res = apply_changes(work_dir)
        .change_context(CliError::Unknown)
        .await;
    match res {
        Ok(()) => {
            println!("Changes applied successfully");
            info!("Changes applied successfully");
        }
        Err(e) => {
            eprintln!("Error applying changes: {:?}", e);
            error!("Error applying changes: {:?}", e);
        }
    }

    Ok(())
}
