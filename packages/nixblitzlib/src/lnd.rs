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
            OptionData, OptionDataChangeNotification, OptionId, ToNixString, ToOptionId,
        },
        port_data::PortOptionData,
        text_edit_data::TextOptionData,
    },
    apps::SupportedApps,
    errors::{ProjectError, TemplatingError},
    number_value::NumberValue,
    utils::BASE_TEMPLATE,
};

pub const TEMPLATE_FILE_NAME: &str = "src/apps/lnd.nix.templ";
pub const JSON_FILE_NAME: &str = "src/apps/lnd.json";

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct LightningNetworkDaemonService {
    /// Whether the service is enabled or not
    pub enable: Box<BoolOptionData>,

    /// Address to listen for peer connections
    pub address: Box<NetAddressOptionData>,

    /// Port to listen for peer connections
    pub port: Box<PortOptionData>,

    /// The user as which to run LND.
    pub user: Box<TextOptionData>,

    /// Address to listen for gRPC connections.
    pub rpc_address: Box<NetAddressOptionData>,

    /// Port to listen for gRPC connections
    pub rpc_port: Box<PortOptionData>,

    /// Address to listen for REST connections.
    pub rest_address: Box<NetAddressOptionData>,

    /// Port to listen for REST connections.
    pub rest_port: Box<PortOptionData>,

    /// The data directory for LND.
    pub data_dir: Box<TextOptionData>,

    /// The network data directory.
    pub network_dir: Box<TextOptionData>,

    /// Extra `subjectAltName` IPs added to the certificate.
    /// This works the same as lnd option {option}`tlsextraip`.
    pub cert_extra_ips: Box<Vec<NetAddressOptionData>>,

    /// Extra `subjectAltName` domain names added to the certificate.
    /// This works the same as lnd option {option}`tlsextradomain`.
    pub cert_extra_domains: Box<Vec<TextOptionData>>,

    /// Extra lines appended to {file}`lnd.conf`.
    /// See here for all available options:
    /// https://github.com/lightningnetwork/lnd/blob/master/sample-lnd.conf
    pub extra_config: Box<TextOptionData>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LndConfigOption {
    Enable,
    Address,
    Port,
    User,
    RpcAddress,
    RpcPort,
    RestAddress,
    RestPort,
    DataDir,
    NetworkDir,
    CertExtraIps,
    CertExtraDomains,
    ExtraConfig,
}

impl ToOptionId for LndConfigOption {
    fn to_option_id(&self) -> OptionId {
        OptionId::new(SupportedApps::LND, self.to_string())
    }
}
impl FromStr for LndConfigOption {
    type Err = ();

    fn from_str(s: &str) -> std::result::Result<LndConfigOption, ()> {
        match s {
            "enable" => Ok(LndConfigOption::Enable),
            "address" => Ok(LndConfigOption::Address),
            "port" => Ok(LndConfigOption::Port),
            "user" => Ok(LndConfigOption::User),
            "rpc_address" => Ok(LndConfigOption::RpcAddress),
            "rpc_port" => Ok(LndConfigOption::RpcPort),
            "rest_address" => Ok(LndConfigOption::RestAddress),
            "rest_port" => Ok(LndConfigOption::RestPort),
            "data_dir" => Ok(LndConfigOption::DataDir),
            "network_dir" => Ok(LndConfigOption::NetworkDir),
            "cert_extra_ips" => Ok(LndConfigOption::CertExtraIps),
            "cert_extra_domains" => Ok(LndConfigOption::CertExtraDomains),
            "extra_config" => Ok(LndConfigOption::ExtraConfig),
            _ => Err(()),
        }
    }
}

impl fmt::Display for LndConfigOption {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let option_str = match self {
            LndConfigOption::Enable => "enable",
            LndConfigOption::Address => "address",
            LndConfigOption::Port => "port",
            LndConfigOption::User => "user",
            LndConfigOption::RpcAddress => "rpc_address",
            LndConfigOption::RpcPort => "rpc_port",
            LndConfigOption::RestAddress => "rest_address",
            LndConfigOption::RestPort => "rest_port",
            LndConfigOption::DataDir => "data_dir",
            LndConfigOption::NetworkDir => "network_dir",
            LndConfigOption::CertExtraIps => "cert_extra_ips",
            LndConfigOption::CertExtraDomains => "cert_extra_domains",
            LndConfigOption::ExtraConfig => "extra_config",
        };
        write!(f, "{}", option_str)
    }
}

