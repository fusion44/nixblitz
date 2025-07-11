use std::{collections::HashMap, net::IpAddr, path::Path, str::FromStr};

use alejandra::format;
use error_stack::{Report, Result, ResultExt};
use handlebars::{Handlebars, no_escape};
use serde::{Deserialize, Serialize};

use nixblitz_core::{
    app_config::AppConfig,
    app_option_data::{
        bool_data::BoolOptionData,
        net_address_data::NetAddressOptionData,
        option_data::{
            ApplicableOptionData, GetOptionId, OptionData, OptionDataChangeNotification,
            ToNixString, ToOptionId,
        },
        port_data::PortOptionData,
        text_edit_data::TextOptionData,
    },
    errors::{ProjectError, TemplatingError},
    number_value::NumberValue,
    option_definitions::cln::ClnConfigOption,
};

use crate::utils::{BASE_TEMPLATE, update_file};

pub const TEMPLATE_FILE_NAME: &str = "src/btc/cln.nix.templ";
pub const JSON_FILE_NAME: &str = "src/btc/cln.json";

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct CoreLightningService {
    /// Whether the service is enabled or not
    pub enable: Box<BoolOptionData>,

    /// Address to listen for peer connections
    pub address: Box<NetAddressOptionData>,

    /// Port to listen for peer connections
    pub port: Box<PortOptionData>,

    /// Socks proxy for connecting to Tor nodes
    /// (or for all connections if option always-use-proxy is set).
    ///
    /// default: if cfg.tor.proxy then config.nix-bitcoin.torClientAddressWithPort else null;
    pub proxy: Box<TextOptionData>,

    /// Always use the proxy, even to connect to normal IP addresses.
    /// You can still connect to Unix domain sockets manually.
    /// This also disables all DNS lookups, to avoid leaking address information.
    ///
    /// default: cfg.tor.proxy;
    pub always_use_proxy: Box<BoolOptionData>,

    /// The data directory for clightning.
    ///
    /// default: "/mnt/data/clightning"
    pub data_dir: Box<TextOptionData>,

    /// Wallet data scheme (sqlite3 or postgres) and location/connection
    /// parameters, as fully qualified data source name.
    ///
    /// default: null
    /// example: "sqlite3:///mnt/data/clightning/bitcoin/lightningd.sqlite3";
    pub wallet: Box<TextOptionData>,

    /// Extra lines appended to the configuration file.
    ///
    /// See all available options at
    /// https:/// github.com/ElementsProject/lightning/blob/master/doc/lightningd-config.5.md
    /// or by running {command}`lightningd --help`.
    /// example: "
    ///   alias=mynode
    /// "
    pub extra_config: Box<TextOptionData>,

    /// Bash expression which outputs the public service address to announce
    /// to peers. If left empty, no address is announced.
    ///
    /// default: ""
    pub get_public_address_cmd: Box<TextOptionData>,
}

impl AppConfig for CoreLightningService {
    fn get_options(&self) -> Vec<OptionData> {
        vec![
            OptionData::Bool(self.enable.clone()),
            OptionData::NetAddress(self.address.clone()),
            OptionData::Port(self.port.clone()),
            OptionData::TextEdit(self.proxy.clone()),
            OptionData::Bool(self.always_use_proxy.clone()),
            OptionData::TextEdit(self.data_dir.clone()),
            OptionData::TextEdit(self.wallet.clone()),
            OptionData::TextEdit(self.extra_config.clone()),
            OptionData::TextEdit(self.get_public_address_cmd.clone()),
        ]
    }

