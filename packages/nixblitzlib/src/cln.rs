use core::fmt;
use std::{collections::HashMap, net::IpAddr, str::FromStr};

use alejandra::format;
use error_stack::{Report, Result, ResultExt};
use handlebars::{no_escape, Handlebars};
use serde::{Deserialize, Serialize};

use crate::{
    app_config::AppConfig,
    app_option_data::{
        bool_data::BoolOptionData,
        net_address_data::NetAddressOptionData,
        option_data::{
            GetOptionId, OptionData, OptionDataChangeNotification, OptionId, ToNixString,
            ToOptionId,
        },
        port_data::PortOptionData,
        text_edit_data::TextOptionData,
    },
    apps::SupportedApps,
    errors::{ProjectError, TemplatingError},
    number_value::NumberValue,
    utils::BASE_TEMPLATE,
};

pub const TEMPLATE_FILE_NAME: &str = "src/apps/cln.nix.templ";
pub const JSON_FILE_NAME: &str = "src/apps/cln.json";

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
    /// default: "/var/lib/clightning"
    pub data_dir: Box<TextOptionData>,

    /// Wallet data scheme (sqlite3 or postgres) and location/connection
    /// parameters, as fully qualified data source name.
    ///
    /// default: null
    /// example: "sqlite3:///var/lib/clightning/bitcoin/lightningd.sqlite3";
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

    /// The user as which to run clightning.
    ///
    /// default: "clightniung"
    pub user: Box<TextOptionData>,

    /// The group as which to run clightning.
    ///
    /// default: "cfg.user"
    pub group: Box<TextOptionData>,

    /// Bash expression which outputs the public service address to announce
    /// to peers. If left empty, no address is announced.
    ///
    /// default: ""
    pub get_public_address_cmd: Box<TextOptionData>,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum ClnConfigOption {
    Enable,
    Address,
    Port,
    Proxy,
    AlwaysUseProxy,
    DataDir,
    Wallet,
    ExtraConfig,
    User,
    Group,
    GetPublicAddressCmd,
}

impl ToOptionId for ClnConfigOption {
    fn to_option_id(&self) -> OptionId {
        OptionId::new(SupportedApps::CoreLightning, self.to_string())
    }
}
impl FromStr for ClnConfigOption {
    type Err = ();

    fn from_str(s: &str) -> std::result::Result<ClnConfigOption, ()> {
        match s {
            "enable" => Ok(ClnConfigOption::Enable),
            "address" => Ok(ClnConfigOption::Address),
            "port" => Ok(ClnConfigOption::Port),
            "proxy" => Ok(ClnConfigOption::Proxy),
            "always_use_proxy" => Ok(ClnConfigOption::AlwaysUseProxy),
            "data_dir" => Ok(ClnConfigOption::DataDir),
            "wallet" => Ok(ClnConfigOption::Wallet),
            "extra_config" => Ok(ClnConfigOption::ExtraConfig),
            "user" => Ok(ClnConfigOption::User),
            "group" => Ok(ClnConfigOption::Group),
            "get_public_address_cmd" => Ok(ClnConfigOption::GetPublicAddressCmd),
            _ => Err(()),
        }
    }
}

