use crate::{
    app_option_data::option_data::{OptionData, OptionDataChangeNotification},
    errors::ProjectError,
};
use error_stack::Result;
use std::{fmt::Debug, path::Path};

pub trait AppConfig: Debug {
    fn app_option_changed(
        &mut self,
        option: &OptionDataChangeNotification,
    ) -> Result<bool, ProjectError>;

    fn get_options(&self) -> Vec<OptionData>;

    fn save(&mut self, work_dir: &Path) -> Result<(), ProjectError>;
}
