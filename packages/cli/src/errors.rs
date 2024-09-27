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
}