    fn app_option_changed(
        &mut self,
        option: &OptionDataChangeNotification,
    ) -> Result<bool, ProjectError> {
        let id = option.id();
        if let Ok(opt) = ClnConfigOption::from_str(&id.option) {
            let mut res = Ok(false);
            match opt {
                ClnConfigOption::Enable => {
                    if let OptionDataChangeNotification::Bool(val) = option {
                        res = Ok(self.enable.value() != val.value);
                        self.enable.set_value(val.value);
                    }
                }
                ClnConfigOption::Address => {
                    if let OptionDataChangeNotification::NetAddress(val) = option {
                        res = Ok(self.address.value() != val.value);
                        self.address.set_value(val.value);
                    }
                }
                ClnConfigOption::Port => {
                    if let OptionDataChangeNotification::Port(val) = option {
                        res = Ok(*self.port.value() != val.value);
                        self.port.set_value(val.value.clone());
                    }
                }
                ClnConfigOption::Proxy => {
                    if let OptionDataChangeNotification::TextEdit(val) = option {
                        res = Ok(self.proxy.value() != val.value);
                        self.proxy.set_value(val.value.clone());
                    }
                }
                ClnConfigOption::AlwaysUseProxy => {
                    if let OptionDataChangeNotification::Bool(val) = option {
                        res = Ok(self.always_use_proxy.value() != val.value);
                        self.always_use_proxy.set_value(val.value);
                    }
                }
                ClnConfigOption::DataDir => {
                    if let OptionDataChangeNotification::TextEdit(val) = option {
                        res = Ok(self.data_dir.value() != val.value);
                        self.data_dir.set_value(val.value.clone());
                    }
                }
                ClnConfigOption::Wallet => {
                    if let OptionDataChangeNotification::TextEdit(val) = option {
                        res = Ok(self.wallet.value() != val.value);
                        self.wallet.set_value(val.value.clone());
                    }
                }
                ClnConfigOption::ExtraConfig => {
                    if let OptionDataChangeNotification::TextEdit(val) = option {
                        res = Ok(self.extra_config.value() != val.value);
                        self.extra_config.set_value(val.value.clone());
                    }
                }
                ClnConfigOption::GetPublicAddressCmd => {
                    if let OptionDataChangeNotification::TextEdit(val) = option {
                        res = Ok(self.get_public_address_cmd.value() != val.value);
                        self.get_public_address_cmd.set_value(val.value.clone());
                    }
                }
            }
            return res;
        }

        Ok(false)
    }

    fn save(&mut self, work_dir: &Path) -> Result<(), ProjectError> {
        let rendered_json = self
            .to_json_string()
            .change_context(ProjectError::GenFilesError)?;
        let rendered_nix = self.render().change_context(ProjectError::CreateBaseFiles(
            "Failed at rendering cln config".to_string(),
        ))?;

        for (key, val) in rendered_nix.iter() {
            update_file(
                Path::new(&work_dir.join(key.replace(".templ", ""))),
                val.as_bytes(),
            )?;
        }

        update_file(
            Path::new(&work_dir.join(JSON_FILE_NAME)),
            rendered_json.as_bytes(),
        )?;

        Ok(())
    }

    fn set_applied(&mut self) {
        self.enable.set_applied();
        self.address.set_applied();
        self.port.set_applied();
        self.proxy.set_applied();
        self.always_use_proxy.set_applied();
        self.data_dir.set_applied();
        self.wallet.set_applied();
        self.extra_config.set_applied();
        self.get_public_address_cmd.set_applied();
    }
}

impl Default for CoreLightningService {
    fn default() -> Self {
        Self {
            enable: Box::new(BoolOptionData::new(
                ClnConfigOption::Enable.to_option_id(),
                false,
            )),
            address: Box::new(NetAddressOptionData::new(
                ClnConfigOption::Address.to_option_id(),
                Some(IpAddr::from_str("127.0.0.1").unwrap()),
            )),
            port: Box::new(PortOptionData::new(
                ClnConfigOption::Port.to_option_id(),
                NumberValue::U16(Some(9735)),
            )),
            proxy: Box::new(TextOptionData::new(
                ClnConfigOption::Proxy.to_option_id(),
                "".to_string(),
                1,
                false,
                "".to_string(),
            )),
            always_use_proxy: Box::new(BoolOptionData::new(
                ClnConfigOption::AlwaysUseProxy.to_option_id(),
                false,
            )),
            data_dir: Box::new(TextOptionData::new(
                ClnConfigOption::DataDir.to_option_id(),
                "/mnt/data/clightning".to_string(),
                1,
                false,
                "/mnt/data/clightning".to_string(),
            )),
            wallet: Box::new(TextOptionData::new(
                ClnConfigOption::Wallet.to_option_id(),
                "sqlite3:///mnt/data/clightning/bitcoin/lightningd.sqlite3".to_string(),
                1,
                false,
                "sqlite3:///mnt/data/clightning/bitcoin/lightningd.sqlite3".to_string(),
            )),
            extra_config: Box::new(TextOptionData::new(
                ClnConfigOption::ExtraConfig.to_option_id(),
                "".to_string(),
                9999,
                false,
                "".to_string(),
            )),
            get_public_address_cmd: Box::new(TextOptionData::new(
                ClnConfigOption::GetPublicAddressCmd.to_option_id(),
                "".to_string(),
                1,
                false,
                "".to_string(),
            )),
        }
    }
}

