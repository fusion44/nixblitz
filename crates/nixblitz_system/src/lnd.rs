use std::{collections::HashMap, net::IpAddr, path::Path, str::FromStr};

use alejandra::format;
use error_stack::{Report, Result, ResultExt};
use handlebars::{Handlebars, no_escape};
use log::warn;
use serde::{Deserialize, Serialize};

use crate::utils::{BASE_TEMPLATE, update_file};
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
    option_definitions::lnd::LndConfigOption,
};

pub const TEMPLATE_FILE_NAME: &str = "src/btc/lnd.nix.templ";
pub const JSON_FILE_NAME: &str = "src/btc/lnd.json";

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct LightningNetworkDaemonService {
    /// Whether the service is enabled or not
    pub enable: Box<BoolOptionData>,

    /// Address to listen for peer connections
    pub address: Box<NetAddressOptionData>,

    /// Port to listen for peer connections
    pub port: Box<PortOptionData>,

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

impl AppConfig for LightningNetworkDaemonService {
    fn get_options(&self) -> Vec<OptionData> {
        vec![
            OptionData::Bool(self.enable.clone()),
            OptionData::NetAddress(self.address.clone()),
            OptionData::Port(self.port.clone()),
            OptionData::NetAddress(self.rpc_address.clone()),
            OptionData::Port(self.rpc_port.clone()),
            OptionData::NetAddress(self.rest_address.clone()),
            OptionData::Port(self.rest_port.clone()),
            //OptionData::TextEdit(self.data_dir.clone()),
            //OptionData::IpList(self.cert_extra_ips.clone()),
            //OptionData::TextList(self.cert_extra_domains.clone()),
            OptionData::TextEdit(self.extra_config.clone()),
        ]
    }

    fn app_option_changed(
        &mut self,
        option: &OptionDataChangeNotification,
    ) -> Result<bool, ProjectError> {
        let id = option.id();
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
                LndConfigOption::CertExtraIps => {
                    warn!("implement me");
                    //if let OptionDataChangeNotification::IpList(val) = option {
                    //    res = Ok(self.cert_extra_ips != val.value);
                    //    self.cert_extra_ips = val.value.clone();
                    //}
                }
                LndConfigOption::CertExtraDomains => {
                    warn!("implement me");
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

    fn save(&mut self, work_dir: &Path) -> Result<(), ProjectError> {
        let rendered_json = self
            .to_json_string()
            .change_context(ProjectError::GenFilesError)?;
        let rendered_nix = self.render().change_context(ProjectError::CreateBaseFiles(
            "Failed at rendering lnd config".to_string(),
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
        self.rpc_address.set_applied();
        self.rpc_port.set_applied();
        self.rest_address.set_applied();
        self.rest_port.set_applied();
        self.data_dir.set_applied();
        for v in self.cert_extra_ips.iter_mut() {
            v.set_applied();
        }
        for v in self.cert_extra_domains.iter_mut() {
            v.set_applied();
        }
        self.extra_config.set_applied();
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
                "/mnt/data/lnd".to_string(),
                1,
                false,
                "/mnt/data/lnd".to_string(),
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
            ("port", format!("{}", self.port.value())),
            ("rpc_address", self.rpc_address.to_nix_string(false)),
            ("rpc_port", format!("{}", self.rpc_port.value())),
            ("rest_address", self.rest_address.to_nix_string(false)),
            ("rest_port", format!("{}", self.rest_port.value())),
            // keep dataDir quoted. Otherwise, the nix installing will fail
            ("data_dir", self.data_dir.to_nix_string(true)),
            (
                // TODO: implement me
                "cert_extra_ips",
                self.cert_extra_ips
                    .iter()
                    .map(|s| s.to_nix_string(true).to_string())
                    .collect::<Vec<_>>()
                    .join("\n"),
            ),
            (
                "cert_extra_domains",
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
        serde_json::to_string_pretty(self).change_context(TemplatingError::JsonRenderError)
    }

    pub(crate) fn from_json(
        json_data: &str,
    ) -> Result<LightningNetworkDaemonService, TemplatingError> {
        serde_json::from_str(json_data).change_context(TemplatingError::JsonLoadError)
    }
}

#[cfg(test)]
mod tests {
    use std::{fs, net::IpAddr, str::FromStr};
    use tempfile::tempdir;

    use crate::utils::init_default_project;

    use super::*;

    fn get_test_service() -> LightningNetworkDaemonService {
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
                    id: LndConfigOption::Enable.to_option_id(),
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
                    id: LndConfigOption::Enable.to_option_id(),
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

        let service = LightningNetworkDaemonService::from_json(&data);
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
            assert!(data.contains(&format!(
                "rpcAddress = \"{}\";",
                s.rpc_address.to_nix_string(false)
            )));
            assert!(data.contains(&format!("rpcPort = {};", s.rpc_port.value())));
            assert!(data.contains(&format!(
                "restAddress = \"{}\";",
                s.rest_address.to_nix_string(false)
            )));
            assert!(data.contains(&format!("restPort = {};", s.rest_port.value())));
            assert!(data.contains(&format!("dataDir = \"{}\";", s.data_dir.value())));
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
