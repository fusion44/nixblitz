use std::path::Path;

use error_stack::{Result, ResultExt};
use log::info;
use nixblitzlib::{
    app_option_data::{
        option_data::{OptionDataChangeNotification, ToOptionId},
        text_edit_data::TextOptionChangeData,
    },
    apps::SupportedApps,
    nix_base_config::NixBaseConfigOption,
    project::Project,
};

use crate::errors::CliError;

use super::SupportedAppsValueEnum;

/// Sets a specific configuration option for a given application.
///
/// # Arguments
/// * `work_dir`: The working directory of the project.
/// * `app`: The application (`SupportedAppsValueEnum`) whose option is to be set.
/// * `option`: The string identifier of the option to set (e.g., "disko_device").
/// * `value`: The new string value for the option.
///
/// # Errors
/// Returns `CliError` if loading the project fails, if the app/option combination
/// is unsupported, or if setting the option within the project logic fails.
pub fn set_option_value(
    work_dir: &Path,
    app_value_enum: &SupportedAppsValueEnum,
    option_name: &str,
    value_str: &str,
) -> Result<(), CliError> {
    info!(
        "Attempting to set option '{}' of app {} to '{}'",
        option_name, app_value_enum, value_str
    );

    match (app_value_enum.inner(), option_name) {
        (SupportedApps::NixOS, "disko_device") => {
            let mut project = Project::load(work_dir.to_path_buf())
                .change_context(CliError::UnableToInitProjectStruct)?;
            project.set_selected_app(SupportedApps::NixOS);
            let change_notification =
                OptionDataChangeNotification::TextEdit(TextOptionChangeData::new(
                    NixBaseConfigOption::DiskoDevice.to_option_id(),
                    value_str.to_string(),
                ));

            project
                .on_option_changed(change_notification)
                .change_context(CliError::OptionSetError(
                    option_name.to_string(),
                    app_value_enum.to_string(),
                    value_str.to_string(),
                ))?;

            info!(
                "Successfully set option '{}' of app {} to '{}'",
                option_name, app_value_enum, value_str
            );
            Ok(())
        }
        _ => Err(CliError::OptionUnsupportedError(
            app_value_enum.to_string(),
            option_name.to_string(),
        )
        .into()),
    }
}

#[cfg(test)]
mod tests {
    use crate::{commands::set::set_option_value, errors::CliError};
    use nixblitzlib::{apps::SupportedApps, utils::init_default_project};
    use std::fs;

    // TODO: these are more like integration tests than unit tests
    //       clean this up and move them to actual integration tests
    #[test]
    fn test_set_option_nixos_disko_device_success() {
        let temp_dir = tempfile::tempdir().unwrap();
        init_default_project(temp_dir.path(), None).unwrap();
        let file_contents =
            fs::read_to_string(temp_dir.path().join("src/configuration.common.nix")).unwrap();
        assert!(file_contents.contains("disko.devices.disk.main.device = \"\";"));

        let result = set_option_value(
            temp_dir.path(),
            &crate::commands::SupportedAppsValueEnum::from_base(SupportedApps::NixOS),
            "disko_device",
            "test",
        );
        assert!(result.is_ok());

        let file_contents =
            fs::read_to_string(temp_dir.path().join("src/configuration.common.nix")).unwrap();
        assert!(file_contents.contains("disko.devices.disk.main.device = \"test\";"));
    }

    #[test]
    fn test_set_unsupported_option() {
        let result = set_option_value(
            tempfile::tempdir().unwrap().path(),
            &crate::commands::SupportedAppsValueEnum::from_base(SupportedApps::LND),
            "disko_device",
            "test",
        );
        assert!(result.is_err());
        let err = result.unwrap_err();

        assert_eq!(
            *err.current_context(),
            CliError::OptionUnsupportedError("lnd".into(), "disko_device".into())
        );
    }
}
