use crate::errors::StringErrors;

pub trait GetStringOrCliError {
    fn get_or_err(&self) -> Result<&str, StringErrors>;
}
