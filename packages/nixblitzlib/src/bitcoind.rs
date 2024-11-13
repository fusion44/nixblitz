use core::{fmt, str};
use std::{collections::HashMap, net::IpAddr, str::FromStr};

use alejandra::format;

use error_stack::{Report, Result, ResultExt};
use garde::Validate;
use handlebars::{no_escape, Handlebars};
use serde::{Deserialize, Serialize};

use crate::{errors::TemplatingError, utils::BASE_TEMPLATE};

/// Represents all available Bitcoin network options
#[derive(Copy, Clone, Debug, Default, Serialize, Deserialize, Eq, PartialEq)]
pub enum BitcoinNetwork {
    #[default]
    /// [default] The mainnet network
    Mainnet,

    /// The testnet network
    Testnet,

    /// The regtest network
    Regtest,

    /// The signet network
    Signet,
}

impl fmt::Display for BitcoinNetwork {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BitcoinNetwork::Mainnet => write!(f, "Mainnet"),
            BitcoinNetwork::Testnet => write!(f, "Testnet"),
            BitcoinNetwork::Regtest => write!(f, "Regtest"),
            BitcoinNetwork::Signet => write!(f, "Signet"),
        }
    }
}

/// Prune options for a blockchain.
///
/// This enum defines the different pruning strategies that can be used.
/// # Variants
/// - Disable: Pruning is disabled. This is the default option.
/// - Manual: Pruning is performed manually by the user.
/// - Automatic: Pruning is performed automatically when the blockchain reaches a certain size. ///
/// # Examples
/// ```
/// use nixblitzlib::bitcoind::PruneOptions;
///
/// let options = PruneOptions::Automatic { prune_at: 1024 };
/// ```
#[derive(Debug, Validate, Default, Serialize, Deserialize, Eq, PartialEq)]
pub enum PruneOptions {
    #[default]
    /// [default] Pruning disabled
    Disable,

    /// Manual pruning.
    ///
    /// The user is responsible for pruning the blockchain via RPC.
    Manual,

    /// Automatic pruning at a certain blockchain size.
    ///
    /// * Only active if prune is set to automatic.
    /// * Must be at least 551 MiB.
    /// * The `field` represents the size in MiB at which automatic pruning should occur.
    Automatic {
        /// The size in MiB at which automatic pruning should occur.
        ///
        /// This field must be at least 551 MiB.
        #[garde(range(min = 551))]
        prune_at: u32,
    },
}

#[derive(Debug, Validate, Serialize, Deserialize, Eq, PartialEq)]
pub struct BitcoinDaemonServiceRPCUser {
    /// Password HMAC-SHA-256 for JSON-RPC connections. Must be a string of the format <SALT-HEX>$<HMAC-HEX>.

    /// Tool (Python script) for HMAC generation is available here:
    /// https://github.com/bitcoin/bitcoin/blob/master/share/rpcauth/rpcauth.py
    #[garde(pattern("[0-9a-f]+\\$[0-9a-f]{64}"))]
    pub password_hmac: String,

    /// Username for JSON-RPC connections.
    #[garde(length(min = 3))]
    pub name: String,
}

impl BitcoinDaemonServiceRPCUser {
    pub fn new(password_hmac: String, name: String) -> Self {
        Self {
            password_hmac,
            name,
        }
    }
}

#[derive(Debug, Validate, Serialize, Deserialize, PartialEq, Eq)]
pub struct BitcoinDaemonService {
    /// Whether the service is enabled or not
    #[garde(skip)]
    pub enable: bool,

    /// Address to listen for peer connections
    #[garde(skip)]
    pub address: IpAddr,

    /// Port to listen for peer connections.
    ///
    /// Default: mainnet 8333
    ///          regtest 18444
    #[garde(range(min = 1024, max = 65535))]
    pub port: u16,

    /// Port to listen for Tor peer connections.
    /// If set, inbound connections to this port are tagged as onion peers.
    ///
    /// Default: None
    ///          mainnet 8334
    ///          regtest 18445
    #[garde(range(min = 1024, max = 65535))]
    pub onion_port: Option<u16>,

    /// Listen for peer connections at `address:port`
    /// and `address:onionPort` (if {option}`onionPort` is set).
    ///
    /// Default: false
    #[garde(skip)]
    pub listen: bool,

    /// Additional configurations to be appended to bitcoin.conf
    /// Strings concatenated with "\n"
    ///
    /// # Example
    ///
    /// ''
    /// par=16
    /// rpcthreads=16
    /// logips=1
    /// ''
    #[garde(skip)]
    pub extra_config: String,

    /// The user as which to run bitcoind.
    #[garde(length(min = 3))]
    pub user: String,