impl AppConfig for LightningNetworkDaemonService {
    fn get_options(&self) -> Vec<OptionData> {
        vec![
            OptionData::Bool(self.enable.clone()),
            OptionData::NetAddress(self.address.clone()),
            OptionData::Port(self.port.clone()),
            OptionData::TextEdit(self.user.clone()),
            OptionData::NetAddress(self.rpc_address.clone()),
            OptionData::Port(self.rpc_port.clone()),
            OptionData::NetAddress(self.rest_address.clone()),
            OptionData::Port(self.rest_port.clone()),
            OptionData::TextEdit(self.data_dir.clone()),
            OptionData::TextEdit(self.network_dir.clone()),
            //OptionData::IpList(self.cert_extra_ips.clone()),
            //OptionData::TextList(self.cert_extra_domains.clone()),
            OptionData::TextEdit(self.extra_config.clone()),
        ]
    }

    fn app_option_changed(
        &mut self,
        id: &OptionId,
        option: &OptionDataChangeNotification,
    ) -> Result<bool, ProjectError> {
        if let Ok(opt) = LndConfigOption::from_str(&id.option) {
            let mut res = Ok(false);
            match opt {
                LndConfigOption::Enable => {
                    if let OptionDataChangeNotification::Bool(val) = option {
                        res = Ok(self.enable.value() != val.value);
                        self.enable.set_value(val.value);
                    }
                }
                LndConfigOption::Address => {
                    if let OptionDataChangeNotification::NetAddress(val) = option {
                        res = Ok(self.address.value() != val.value);
                        self.address.set_value(val.value);
                    }
                }
                LndConfigOption::Port => {
                    if let OptionDataChangeNotification::Port(val) = option {
                        res = Ok(*self.port.value() != val.value);
                        self.port.set_value(val.value.clone());
                    }
                }
                LndConfigOption::User => {
                    if let OptionDataChangeNotification::TextEdit(val) = option {
                        res = Ok(self.user.value() != val.value);
                        self.user.set_value(val.value.clone());
                    }
                }
                LndConfigOption::RpcAddress => {
                    if let OptionDataChangeNotification::NetAddress(val) = option {
                        res = Ok(self.rpc_address.value() != val.value);
                        self.rpc_address.set_value(val.value);
                    }
                }
                LndConfigOption::RpcPort => {
                    if let OptionDataChangeNotification::Port(val) = option {
                        res = Ok(*self.rpc_port.value() != val.value);
                        self.rpc_port.set_value(val.value.clone());
                    }
                }
                LndConfigOption::RestAddress => {
                    if let OptionDataChangeNotification::NetAddress(val) = option {
                        res = Ok(self.rest_address.value() != val.value);
                        self.rest_address.set_value(val.value);
                    }
                }
                LndConfigOption::RestPort => {
                    if let OptionDataChangeNotification::Port(val) = option {
                        res = Ok(*self.rest_port.value() != val.value);
                        self.rest_port.set_value(val.value.clone());
                    }
                }
                LndConfigOption::DataDir => {
                    if let OptionDataChangeNotification::TextEdit(val) = option {
                        res = Ok(self.data_dir.value() != val.value);
                        self.data_dir.set_value(val.value.clone());
                    }
                }
                LndConfigOption::NetworkDir => {
                    if let OptionDataChangeNotification::TextEdit(val) = option {
                        res = Ok(self.network_dir.value() != val.value);
                        self.network_dir.set_value(val.value.clone());
                    }
                }
                LndConfigOption::CertExtraIps => {
                    todo!("implement me");
                    //if let OptionDataChangeNotification::IpList(val) = option {
                    //    res = Ok(self.cert_extra_ips != val.value);
                    //    self.cert_extra_ips = val.value.clone();
                    //}
                }
                LndConfigOption::CertExtraDomains => {
                    todo!("implement me");
                    //if let OptionDataChangeNotification::TextList(val) = option {
                    //    res = Ok(self.cert_extra_domains != val.value);
                    //    self.cert_extra_domains = val.value.clone();
                    //}
                }
                LndConfigOption::ExtraConfig => {
                    if let OptionDataChangeNotification::TextEdit(val) = option {
                        res = Ok(self.extra_config.value() != val.value);
                        self.extra_config.set_value(val.value.clone());
                    }
                }
            }
            return res;
        }

        Ok(false)
    }
}

