use std::process::ExitStatus;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum ArgumentError {
    #[error("Wrong argument provided: {}, expected: {} ", .0, .1)]
    InvalidArgument(String, String),
}

/// Represents errors that can occur when starting the process.
#[derive(Debug, Error)]
pub enum CommandError {
    /// Failed to spawn the command (e.g., command not found, permissions).
    #[error("Failed to spawn command: {}", .0)]
    SpawnFailed(String),
    /// Failed to acquire stdout/stderr pipes.
    #[error("Failed to get command pipe: {}", .0)]
    PipeError(String),
    /// Error for when a command spawns successfully but exits with a non-zero
    /// status code, indicating that it failed during execution.
    #[error("Command '{command}' failed with status {status}")]
    ExecutionFailed { command: String, status: ExitStatus },
    /// A git error occurred. Use attached_printable to add other information.
    #[error("A git error occurred while running '{}'", .0)]
    GitError(String),
}

#[derive(Debug, Error)]
pub enum PasswordError {
    #[error("Password too short")]
    TooShort,
    #[error("Unable to hash the password")]
    HashingError,
    #[error("Password is None")]
    IsNone,
    #[error("Confirm password must not be None")]
    MissingConfirm,
    #[error("Passwords do not match")]
    Mismatch,
}

#[derive(Debug, Error)]
pub enum TemplatingError {
    #[error("Unable to register the template file")]
    Register,
    #[error("Unable to render the template file")]
    Render,
    #[error("Unable to format the rendered template")]
    Format,
    #[error("Unable to find the template file {:?}", .0)]
    FileNotFound(String),
    #[error("Unable to generate the json string")]
    JsonRenderError,
    #[error("Unable to load the json string")]
    JsonLoadError,
}

#[derive(Debug, Error)]
pub enum ProjectError {
    #[error("Unable to change the option value for {:?}", .0)]
    ChangeOptionValueError(String),
    #[error("Unable to get options from project")]
    GetOptionsError,
    #[error("Unable to generate the project files")]
    GenFilesError,
    #[error("Unable to read the project files")]
    ParseError,
    #[error("No project files where found in the given directory")]
    NoProjectFound,
    #[error("Unable to load the project")]
    ProjectLoadError,
    #[error("Unable to create the path: {:?}", .0)]
    CreatePathError(String),
    #[error("Unable to create the base project")]
    CreateBaseFiles(String),
    #[error("Unable to find the file for path {:?}", .0)]
    FileNotFound(String),
    #[error("Unable to open the file at path {:?}", .0)]
    FileOpenError(String),
    #[error("Unable to read the file contents at path {:?}", .0)]
    FileReadError(String),
    #[error("Invalid data type. Got {:?} Expected {:?}", .0, .1)]
    InvalidDataType(String, String),
    #[error("Unable to apply changes")]
    ApplyChangesError(),
}

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("Unable to parse address: {:?} ", .0)]
    AddrParseError(String),
    #[error("Unable to parse string '{}'", .0)]
    StringParseError(String),
}

#[derive(Debug, Error)]
pub enum StringErrors {
    #[error("Unable to retrieve string: {:?} ", .0)]
    StringRetrievalError(String),
}

#[derive(Debug, Error)]
pub enum SystemErrors {
    #[error("Unable to retrieve system info: {:?} ", .0)]
    GatherSystemInfoError(String),
}

#[derive(Debug, Error)]
pub enum InstallError {
    #[error("{}", .0)]
    IoError(String),
    #[error("Error while installing the system: {}", .0)]
    InstallExecutionFailed(String),
    #[error("Error while running command: {}\nError: {}", .0,.1 )]
    CommandError(String, String),
    #[error("Lock operation error: {}", .0)]
    LockError(String),
    #[error("Error while copying the config")]
    CopyConfigError,
}
