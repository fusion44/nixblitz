use crate::{
    app_option_data::option_data::{OptionData, OptionDataChangeNotification, OptionId},
    errors::ProjectError,
};
use error_stack::Result;

pub trait AppConfig {
    fn app_option_changed(
        &mut self,
        id: &OptionId,
        option: &OptionDataChangeNotification,
    ) -> Result<bool, ProjectError> {
        // to prevent unused variable warnings in from Clippy
        let _ = (id, option);
        todo!()
    }

    fn get_options(&self) -> Vec<OptionData> {
        todo!();
    }
}
