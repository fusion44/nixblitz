use std::panic;

use cli_log::error;
use error_stack::Report;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CliError {
    #[error("Unknown error")]
    Unknown,
    #[error("Unable to start the TUI")]
    UnableToStartTui,
    #[error("Unable to initialize the Project struct")]
    UnableToInitProjectStruct,
    #[error("Unable to draw a component")]
    UnableToDrawComponent,
    #[error("Unable to find a unbounded sender")]
    UnableToFindUnboundedSender,
    #[error("Unable to send via the unbounded sender")]
    UnableToSendViaUnboundedSender,
    #[error("Length of display name to long")]
    MaxDisplayNameLengthReached,
    #[error("More than one modal not allowed")]
    MultipleModalsOpened,
    #[error("Invalid argument provided")]
    ArgumentError,
    #[error("JSON parse error")]
    JsonParseError,
    #[error("Unable to find option {}", .0)]
    OptionRetrievalError(String),
    #[error("Unable to find title for option {}", .0)]
    OptionTitleRetrievalError(String),
    #[error("Unable to find string for id {}", .0)]
    StringRetrievalError(String),
    #[error("Wrong option type was provided. What we are looking for: {}, What we are: {}", .0, .1)]
    OptionTypeMismatch(String, String),
    #[error("{}", .0 )]
    StringParseError(String),
}

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