impl CoreLightningService {
    pub fn render(&self) -> Result<HashMap<String, String>, TemplatingError> {
        let mut handlebars = Handlebars::new();
        handlebars.register_escape_fn(no_escape);

        let mut rendered_contents = HashMap::new();
        let file = BASE_TEMPLATE.get_file(TEMPLATE_FILE_NAME);
        let file = match file {
            Some(f) => f,
            None => {
                return Err(Report::new(TemplatingError::FileNotFound(
                    TEMPLATE_FILE_NAME.to_string(),
                ))
                .attach_printable(format!("File {TEMPLATE_FILE_NAME} not found in template")))?;
            }
        };

        let file = match file.contents_utf8() {
            Some(f) => f,
            None => {
                return Err(Report::new(TemplatingError::FileNotFound(
                    TEMPLATE_FILE_NAME.to_string(),
                ))
                .attach_printable(format!(
                    "Unable to read file contents of {TEMPLATE_FILE_NAME}"
                )));
            }
        };

        handlebars
            .register_template_string(TEMPLATE_FILE_NAME, file)
            .attach_printable_lazy(|| format!("{handlebars:?} could not register the template"))
            .change_context(TemplatingError::Register)?;

        let data: HashMap<&str, String> = HashMap::from([
            ("enable", format!("{}", self.enable.value())),
            ("address", self.address.to_nix_string(false)),
            ("port", self.port.value().to_string()),
            (
                "proxy",
                if self.proxy.value().len() > 1 {
                    self.proxy.value().to_string()
                } else {
                    "null".to_string()
                },
            ),
            (
                "always_use_proxy",
                format!("{}", self.always_use_proxy.value()),
            ),
            // keep dataDir quoted. Otherwise, the nix installing will fail
            ("data_dir", self.data_dir.to_nix_string(true)),
            ("wallet", self.wallet.to_nix_string(false)),
            ("extra_config", self.extra_config.value().to_string()),
            (
                "get_public_address_cmd",
                self.get_public_address_cmd.to_nix_string(false),
            ),
        ]);

        let res = handlebars
            .render(TEMPLATE_FILE_NAME, &data)
            .attach_printable("Failed to render Core Lightning template".to_string())
            .change_context(TemplatingError::Render)?;

        let (status, text) = format::in_memory("<cln>".to_string(), res);

        if let format::Status::Error(e) = status {
            Err(Report::new(TemplatingError::Format)).attach_printable_lazy(|| {
                format!("Could not format the template file due to error: {e}")
            })?
        } else {
            rendered_contents.insert(TEMPLATE_FILE_NAME.to_string(), text);
        }

        Ok(rendered_contents)
    }

    pub(crate) fn to_json_string(&self) -> Result<String, TemplatingError> {
        serde_json::to_string_pretty(self).change_context(TemplatingError::JsonRenderError)
    }

    pub(crate) fn from_json(json_data: &str) -> Result<CoreLightningService, TemplatingError> {
        serde_json::from_str(json_data).change_context(TemplatingError::JsonLoadError)
    }
}

#[cfg(test)]
mod tests {
    use std::{fs, net::IpAddr, str::FromStr};
    use tempfile::tempdir;

    use crate::utils::init_default_project;

    use super::*;

    fn get_test_service() -> CoreLightningService {
        CoreLightningService {
            enable: Box::new(BoolOptionData::new(
                ClnConfigOption::Enable.to_option_id(),
                true,
            )),
            address: Box::new(NetAddressOptionData::new(
                ClnConfigOption::Address.to_option_id(),
                Some(IpAddr::from_str("123.2.41.22").unwrap()),
            )),
            port: Box::new(PortOptionData::new(
                ClnConfigOption::Port.to_option_id(),
                NumberValue::U16(Some(3412)),
            )),
            proxy: Box::new(TextOptionData::new(
                ClnConfigOption::Proxy.to_option_id(),
                "".to_string(),
                1,
                false,
                "".to_string(),
            )),
            always_use_proxy: Box::new(BoolOptionData::new(
                ClnConfigOption::AlwaysUseProxy.to_option_id(),
                false,
            )),
            data_dir: Box::new(TextOptionData::new(
                ClnConfigOption::DataDir.to_option_id(),
                "/tmp/testing/lnd".to_string(),
                1,
                false,
                "/tmp/testing/lnd".to_string(),
            )),
            wallet: Box::new(TextOptionData::new(
                ClnConfigOption::Wallet.to_option_id(),
                "sqlite3:///mnt/data/clightning/bitcoin/lightningd.sqlite3".to_string(),
                1,
                false,
                "sqlite3:///mnt/data/clightning/bitcoin/lightningd.sqlite3".to_string(),
            )),
            extra_config: Box::new(TextOptionData::new(
                ClnConfigOption::ExtraConfig.to_option_id(),
                "var1=this is extra config".to_string(),
                1,
                false,
                "var1=this is extra config".to_string(),
            )),
            get_public_address_cmd: Box::new(TextOptionData::new(
                ClnConfigOption::GetPublicAddressCmd.to_option_id(),
                "this is a command".to_string(),
                1,
                false,
                "this is a command".to_string(),
            )),
        }
    }

