use std::{collections::HashMap, net::IpAddr, str::FromStr};

use alejandra::format;
use error_stack::{Report, Result, ResultExt};
use garde::Validate;
use handlebars::{no_escape, Handlebars};
use serde::{Deserialize, Serialize};

use crate::{errors::TemplatingError, utils::BASE_TEMPLATE};

pub const TEMPLATE_FILE_NAME: &str = "src/apps/lnd.nix.templ";
pub const JSON_FILE_NAME: &str = "src/apps/lnd.json";

#[derive(Debug, Validate, Serialize, Deserialize, PartialEq, Eq)]
pub struct LightningNetworkDaemonService {
    /// Whether the service is enabled or not
    #[garde(skip)]
    pub enable: bool,

    /// Address to listen for peer connections
    #[garde(skip)]
    pub address: IpAddr,

    /// Port to listen for peer connections
    #[garde(range(min = 1024, max = 65535))]
    pub port: u16,

    /// The user as which to run LND.
    #[garde(length(min = 3))]
    pub user: String,

    /// Address to listen for gRPC connections.
    #[garde(skip)]
    pub rpc_address: IpAddr,

    /// Port to listen for gRPC connections
    #[garde(range(min = 1024, max = 65535))]
    pub rpc_port: u16,

    /// Address to listen for REST connections.
    #[garde(skip)]
    pub rest_address: IpAddr,

    /// Port to listen for REST connections.
    #[garde(range(min = 1024, max = 65535))]
    pub rest_port: u16,

    /// The data directory for LND.
    #[garde(skip)]
    pub data_dir: String,

    /// The network data directory.
    #[garde(skip)]
    pub network_dir: String,

    /// Extra `subjectAltName` IPs added to the certificate.
    /// This works the same as lnd option {option}`tlsextraip`.
    #[garde(skip)]
    pub cert_extra_ips: Vec<IpAddr>,

    /// Extra `subjectAltName` domain names added to the certificate.
    /// This works the same as lnd option {option}`tlsextradomain`.
    #[garde(skip)]
    pub cert_extra_domains: Vec<String>,

    /// Extra lines appended to {file}`lnd.conf`.
    /// See here for all available options:
    /// https://github.com/lightningnetwork/lnd/blob/master/sample-lnd.conf
    #[garde(skip)]
    pub extra_config: String,
}

impl Default for LightningNetworkDaemonService {
    fn default() -> Self {
        Self {
            enable: false,
            address: IpAddr::from_str("127.0.0.1").unwrap(),
            port: 9735,
            user: "admin".into(),
            rpc_address: IpAddr::from_str("127.0.0.1").unwrap(),
            rpc_port: 10009,
            rest_address: IpAddr::from_str("127.0.0.1").unwrap(),
            rest_port: 8080,
            data_dir: "/var/lib/lnd".into(),
            network_dir: "${cfg.dataDir}/chain/bitcoin/${bitcoind.network}".into(),
            cert_extra_ips: [].to_vec(),
            cert_extra_domains: [].to_vec(),
            extra_config: "".into(),
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
            ("enable", format!("{}", self.enable)),
            ("address", self.address.to_string()),
            ("port", format!("{}", self.port)),
            ("rpc_address", self.rpc_address.to_string()),
            ("rpc_port", format!("{}", self.rpc_port)),
            ("rest_address", self.rest_address.to_string()),
            ("rest_port", format!("{}", self.rest_port)),
            ("data_dir", self.data_dir.to_string()),
            ("network_dir", self.network_dir.to_string()),
            (
                "extra_ips",
                self.cert_extra_ips
                    .iter()
                    .map(|s| format!("\"{}\"", s))
                    .collect::<Vec<_>>()
                    .join("\n"),
            ),
            (
                "extra_domains",
                self.cert_extra_domains
                    .iter()
                    .map(|s| format!("\"{}\"", s))
                    .collect::<Vec<_>>()
                    .join("\n"),
            ),
            ("extra_config", self.extra_config.to_string()),
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
            enable: true,
            address: IpAddr::from_str("123.2.41.22").unwrap(),
            port: 3412,
            user: "tester".into(),
            rpc_address: IpAddr::from_str("34.142.12.78").unwrap(),
            rpc_port: 8393,
            rest_address: IpAddr::from_str("0.0.0.0").unwrap(),
            rest_port: 7369,
            data_dir: "/tmp/testing/lnd".into(),
            network_dir: "/mnt/hdd/somewhere".into(),
            cert_extra_ips: [
                IpAddr::from_str("1.1.1.1").unwrap(),
                IpAddr::from_str("2.2.2.2").unwrap(),
            ]
            .to_vec(),
            cert_extra_domains: ["abc.de".into(), "cde.fg".into()].to_vec(),
            extra_config: "var1=this is extra config".into(),
        }
    }

    #[test]
    fn test_validate() {
        let service = get_test_daemon();
        let result = service.validate(&());
        assert!(result.is_ok());
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
        println!(
            "{},\n\n {}",
            s.cert_extra_ips
                .iter()
                .map(|ip| format!("        {}", ip))
                .collect::<Vec<_>>()
                .join("\n"),
            s.cert_extra_domains
                .iter()
                .map(|ip| format!("        {}", ip))
                .collect::<Vec<_>>()
                .join("\n")
        );

        let result = s.render();
        if let Ok(data) = &result {
            assert!(&data.contains_key(TEMPLATE_FILE_NAME));
            let data = &data[TEMPLATE_FILE_NAME];
            assert!(data.contains(&format!("enable = {};", s.enable)));
            assert!(data.contains(&format!("address = \"{}\";", s.address)));
            assert!(data.contains(&format!("port = {};", s.port)));
            assert!(data.contains(&format!("rpcAddress = \"{}\";", s.rpc_address)));
            assert!(data.contains(&format!("rpcPort = {};", s.rpc_port)));
            assert!(data.contains(&format!("restAddress = \"{}\";", s.rest_address)));
            assert!(data.contains(&format!("restPort = {};", s.rest_port)));
            assert!(data.contains(&format!("dataDir = \"{}\";", s.data_dir)));
            assert!(data.contains(&format!("networkDir = \"{}\";", s.network_dir)));
            s.cert_extra_ips
                .iter()
                .for_each(|ip| assert!(data.contains(&format!("\"{}\"", ip))));
            s.cert_extra_domains
                .iter()
                .for_each(|domain| assert!(data.contains(&format!("\"{}\"", domain))));
            assert!(data.contains(&s.extra_config.to_string()));
        }

        assert!(result.is_ok());
    }
}