    /// Whether to use the testnet instead of mainnet.
    #[garde(skip)]
    pub network: BitcoinNetwork,

    /// Allowed users for JSON-RPC connections.
    #[garde(skip)]
    pub rpc_users: Vec<BitcoinDaemonServiceRPCUser>,

    /// Address to listen for rpc connections
    #[garde(skip)]
    pub rpc_address: IpAddr,

    /// Override the default port on which to listen for JSON-RPC connections.
    #[garde(range(min = 1024, max = 65535))]
    pub rpc_port: u16,

    /// Hosts that should be allowed to connect to the RPC server
    ///
    /// Example: "192.168.0.0/16"
    /// Default: None
    #[garde(skip)]
    pub rpc_allow_ip: Vec<IpAddr>,

    /// Whether to prune the node
    // #[garde(custom(_check_prune(&self)))]
    #[garde(skip)]
    pub prune: PruneOptions,

    /// The size in MiB at which the blockchain on disk will be pruned.
    ///
    /// * Only active if prune is set to automatic
    /// * Must be at least 500 MiB
    #[garde(range(min = 500))]
    pub prune_size: u16,

    /// Extra command line options to pass to bitcoind. Run bitcoind –help to list all available options.
    #[garde(skip)]
    pub extra_cmd_line_options: Vec<String>,

    /// Override the default database cache size in MiB.
    /// Integer between 4 and 16384 (both inclusive)
    ///
    /// Example: 4000
    /// Default: None
    #[garde(range(min = 4, max = 16384))]
    pub db_cache: Option<i16>,

    /// The data directory for bitcoind.
    ///
    /// Default: "/var/lib/bitcoind"
    #[garde(skip)]
    pub data_dir: String,

    /// Whether to enable the tx index
    #[garde(skip)]
    pub tx_index: bool,

    /// Whether to enable the integrated wallet
    #[garde(skip)]
    pub disable_wallet: bool,

    /// ZMQ address for zmqpubrawtx notifications
    ///
    /// # Example
    /// "tcp://127.0.0.1:28333"
    #[garde(skip)]
    pub zmqpubrawtx: Option<String>,

    /// ZMQ address for zmqpubrawblock notifications
    ///
    /// # Example
    /// "tcp://127.0.0.1:28332"
    #[garde(skip)]
    pub zmqpubrawblock: Option<String>,
}

impl Default for BitcoinDaemonService {
    fn default() -> Self {
        Self {
            enable: false,
            address: IpAddr::from_str("127.0.0.1").unwrap(),
            port: 8333,
            onion_port: None,
            listen: false,
            extra_config: "".into(),
            user: "admin".into(),
            network: BitcoinNetwork::Mainnet,
            rpc_users: [].into(),
            rpc_address: IpAddr::from_str("127.0.0.1").unwrap(),
            rpc_port: 8332,
            rpc_allow_ip: [].into(),
            prune: PruneOptions::Disable,
            prune_size: 2000,
            extra_cmd_line_options: [].into(),
            db_cache: None,
            data_dir: "/var/lib/bitcoind".into(),
            tx_index: false,
            disable_wallet: true,
            zmqpubrawtx: None,
            zmqpubrawblock: None,
        }
    }
}

const FILE_NAME: &str = "src/apps/bitcoind.nix.templ";

