use thiserror::Error;

#[derive(Debug, Error)]
pub enum CliError {
    #[error("Unknown error")]
    Unknown,
    #[error("Unable to start the GUI")]
    UnableToStartGui,
    #[error("Unable to initialize the system")]
    UnableToInitializeSystem,
}