impl Default for LightningNetworkDaemonService {
    fn default() -> Self {
        Self {
            enable: Box::new(BoolOptionData::new(
                LndConfigOption::Enable.to_option_id(),
                false,
            )),
            address: Box::new(NetAddressOptionData::new(
                LndConfigOption::Address.to_option_id(),
                Some(IpAddr::from_str("127.0.0.1").unwrap()),
            )),
            port: Box::new(PortOptionData::new(
                LndConfigOption::Port.to_option_id(),
                NumberValue::U16(Some(9735)),
            )),
            user: Box::new(TextOptionData::new(
                LndConfigOption::User.to_option_id(),
                "admin".to_string(),
                1,
                false,
                "admin".to_string(),
            )),
            rpc_address: Box::new(NetAddressOptionData::new(
                LndConfigOption::RpcAddress.to_option_id(),
                Some(IpAddr::from_str("127.0.0.1").unwrap()),
            )),
            rpc_port: Box::new(PortOptionData::new(
                LndConfigOption::RpcPort.to_option_id(),
                NumberValue::U16(Some(10009)),
            )),
            rest_address: Box::new(NetAddressOptionData::new(
                LndConfigOption::RestAddress.to_option_id(),
                Some(IpAddr::from_str("127.0.0.1").unwrap()),
            )),
            rest_port: Box::new(PortOptionData::new(
                LndConfigOption::RestPort.to_option_id(),
                NumberValue::U16(Some(8080)),
            )),
            data_dir: Box::new(TextOptionData::new(
                LndConfigOption::DataDir.to_option_id(),
                "/var/lib/lnd".to_string(),
                1,
                false,
                "/var/lib/lnd".to_string(),
            )),
            network_dir: Box::new(TextOptionData::new(
                LndConfigOption::NetworkDir.to_option_id(),
                "${cfg.lnd.dataDir}/chain/bitcoin/${cfg.bitcoind.network}".to_string(),
                1,
                false,
                "${cfg.lnd.dataDir}/chain/bitcoin/${cfg.bitcoind.network}".to_string(),
            )),
            cert_extra_ips: Box::new(Vec::new()),
            cert_extra_domains: Box::new(Vec::new()),
            extra_config: Box::new(TextOptionData::new(
                LndConfigOption::ExtraConfig.to_option_id(),
                "".to_string(),
                1,
                false,
                "".to_string(),
            )),
        }
    }
}

impl LightningNetworkDaemonService {
    pub fn render(&self) -> Result<HashMap<String, String>, TemplatingError> {
        // TODO: I'd like to return a &str key here, as it is always a 'static
        //       reference to the _FILES array. Why no workey?
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
            ("port", format!("{}", self.port.value())),
            ("rpc_address", self.rpc_address.to_nix_string(false)),
            ("rpc_port", format!("{}", self.rpc_port.value())),
            ("rest_address", self.rest_address.to_nix_string(false)),
            ("rest_port", format!("{}", self.rest_port.value())),
            ("data_dir", self.data_dir.value().to_string()),
            ("network_dir", self.network_dir.value().to_string()),
            (
                // TODO: implement me
                "extra_ips",
                self.cert_extra_ips
                    .iter()
                    .map(|s| s.to_nix_string(true).to_string())
                    .collect::<Vec<_>>()
                    .join("\n"),
            ),
            (
                "extra_domains",
                self.cert_extra_domains
                    .iter()
                    .map(|s| format!("\"{}\"", s.value()))
                    .collect::<Vec<_>>()
                    .join("\n"),
            ),
            ("extra_config", self.extra_config.value().to_string()),
        ]);

        let res = handlebars
            .render(TEMPLATE_FILE_NAME, &data)
            .attach_printable("Failed to render LND template".to_string())
            .change_context(TemplatingError::Render)?;
        let (status, text) = format::in_memory("<lnd>".to_string(), res);

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

    pub(crate) fn from_json(
        json_data: &str,
    ) -> Result<LightningNetworkDaemonService, TemplatingError> {
        serde_json::from_str(json_data).change_context(TemplatingError::JsonLoadError)
    }
}

#[cfg(test)]
mod tests {
    use std::{net::IpAddr, str::FromStr};

    use super::*;

