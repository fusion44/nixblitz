use thiserror::Error;

#[derive(Debug, Error)]
pub enum GenerateDefaultSystemError {
    #[error("")]
    ToShort,
}

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
    #[error("Unable to find the template file")]
    FileNotFound,
    #[error("Unable to generate the json string")]
    JsonRenderError,
}

#[derive(Debug, Error)]
pub enum SystemError {
    #[error("Unable to generate the system files")]
    GenFilesError,
    #[error("Unable to read the system files")]
    ParseError,
    #[error("No system files where found in the given directory")]
    NoSystemFound,
    #[error("Unable to create the path: {:?}", .0)]
    CreatePathError(String),
    #[error("Unable to crate the base system")]
    CreateBaseFiles(String),
}
