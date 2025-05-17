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
}

/// Represents errors that can occur when interacting with the git repository.
#[derive(Debug, Error)]
pub enum GitError {
    #[error("Unable to initialize git repository in directory: {}", .0)]
    InitError(String),
    #[error("Unable to add file to git repository in directory: {}", .0)]
    AddFileError(String),
    #[error("Unable to commit changes to git repository in directory: {}", .0)]
    CommitError(String),
    #[error("Unable to push changes to git repository in directory: {}", .0)]
    PushError(String),
    #[error("Unable to pull changes from git repository in directory: {}", .0)]
    PullError(String),
    #[error("Unable to checkout a branch from git repository in directory: {}", .0)]
    CheckoutError(String),
    #[error("Unable to fetch changes from git repository in directory: {}", .0)]
    FetchError(String),
    #[error("The given workpath is invalid: {}", .0)]
    InvalidPath(String),
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
