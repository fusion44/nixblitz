use std::panic;

use cli_log::error;
use error_stack::Report;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CliError {
    #[error("Unknown error")]
    Unknown,
    #[error("Unable to start the GUI")]
    UnableToStartGui,
    #[error("Unable to initialize the System struct")]
    UnableToInitSystemStruct,
    #[error("Unable to draw a component")]
    UnableToDrawComponent,
    #[error("Length of display name to long")]
    MaxDisplayNameLengthReached,
    #[error("Negative number of modals reached (below 0)")]
    NumModalComponentNegative,
    #[error("Maximum number of modals reached (Max: u8::MAX)")]
    MaxModalComponentReached,
pub fn init_error_handlers() {
    panic::set_hook(Box::new(move |panic_info| {
        let message = panic_info.to_string();
        let location = panic_info
            .location()
            .map(|loc| {
                format!(
                    "Panic occurred in file '{}' at line {}",
                    loc.file(),
                    loc.line()
                )
            })
            .unwrap_or_else(|| "Unknown location".to_string());

        let report = Report::new(CliError::Unknown)
            .attach_printable(message)
            .attach_printable(location);

        error!("Panic occurred: {:?}", report);
    }));
}
