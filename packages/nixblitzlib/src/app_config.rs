use crate::{
    app_option_data::option_data::{OptionData, OptionDataChangeNotification},
    errors::ProjectError,
};
use error_stack::Result;

pub trait AppConfig: Debug {
    fn app_option_changed(
        &mut self,
        option: &OptionDataChangeNotification,
    ) -> Result<bool, ProjectError>;

    fn get_options(&self) -> Vec<OptionData>;
}
