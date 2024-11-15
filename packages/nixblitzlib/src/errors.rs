use thiserror::Error;

#[derive(Debug, Error)]
pub enum PasswordError {
    #[error("Password to short")]
    ToShort,
    #[error("Unable to hash the password")]
    HashingError,
    #[error("Password is None")]
    IsNone,
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
    #[error("Unable to generate the project files")]
    GenFilesError,
    #[error("Unable to read the project files")]
    ParseError,
    #[error("No project files where found in the given directory")]
    NoProjectFound,
    #[error("No project files where found in the given directory")]
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
}
