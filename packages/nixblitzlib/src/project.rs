use std::{path::PathBuf, rc::Rc};

use error_stack::{Result, ResultExt};

use crate::{
    app_config::AppConfig,
    app_option_data::option_data::{OptionData, OptionDataChangeNotification, OptionId},
    apps::SupportedApps,
    bitcoind::{self, BitcoinDaemonService},
    errors::ProjectError,
    lnd::{self, LightningNetworkDaemonService},
    nix_base_config::{self, NixBaseConfig},
    utils::load_json_file,
};

/// Represents a system config that is stored at the [System::path].
#[derive(Default, Debug)]
pub struct Project {
    /// The working directory we operate in
    work_dir: PathBuf,

    /// The currently selected app
    selected_app: SupportedApps,

    /// The nix base config
    pub nix_base: NixBaseConfig,

    /// The bitcoin daemon service
    bitcoin: BitcoinDaemonService,

    /// The lightning network daemon service
    lnd: LightningNetworkDaemonService,
}

impl Project {
    /// Loads the project configuration from the specified working directory.
    ///
    /// This function initializes a `Project` instance by loading configuration
    /// files for the supported applications from the given directory.
    /// It constructs the necessary components and sets the initial selected
    /// application to NixOS.
    ///
    /// # Parameters
    ///
    /// - `work_dir`: The path to the working directory containing the configuration files.
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing:
    /// - `Ok(Project)` if the configuration is successfully loaded.
    /// - `Err(ProjectError)` if an error occurs during the loading process.
    ///
    /// # Errors
    ///
    /// This function will return an error if any of the configuration files
    /// cannot be loaded or parsed correctly.
    pub fn load(work_dir: PathBuf) -> Result<Self, ProjectError> {
        let nix_path = work_dir.join(nix_base_config::JSON_FILE_NAME);
        let nix_base_config_json =
            load_json_file(&nix_path).change_context(ProjectError::ProjectLoadError)?;
        let nix_base = NixBaseConfig::from_json(&nix_base_config_json)
            .change_context(ProjectError::ProjectLoadError)
            .attach_printable(format!(
                "Trying to load {}",
                nix_base_config::JSON_FILE_NAME
            ))?;

        let bitcoind_path = work_dir.join(bitcoind::JSON_FILE_NAME);
        let bitcoind_json =
            load_json_file(&bitcoind_path).change_context(ProjectError::ProjectLoadError)?;
        let bitcoin = BitcoinDaemonService::from_json(&bitcoind_json)
            .change_context(ProjectError::ProjectLoadError)
            .attach_printable(format!("Trying to load {}", bitcoind::JSON_FILE_NAME))?;

        let lnd_path = work_dir.join(lnd::JSON_FILE_NAME);
        let lnd_json = load_json_file(&lnd_path).change_context(ProjectError::ProjectLoadError)?;
        let lnd = LightningNetworkDaemonService::from_json(&lnd_json)
            .change_context(ProjectError::ProjectLoadError)
            .attach_printable(format!("Trying to load {}", lnd::JSON_FILE_NAME))?;

        Ok(Self {
            selected_app: SupportedApps::NixOS,
            work_dir,
            nix_base,
            bitcoin,
            lnd,
        })
    }

    /// Retrieves the application options for the currently selected app.
    ///
    /// This function returns a reference-counted vector of `OptionData` for the
    /// application that is currently selected. The options provide configuration
    /// details specific to the app.
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing:
    /// - `Ok(Rc<Vec<OptionData>>)` with the options for the selected app.
    /// - `Err(ProjectError)` if an error occurs while retrieving the options.
    ///
    /// # Errors
    ///
    /// This function will return an error if the options cannot be retrieved
    /// for the specified application.
    pub fn get_app_options(&mut self) -> Result<Rc<Vec<OptionData>>, ProjectError> {
        match self.selected_app {
            SupportedApps::NixOS => Ok(Rc::new(self.nix_base.get_options())),
            SupportedApps::BitcoinCore => todo!(),
            SupportedApps::CoreLightning => todo!(),
            SupportedApps::LND => todo!(),
            SupportedApps::BlitzAPI => todo!(),
            SupportedApps::WebUI => todo!(),
        }
    }

    /// Handles changes to application options.
    ///
    /// This function is called when an option's value is changed. It determines
    /// the application associated with the option and delegates the change handling
    /// to the appropriate component.
    ///
    /// # Parameters
    ///
    /// - `option`: The notification containing the details of the option change.
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing a boolean value:
    /// - `Ok(true)` if the option value has changed.
    /// - `Ok(false)` if the option value has not changed.
    /// - `Err(ProjectError)` if an error occurs during the process.
    ///
    /// # Errors
    ///
    /// This function will return an error if the option change cannot be processed
    /// for the specified application.
    pub fn on_option_changed(
        &mut self,
        option: OptionDataChangeNotification,
    ) -> Result<bool, ProjectError> {
        // TODO: figure out how to implement this with a trait.
        let id: &OptionId = match &option {
            OptionDataChangeNotification::Bool(value) => &value.id,
            OptionDataChangeNotification::StringList(value) => &value.id,
            OptionDataChangeNotification::TextEdit(value) => &value.id,
            OptionDataChangeNotification::PasswordEdit(value) => &value.id,
        };

        match id.app {
            SupportedApps::NixOS => self.nix_base.app_option_changed(id, &option),
            SupportedApps::BitcoinCore => todo!(),
            SupportedApps::CoreLightning => todo!(),
            SupportedApps::LND => todo!(),
            SupportedApps::BlitzAPI => todo!(),
            SupportedApps::WebUI => todo!(),
        }
    }
}
