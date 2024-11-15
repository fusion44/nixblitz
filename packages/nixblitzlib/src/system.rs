use std::path::PathBuf;

use error_stack::{Result, ResultExt};

use crate::{
    bitcoind::{self, BitcoinDaemonService},
    errors::SystemError,
    lnd::{self, LightningNetworkDaemonService},
    nix_base_config::{self, NixBaseConfig},
    utils::load_json_file,
};

/// Represents a system config that is stored at the [System::path].
#[derive(Default, Debug)]
pub struct System {
    /// The working directory we operate in
    work_dir: PathBuf,

    /// The nix base config
    nix_base: NixBaseConfig,

    /// The bitcoin daemon service
    bitcoin: BitcoinDaemonService,

    /// The lightning network daemon service
    lnd: LightningNetworkDaemonService,
}

impl System {
    pub fn load(work_dir: PathBuf) -> Result<Self, SystemError> {
        let nix_path = work_dir.join(nix_base_config::JSON_FILE_NAME);
        let nix_base_config_json =
            load_json_file(&nix_path).change_context(SystemError::SystemLoadError)?;
        let nix_base = NixBaseConfig::from_json(&nix_base_config_json)
            .change_context(SystemError::SystemLoadError)
            .attach_printable(format!(
                "Trying to load {}",
                nix_base_config::JSON_FILE_NAME
            ))?;

        let bitcoind_path = work_dir.join(bitcoind::JSON_FILE_NAME);
        let bitcoind_json =
            load_json_file(&bitcoind_path).change_context(SystemError::SystemLoadError)?;
        let bitcoin = BitcoinDaemonService::from_json(&bitcoind_json)
            .change_context(SystemError::SystemLoadError)
            .attach_printable(format!("Trying to load {}", bitcoind::JSON_FILE_NAME))?;

        let lnd_path = work_dir.join(lnd::JSON_FILE_NAME);
        let lnd_json = load_json_file(&lnd_path).change_context(SystemError::SystemLoadError)?;
        let lnd = LightningNetworkDaemonService::from_json(&lnd_json)
            .change_context(SystemError::SystemLoadError)
            .attach_printable(format!("Trying to load {}", lnd::JSON_FILE_NAME))?;

        Ok(Self {
            work_dir,
            nix_base,
            bitcoin,
            lnd,
        })
    }
}
