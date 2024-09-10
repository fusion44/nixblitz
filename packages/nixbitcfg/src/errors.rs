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
    #[error("Unable to find the template file")]
    FileNotFound,
}

#[derive(Debug, Error)]
pub enum PathError {
    #[error("Unable to create the path")]
    CreatePathError,
    #[error("Unable to read the path")]
    PathUnreadable,
}