fn quoted_string_or_null(value: Option<String>) -> String {
    match value {
        Some(value) => format!("\"{}\"", value).to_string(),
        None => "null".to_string(),
    }
}
impl BitcoinDaemonService {
    pub fn render(&self) -> Result<HashMap<String, String>, TemplatingError> {
        let mut handlebars = Handlebars::new();
        handlebars.register_escape_fn(no_escape);

        let mut rendered_contents = HashMap::new();
        let file = BASE_TEMPLATE.get_file(FILE_NAME);
        let file = match file {
            Some(f) => f,
            None => {
                return Err(Report::new(TemplatingError::FileNotFound)
                    .attach_printable(format!("File {FILE_NAME} not found in template")))
            }
        };

        let file = match file.contents_utf8() {
            Some(f) => f,
            None => {
                return Err(Report::new(TemplatingError::FileNotFound)
                    .attach_printable(format!("Unable to read file contents of {FILE_NAME}")))
            }
        };

        handlebars
            .register_template_string(FILE_NAME, file)
            .attach_printable_lazy(|| format!("{handlebars:?} could not register the template"))
            .change_context(TemplatingError::Register)?;

        let data: HashMap<&str, String> = HashMap::from([
            ("enable", self.enable.to_string()),
            (
                "regtest",
                (self.network == BitcoinNetwork::Regtest).to_string(),
            ),
            ("tx_index", self.tx_index.to_string()),
            ("disable_wallet", self.disable_wallet.to_string()),
            ("address", self.address.to_string()),
            ("listen", self.listen.to_string()),
            ("port", self.port.to_string()),
            ("rpc_address", self.rpc_address.to_string()),
            ("rpc_port", self.rpc_port.to_string()),
            (
                "rpc_allow_ip",
                self.rpc_allow_ip
                    .iter()
                    .map(|s| format!("\"{}\"", s))
                    .collect::<Vec<_>>()
                    .join("\n"),
            ),
            (
                "rpc_users",
                self.rpc_users
                    .iter()
                    .map(|s| format!("{}={{ passwordHMAC = \"{}\"; }};", s.name, s.password_hmac))
                    .collect::<Vec<_>>()
                    .join("\n"),
            ),
            (
                "zmqpubrawblock",
                quoted_string_or_null(self.zmqpubrawblock.clone()),
            ),
            (
                "zmqpubrawtx",
                quoted_string_or_null(self.zmqpubrawtx.clone()),
            ),
        ]);

        let res = handlebars
            .render(FILE_NAME, &data)
            .attach_printable("Failed to render bitcoin daemon template".to_string())
            .change_context(TemplatingError::Render)?;

        let (status, text) = format::in_memory("<convert bitcoind>".to_string(), res);

        if let format::Status::Error(e) = status {
            Err(Report::new(TemplatingError::Format)).attach_printable_lazy(|| {
                format!("Could not format the template file due to error: {e}")
            })?
        } else {
            println!("{}", text);
            rendered_contents.insert(FILE_NAME.to_string(), text);
        }

        Ok(rendered_contents)
    }

    pub(crate) fn to_json_string(&self) -> Result<String, TemplatingError> {
        serde_json::to_string(self).change_context(TemplatingError::JsonRenderError)
    }