impl fmt::Display for ClnConfigOption {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let option_str = match self {
            ClnConfigOption::Enable => "enable",
            ClnConfigOption::Address => "address",
            ClnConfigOption::Port => "port",
            ClnConfigOption::Proxy => "proxy",
            ClnConfigOption::AlwaysUseProxy => "always_use_proxy",
            ClnConfigOption::DataDir => "data_dir",
            ClnConfigOption::Wallet => "wallet",
            ClnConfigOption::ExtraConfig => "extra_config",
            ClnConfigOption::User => "user",
            ClnConfigOption::Group => "group",
            ClnConfigOption::GetPublicAddressCmd => "get_public_address_cmd",
        };
        write!(f, "{}", option_str)
    }
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
            OptionData::TextEdit(self.user.clone()),
            OptionData::TextEdit(self.group.clone()),
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
                ClnConfigOption::User => {
                    if let OptionDataChangeNotification::TextEdit(val) = option {
                        res = Ok(self.user.value() != val.value);
                        self.user.set_value(val.value.clone());
                    }
                }
                ClnConfigOption::Group => {
                    if let OptionDataChangeNotification::TextEdit(val) = option {
                        res = Ok(self.group.value() != val.value);
                        self.group.set_value(val.value.clone());
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
                "/var/lib/clightning".to_string(),
                1,
                false,
                "/var/lib/clightning".to_string(),
            )),
            wallet: Box::new(TextOptionData::new(
                ClnConfigOption::Wallet.to_option_id(),
                "sqlite3:///var/lib/clightning/bitcoin/lightningd.sqlite3".to_string(),
                1,
                false,
                "sqlite3:///var/lib/clightning/bitcoin/lightningd.sqlite3".to_string(),
            )),
            extra_config: Box::new(TextOptionData::new(
                ClnConfigOption::ExtraConfig.to_option_id(),
                "".to_string(),
                9999,
                false,
                "".to_string(),
            )),
            user: Box::new(TextOptionData::new(
                ClnConfigOption::User.to_option_id(),
                "admin".to_string(),
                1,
                false,
                "admin".to_string(),
            )),
            group: Box::new(TextOptionData::new(
                ClnConfigOption::Group.to_option_id(),
                "cfg.user".to_string(),
                1,
                false,
                "cfg.user".to_string(),
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
                .attach_printable(format!("File {TEMPLATE_FILE_NAME} not found in template")))?
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
                )))
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
            ("data_dir", format!("\"{}\"", self.data_dir.value())),
            ("wallet", format!("\"{}\"", self.wallet.value())),
            ("extra_config", self.extra_config.value().to_string()),
            ("user", format!("\"{}\"", self.user.value())),
            ("group", format!("\"{}\"", self.group.value())),
            (
                "get_public_address_cmd",
                format!("\"{}\"", self.get_public_address_cmd.value()),
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
        serde_json::to_string(self).change_context(TemplatingError::JsonRenderError)
    }

    pub(crate) fn from_json(json_data: &str) -> Result<CoreLightningService, TemplatingError> {
        serde_json::from_str(json_data).change_context(TemplatingError::JsonLoadError)
    }
}

#[cfg(test)]
mod tests {
    use std::{net::IpAddr, str::FromStr};

    use super::*;

    fn get_test_daemon() -> CoreLightningService {
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
                "sqlite3:///var/lib/clightning/bitcoin/lightningd.sqlite3".to_string(),
                1,
                false,
                "sqlite3:///var/lib/clightning/bitcoin/lightningd.sqlite3".to_string(),
            )),
            extra_config: Box::new(TextOptionData::new(
                ClnConfigOption::ExtraConfig.to_option_id(),
                "var1=this is extra config".to_string(),
                1,
                false,
                "var1=this is extra config".to_string(),
            )),
            user: Box::new(TextOptionData::new(
                ClnConfigOption::User.to_option_id(),
                "tester".to_string(),
                1,
                false,
                "tester".to_string(),
            )),
            group: Box::new(TextOptionData::new(
                ClnConfigOption::Group.to_option_id(),
                "cfg.user".to_string(),
                1,
                false,
                "cfg.user".to_string(),
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

    #[test]
    fn test_from_json_string() {
        let source = get_test_daemon();
        let data = source.to_json_string().unwrap();

        let service = CoreLightningService::from_json(&data);
        assert!(service.is_ok());
        let target = service.unwrap();
        assert!(source == target);
    }

    #[test]
    fn test_render() {
        let s = get_test_daemon();

        let result = s.render();
        if let Ok(data) = &result {
            println!("{}", data[TEMPLATE_FILE_NAME]);
            assert!(&data.contains_key(TEMPLATE_FILE_NAME));
            let data = &data[TEMPLATE_FILE_NAME];
            assert!(data.contains(&format!("enable = {};", s.enable.value())));
            assert!(data.contains(&format!("address = {};", s.address.to_nix_string(true))));
            assert!(data.contains(&format!("port = {};", s.port.value())));
            assert!(data.contains(&format!("dataDir = \"{}\";", s.data_dir.value())));
            s.extra_config
                .value()
                .lines()
                .for_each(|line| assert!(data.contains(line)));
            assert!(data.contains(&format!("user = \"{}\";", s.user.value())));
            assert!(data.contains(&format!("group = \"{}\";", s.group.value())));
            assert!(data.contains(&format!(
                "getPublicAddressCmd = \"{}\";",
                s.get_public_address_cmd.value()
            )));
        } else if let Err(e) = &result {
            println!("{}", e);
        }

        assert!(result.is_ok());
    }
}