    #[test]
    fn test_save_function() {
        // Note: maybe test every field? Right now we just check if
        //       enable is set to true or false respectively

        // Create a temporary directory
        let temp_dir = tempdir().unwrap();
        let work_dir = temp_dir.path();

        let _ = init_default_project(work_dir, Some(false));
        let mut service = get_test_service();

        // force enable to "true"
        let _ = service
            .app_option_changed(&OptionDataChangeNotification::Bool(
                nixblitz_core::app_option_data::bool_data::BoolOptionChangeData {
                    id: ClnConfigOption::Enable.to_option_id(),
                    value: true,
                },
            ))
            .unwrap();

        // Call the save function
        let result = service.save(work_dir);

        // Assert that the save function returns Ok
        assert!(result.is_ok());

        let json_file_path = work_dir.join(JSON_FILE_NAME);
        // Check that the JSON file contains the expected content
        let json_content = fs::read_to_string(&json_file_path).unwrap();
        let expected_json_content = service.to_json_string().unwrap();
        assert_eq!(json_content, expected_json_content);

        // Check that the Nix file contains the expected content
        let nix_file_path = work_dir.join(TEMPLATE_FILE_NAME.replace(".templ", ""));
        let rendered_nix = service.render().unwrap();
        let expected_nix_content = rendered_nix.get(TEMPLATE_FILE_NAME).unwrap();
        let nix_content = fs::read_to_string(&nix_file_path).unwrap();
        assert_eq!(nix_content, *expected_nix_content);

        // force enable to "false"
        let _ = service
            .app_option_changed(&OptionDataChangeNotification::Bool(
                nixblitz_core::app_option_data::bool_data::BoolOptionChangeData {
                    id: ClnConfigOption::Enable.to_option_id(),
                    value: false,
                },
            ))
            .unwrap();
        let _ = service.save(work_dir);

        let json_content = fs::read_to_string(&json_file_path).unwrap();
        let expected_json_content = service.to_json_string().unwrap();
        assert_eq!(json_content, expected_json_content);

        let rendered_nix = service.render().unwrap();
        let expected_nix_content = rendered_nix.get(TEMPLATE_FILE_NAME).unwrap();
        let nix_content = fs::read_to_string(nix_file_path).unwrap();
        assert_eq!(nix_content, *expected_nix_content);
    }

    #[test]
    fn test_from_json_string() {
        let source = get_test_service();
        let data = source.to_json_string().unwrap();

        let service = CoreLightningService::from_json(&data);
        assert!(service.is_ok());
        let target = service.unwrap();
        assert!(source == target);
    }

    #[test]
    fn test_render() {
        let s = get_test_service();

        let result = s.render();
        if let Ok(data) = &result {
            println!("{}", data[TEMPLATE_FILE_NAME]);
            assert!(&data.contains_key(TEMPLATE_FILE_NAME));
            let data = &data[TEMPLATE_FILE_NAME];
            assert!(data.contains(&format!("enable = {};", s.enable.value())));
            assert!(data.contains(&format!(
                "address = \"{}\";",
                s.address.to_nix_string(false)
            )));
            assert!(data.contains(&format!("port = {};", s.port.value())));
            assert!(data.contains(&format!("dataDir = {};", s.data_dir.to_nix_string(true))));
            s.extra_config
                .value()
                .lines()
                .for_each(|line| assert!(data.contains(line)));
            assert!(data.contains(&format!(
                "getPublicAddressCmd = \"{}\";",
                s.get_public_address_cmd.to_nix_string(false)
            )));
        } else if let Err(e) = &result {
            println!("{}", e);
        }

        assert!(result.is_ok());
    }
}