    pub(crate) fn from_json(json_data: &str) -> Result<BitcoinDaemonService, TemplatingError> {
        serde_json::from_str(json_data).change_context(TemplatingError::JsonLoadError)
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    fn get_test_daemon() -> BitcoinDaemonService {
        let enable = true;
        let address = IpAddr::from_str("127.0.0.1").unwrap();
        let port = 8333;
        let network = BitcoinNetwork::Testnet;
        let tx_index = true;
        let onion_port = Some(1551);
        let listen = false;
        let extra_config = "extra_config_value".to_string();
        let user = "user_name".to_string();
        let rpc_users = vec![
            BitcoinDaemonServiceRPCUser::new("rpc_user1".into(), "dsfsdf".into()),
            BitcoinDaemonServiceRPCUser::new("rpc_user2".into(), "owieru".into()),
        ];

        let rpc_address = IpAddr::from_str("128.22.22.4").unwrap();
        let rpc_port = 8332;
        let rpc_allow_ip: Vec<IpAddr> = vec![
            IpAddr::from_str("192.168.1.100").unwrap(),
            IpAddr::from_str("192.168.1.111").unwrap(),
        ];
        let prune = PruneOptions::Automatic { prune_at: 2500 };
        let prune_size = 500;
        let extra_cmd_line_options: Vec<String> =
            vec!["option1".to_string(), "option2=value".to_string()];
        let db_cache = Some(2048);
        let data_dir = "/path/to/data/dir".to_string();
        let disable_wallet = true;
        let zmqpubrawtx = Some("zmqpubrawtx_test".to_string());
        let zmqpubrawblock = Some("zmqpubrawblock_test".to_string());

        BitcoinDaemonService {
            enable,
            address: address.clone(),
            port,
            network,
            tx_index,
            onion_port,
            listen,
            extra_config: extra_config.clone(),
            user: user.clone(),
            rpc_users,
            rpc_address: rpc_address.clone(),
            rpc_port,
            rpc_allow_ip,
            prune,
            prune_size,
            extra_cmd_line_options,
            db_cache,
            data_dir: data_dir.clone(),
            disable_wallet,
            zmqpubrawtx,
            zmqpubrawblock,
        }
    }

    #[test]
    fn test_bitcoin_daemon_service_defaults() {
        let default_service = BitcoinDaemonService::default();
        let default_ip = IpAddr::from_str("127.0.0.1").unwrap();
        assert!(!default_service.enable);
        assert_eq!(default_service.address, default_ip);
        assert_eq!(default_service.port, 8333);
        assert_eq!(default_service.rpc_address, default_ip);
        assert_eq!(default_service.rpc_port, 8332);
        assert_eq!(default_service.user, "admin");
        assert_eq!(default_service.data_dir, "/var/lib/bitcoind");
        assert_eq!(default_service.network, BitcoinNetwork::Mainnet);
        assert_eq!(default_service.prune, PruneOptions::Disable);
        assert_eq!(default_service.prune_size, 2000);
    }

    #[test]
    fn test_to_json_string() {
        let d = get_test_daemon();
        let json_str = d.to_json_string().unwrap();
        println!("{}", json_str);

        assert!(json_str.contains(&format!("\"enable\":{}", d.enable)));
        assert!(json_str.contains(&format!("\"address\":\"{}\"", d.address)));
        assert!(json_str.contains(&format!("\"port\":{}", d.port)));
        assert!(json_str.contains(&format!("\"network\":\"{}\"", d.network)));
        assert!(json_str.contains(&format!("\"tx_index\":{}", d.tx_index)));
        assert!(json_str.contains(&format!("\"onion_port\":{}", d.onion_port.unwrap())));
        assert!(json_str.contains(&format!("\"listen\":{}", d.listen)));
        assert!(json_str.contains(&format!("\"extra_config\":\"{}\"", d.extra_config)));
        assert!(json_str.contains(&format!("\"user\":\"{}\"", d.user)));
        let rpc_users_string = d
            .rpc_users
            .iter()
            .map(|u| {
                format!(
                    "{{\"password_hmac\":\"{}\",\"name\":\"{}\"}}",
                    u.password_hmac, u.name
                )
            })
            .collect::<Vec<_>>()
            .join(",");
        assert!(json_str.contains(&format!("\"rpc_users\":[{}]", rpc_users_string)));
        assert!(json_str.contains(&format!("\"rpc_address\":\"{}\"", d.rpc_address)));
        assert!(json_str.contains(&format!("\"rpc_port\":{}", d.rpc_port)));
        assert!(json_str
            .contains(&"\"rpc_allow_ip\":[\"192.168.1.100\",\"192.168.1.111\"]".to_string()));
        assert!(json_str.contains(&"\"prune\":{\"Automatic\":{\"prune_at\":2500}".to_string()));
        assert!(json_str.contains(&format!("\"prune_size\":{}", d.prune_size)));
        assert!(json_str
            .contains(&"\"extra_cmd_line_options\":[\"option1\",\"option2=value\"]".to_string()));
        assert!(json_str.contains(&format!("\"db_cache\":{}", d.db_cache.unwrap())));
        assert!(json_str.contains(&format!("\"data_dir\":\"{}\"", d.data_dir)));
        assert!(json_str.contains(&format!("\"disable_wallet\":{}", d.disable_wallet)));
        assert!(json_str.contains(&format!("\"zmqpubrawtx\":\"{}\"", d.zmqpubrawtx.unwrap())));
        assert!(json_str.contains(&format!(
            "\"zmqpubrawblock\":\"{}\"",
            d.zmqpubrawblock.unwrap()
        )));
    }

    #[test]
    fn test_from_json_string() {
        let source = get_test_daemon();
        let data = source.to_json_string().unwrap();

        let service = BitcoinDaemonService::from_json(&data);
        assert!(service.is_ok());
        let target = service.unwrap();
        assert!(source == target);
    }

    #[test]
    fn test_render_mainnet() {
        let d = get_test_daemon();

        let res = d.render().unwrap();
        assert!(res.contains_key(FILE_NAME));
        let nix_str = res.get(FILE_NAME).unwrap();

        assert!(nix_str.contains(&format!("enable = {};", d.enable)));
        assert!(nix_str.contains(&"regtest = false;".to_string()));
        assert!(nix_str.contains(&format!("txindex = {}", d.tx_index)));
        assert!(nix_str.contains(&format!("disablewallet = {};", d.disable_wallet)));
        assert!(nix_str.contains(&format!("listen = {};", d.listen)));
        assert!(nix_str.contains(&format!("address = \"{}\";", d.address)));
        assert!(nix_str.contains(&format!("port = {};", d.port)));
        assert!(nix_str.contains(&format!(
            r#"
    rpc = {{
      address = "{}";
      port = {};
      allowip = [
        "192.168.1.100"
        "192.168.1.111"
      ];
      users = {{
        dsfsdf = {{passwordHMAC = "rpc_user1";}};
        owieru = {{passwordHMAC = "rpc_user2";}};
      }};
    }};
"#,
            d.rpc_address, d.rpc_port
        )));
        assert!(nix_str.contains(&format!(
            "zmqpubrawblock = \"{}\";",
            d.zmqpubrawblock.unwrap()
        )));
        assert!(nix_str.contains(&format!("zmqpubrawtx = \"{}\";", d.zmqpubrawtx.unwrap())));
    }
}
