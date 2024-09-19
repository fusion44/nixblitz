use std::path::PathBuf;

use error_stack::{Report, Result};

use crate::errors::SystemError;

/// Represents a system config that is stored at the [System::path].
#[derive(Default, Debug)]
pub struct System {
    /// The working directory we operate in
    work_dir: PathBuf,
}

impl System {
    pub fn new(path: PathBuf) -> Self {
        Self { work_dir: path }
    }

    /// Initializes the system based on the configuration found in the given path.
    ///
    /// This function checks if the specified path exists and is a directory. If the directory is empty,
    /// it will initialize a new system if init was set to true. Otherwise, it returns an error.
    ///
    /// # Errors
    ///
    /// * [SystemError::ParseError] - If the path is invalid, doesn't exist, or is not a directory.
    /// * [SystemError::NoSystemFound] - If the directory is empty and init is false.
    pub fn init(&self) -> Result<(), SystemError> {
        if self.work_dir.exists() {
            let items = self.work_dir.read_dir();
            let mut items = match items {
                Ok(items) => items,
                Err(e) => {
                    return Err(Report::new(SystemError::ParseError)
                        .attach_printable(format!("Error reading items in the directory: {}", e)))
                }
            };

            let item = items.next();
            if item.is_none() {
                return Err(Report::new(SystemError::ParseError).attach_printable(
                    "Given folder is empty, use the init command first to initialize a new system",
                ));
            }

            return Ok(());
        }

        Err(Report::new(SystemError::ParseError).attach_printable(
            "Given folder is empty, use the init command first to initialize a new system",
        ))
    }
}
