use nixblitzlib::strings::{Strings, STRINGS};

use crate::errors::CliError;

pub trait GetStringOrCliError {
    fn get_or_err(&self) -> Result<&str, CliError>;
}

impl GetStringOrCliError for Strings {
    fn get_or_err(&self) -> Result<&str, CliError> {
        match self {
            Strings::PasswordInputPlaceholderMain => Ok(STRINGS
                .get(self)
                .ok_or(CliError::StringRetrievalError(self.to_string()))?),
            Strings::PasswordInputPlaceholderConfirm => Ok(STRINGS
                .get(self)
                .ok_or(CliError::StringRetrievalError(self.to_string()))?),
        }
    }
}
