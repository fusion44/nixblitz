use std::{cell::RefCell, path::PathBuf, rc::Rc};

use error_stack::{Result, ResultExt};

use crate::{
    app_config::AppConfig,
    app_option_data::option_data::{OptionData, OptionDataChangeNotification},
    apps::SupportedApps,
    bitcoind::{self, BitcoinDaemonService},
    blitz_api::{self, BlitzApiService},
    blitz_webui::{self, BlitzWebUiService},
    cln::{self, CoreLightningService},
    errors::ProjectError,
    lnd::{self, LightningNetworkDaemonService},
    nix_base_config::{self, NixBaseConfig},
    utils::load_json_file,
};

/// Represents a system config that is stored at :Wathe [System::path].
#[derive(Debug)]
pub struct Project {
    /// The working directory we operate in
    work_dir: PathBuf,

    /// The currently selected app
    selected_app: Box<Rc<RefCell<dyn AppConfig>>>,

    /// The nix base config
    nix_base: Rc<RefCell<NixBaseConfig>>,

    /// The bitcoin daemon service
    bitcoin: Rc<RefCell<BitcoinDaemonService>>,

    /// The Core Lightning service
    cln: Rc<RefCell<CoreLightningService>>,

    /// The lightning network daemon service
    lnd: Rc<RefCell<LightningNetworkDaemonService>>,

    /// Blitz API service
    blitz_api: Rc<RefCell<BlitzApiService>>,

    /// Blitz Web UI service
    blitz_webui: Rc<RefCell<BlitzWebUiService>>,
}

impl Project {
    /// Sets the currently selected application.
    ///
    /// This function updates the `selected_app` field of the `Project` struct
    /// to the specified application.
    ///
    /// # Parameters
    ///
    /// - `app`: The application to be set as the currently selected app.
    pub fn set_selected_app(&mut self, app: SupportedApps) {
        self.selected_app = match app {
            SupportedApps::NixOS => Box::new(self.nix_base.clone()),
            SupportedApps::BitcoinCore => Box::new(self.bitcoin.clone()),
            SupportedApps::CoreLightning => Box::new(self.cln.clone()),
            SupportedApps::LND => Box::new(self.lnd.clone()),
            SupportedApps::BlitzAPI => Box::new(self.blitz_api.clone()),
            SupportedApps::WebUI => Box::new(self.blitz_webui.clone()),
        };
    }

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
        let nix_base = Rc::new(RefCell::new(nix_base));

        let bitcoind_path = work_dir.join(bitcoind::JSON_FILE_NAME);
        let bitcoind_json =
            load_json_file(&bitcoind_path).change_context(ProjectError::ProjectLoadError)?;
        let bitcoin = BitcoinDaemonService::from_json(&bitcoind_json)
            .change_context(ProjectError::ProjectLoadError)
            .attach_printable(format!("Trying to load {}", bitcoind::JSON_FILE_NAME))?;
        let bitcoin = Rc::new(RefCell::new(bitcoin));

        let cln_path = work_dir.join(cln::JSON_FILE_NAME);
        let cln_json = load_json_file(&cln_path).change_context(ProjectError::ProjectLoadError)?;
        let cln = CoreLightningService::from_json(&cln_json)
            .change_context(ProjectError::ProjectLoadError)
            .attach_printable(format!("Trying to load {}", cln::JSON_FILE_NAME))?;
        let cln = Rc::new(RefCell::new(cln));

        let lnd_path = work_dir.join(lnd::JSON_FILE_NAME);
        let lnd_json = load_json_file(&lnd_path).change_context(ProjectError::ProjectLoadError)?;
        let lnd = LightningNetworkDaemonService::from_json(&lnd_json)
            .change_context(ProjectError::ProjectLoadError)
            .attach_printable(format!("Trying to load {}", lnd::JSON_FILE_NAME))?;
        let lnd = Rc::new(RefCell::new(lnd));

        let blitz_api_path = work_dir.join(blitz_api::JSON_FILE_NAME);
        let blitz_api_json =
            load_json_file(&blitz_api_path).change_context(ProjectError::ProjectLoadError)?;
        let blitz_api = BlitzApiService::from_json(&blitz_api_json)
            .change_context(ProjectError::ProjectLoadError)
            .attach_printable(format!("Trying to load {}", blitz_api::JSON_FILE_NAME))?;
        let blitz_api = Rc::new(RefCell::new(blitz_api));

        let blitz_webui_path = work_dir.join(blitz_webui::JSON_FILE_NAME);
        let blitz_webui_json =
            load_json_file(&blitz_webui_path).change_context(ProjectError::ProjectLoadError)?;
        let blitz_webui = BlitzWebUiService::from_json(&blitz_webui_json)
            .change_context(ProjectError::ProjectLoadError)
            .attach_printable(format!("Trying to load {}", blitz_webui::JSON_FILE_NAME))?;
        let blitz_webui = Rc::new(RefCell::new(blitz_webui));

        Ok(Self {
            selected_app: Box::new(nix_base.clone()),
            work_dir,
            nix_base,
            bitcoin,
            cln,
            lnd,
            blitz_api,
            blitz_webui,
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
        Ok(Rc::new(
            self.selected_app.clone().borrow_mut().get_options(),
        ))
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
        let res = self.selected_app.borrow_mut().app_option_changed(&option)?;
        };

        Ok(res)
    }
}