    fn get_test_daemon() -> LightningNetworkDaemonService {
        LightningNetworkDaemonService {
            enable: Box::new(BoolOptionData::new(
                LndConfigOption::Enable.to_option_id(),
                true,
            )),
            address: Box::new(NetAddressOptionData::new(
                LndConfigOption::Address.to_option_id(),
                Some(IpAddr::from_str("123.2.41.22").unwrap()),
            )),
            port: Box::new(PortOptionData::new(
                LndConfigOption::Port.to_option_id(),
                NumberValue::U16(Some(3412)),
            )),
            user: Box::new(TextOptionData::new(
                LndConfigOption::User.to_option_id(),
                "tester".to_string(),
                1,
                false,
                "tester".to_string(),
            )),
            rpc_address: Box::new(NetAddressOptionData::new(
                LndConfigOption::RpcAddress.to_option_id(),
                Some(IpAddr::from_str("12.123.12.123").unwrap()),
            )),
            rpc_port: Box::new(PortOptionData::new(
                LndConfigOption::RpcPort.to_option_id(),
                NumberValue::U16(Some(8393)),
            )),
            rest_address: Box::new(NetAddressOptionData::new(
                LndConfigOption::RestAddress.to_option_id(),
                Some(IpAddr::from_str("0.0.0.0").unwrap()),
            )),
            rest_port: Box::new(PortOptionData::new(
                LndConfigOption::RestPort.to_option_id(),
                NumberValue::U16(Some(7369)),
            )),
            data_dir: Box::new(TextOptionData::new(
                LndConfigOption::DataDir.to_option_id(),
                "/tmp/testing/lnd".to_string(),
                1,
                false,
                "/tmp/testing/lnd".to_string(),
            )),
            network_dir: Box::new(TextOptionData::new(
                LndConfigOption::NetworkDir.to_option_id(),
                "/mnt/hdd/somewhere".to_string(),
                1,
                false,
                "/mnt/hdd/somewhere".to_string(),
            )),
            cert_extra_ips: Box::new(vec![
                NetAddressOptionData::new(
                    LndConfigOption::CertExtraIps.to_option_id(),
                    Some(IpAddr::from_str("1.1.1.1").unwrap()),
                ),
                NetAddressOptionData::new(
                    LndConfigOption::CertExtraIps.to_option_id(),
                    Some(IpAddr::from_str("2.2.2.2").unwrap()),
                ),
            ]),
            cert_extra_domains: Box::new(vec![
                TextOptionData::new(
                    LndConfigOption::CertExtraDomains.to_option_id(),
                    "abc.de".to_string(),
                    1,
                    false,
                    "abc.de".to_string(),
                ),
                TextOptionData::new(
                    LndConfigOption::CertExtraDomains.to_option_id(),
                    "cde.fg".to_string(),
                    1,
                    false,
                    "cde.fg".to_string(),
                ),
            ]),
            extra_config: Box::new(TextOptionData::new(
                LndConfigOption::ExtraConfig.to_option_id(),
                "var1=this is extra config".to_string(),
                1,
                false,
                "var1=this is extra config".to_string(),
            )),
        }
    }

    #[test]
    fn test_from_json_string() {
        let source = get_test_daemon();
        let data = source.to_json_string().unwrap();

        let service = LightningNetworkDaemonService::from_json(&data);
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
            assert!(data.contains(&format!(
                "rpcAddress = {};",
                s.rpc_address.to_nix_string(true)
            )));
            assert!(data.contains(&format!("rpcPort = {};", s.rpc_port.value())));
            assert!(data.contains(&format!(
                "restAddress = {};",
                s.rest_address.to_nix_string(true)
            )));
            assert!(data.contains(&format!("restPort = {};", s.rest_port.value())));
            assert!(data.contains(&format!("dataDir = \"{}\";", s.data_dir.value())));
            assert!(data.contains(&format!("networkDir = \"{}\";", s.network_dir.value())));
            s.cert_extra_ips
                .iter()
                .for_each(|ip| assert!(data.contains(&format!("\"{}\"", ip.to_nix_string(false)))));
            s.cert_extra_domains
                .iter()
                .for_each(|domain| assert!(data.contains(&format!("\"{}\"", domain.value()))));
            assert!(data.contains(&s.extra_config.value().to_string()));
        }

        assert!(result.is_ok());
    }
}
