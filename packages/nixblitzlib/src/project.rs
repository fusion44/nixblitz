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

    pub fn on_option_changed(
        &mut self,
        option: OptionDataChangeNotification,
    ) -> Result<bool, ProjectError> {
        // TODO: figure out how to implement this with a trait.
        let id: &OptionId = match &option {
            OptionDataChangeNotification::Bool(value) => &value.id,
            OptionDataChangeNotification::StringList(value) => &value.id,
            OptionDataChangeNotification::TextEdit(value) => &value.id,
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
