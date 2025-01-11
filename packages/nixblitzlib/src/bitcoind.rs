use core::{fmt, str};
use std::{collections::HashMap, net::IpAddr, str::FromStr};

use alejandra::format;

use cli_log::info;
use error_stack::{Report, Result, ResultExt};
use garde::{rules::AsStr, Validate};
use handlebars::{no_escape, Handlebars};
use serde::{Deserialize, Serialize};
use strum::EnumCount;

use crate::{
    app_config::AppConfig,
    app_option_data::{
        bool_data::BoolOptionData,
        net_address_data::NetAddressOptionData,
        number_data::NumberOptionData,
        option_data::{
            OptionData, OptionDataChangeNotification, OptionId, ToNixString, ToOptionId,
        },
        password_data::PasswordOptionData,
        port_data::PortOptionData,
        string_list_data::{StringListOptionData, StringListOptionItem},
        text_edit_data::TextOptionData,
    },
    apps::SupportedApps,
    errors::{ProjectError, TemplatingError},
    number_value::NumberValue,
    utils::BASE_TEMPLATE,
};

pub const TEMPLATE_FILE_NAME: &str = "src/apps/bitcoind.nix.templ";
pub const JSON_FILE_NAME: &str = "src/apps/bitcoind.json";

#[derive(Debug, Clone, Copy, EnumCount, Hash, PartialEq, Serialize, Deserialize)]
pub enum BitcoindConfigOption {
    Enable,
    Address,
    Port,
    OnionPort,
    Listen,
    ExtraConfig,
    User,
    Network,
    RpcUsers,
    RpcAddress,
    RpcPort,
    RpcAllowIp,
    Prune,
    PruneSize,
    ExtraCmdLineOptions,
    DbCache,
    DataDir,
    TxIndex,
    DisableWallet,
    ZmqPubRawTx,
    ZmqPubRawBlock,
}

impl ToOptionId for BitcoindConfigOption {
    fn to_option_id(&self) -> OptionId {
        OptionId::new(SupportedApps::BitcoinCore, self.to_string())
    }
}

impl FromStr for BitcoindConfigOption {
    type Err = ();

    fn from_str(s: &str) -> std::result::Result<BitcoindConfigOption, ()> {
        match s {
            "enable" => Ok(BitcoindConfigOption::Enable),
            "address" => Ok(BitcoindConfigOption::Address),
            "port" => Ok(BitcoindConfigOption::Port),
            "onion_port" => Ok(BitcoindConfigOption::OnionPort),
            "listen" => Ok(BitcoindConfigOption::Listen),
            "extra_config" => Ok(BitcoindConfigOption::ExtraConfig),
            "user" => Ok(BitcoindConfigOption::User),
            "network" => Ok(BitcoindConfigOption::Network),
            "rpc_users" => Ok(BitcoindConfigOption::RpcUsers),
            "rpc_address" => Ok(BitcoindConfigOption::RpcAddress),
            "rpc_port" => Ok(BitcoindConfigOption::RpcPort),
            "rpc_allow_ip" => Ok(BitcoindConfigOption::RpcAllowIp),
            "prune" => Ok(BitcoindConfigOption::Prune),
            "prune_size" => Ok(BitcoindConfigOption::PruneSize),
            "extra_cmd_line_options" => Ok(BitcoindConfigOption::ExtraCmdLineOptions),
            "db_cache" => Ok(BitcoindConfigOption::DbCache),
            "data_dir" => Ok(BitcoindConfigOption::DataDir),
            "tx_index" => Ok(BitcoindConfigOption::TxIndex),
            "disable_wallet" => Ok(BitcoindConfigOption::DisableWallet),
            "zmq_pub_raw_tx" => Ok(BitcoindConfigOption::ZmqPubRawTx),
            "zmq_pub_raw_block" => Ok(BitcoindConfigOption::ZmqPubRawBlock),
            _ => Err(()),
        }
    }
}

impl fmt::Display for BitcoindConfigOption {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let option_str = match self {
            BitcoindConfigOption::Enable => "enable",
            BitcoindConfigOption::Address => "address",
            BitcoindConfigOption::Port => "port",
            BitcoindConfigOption::OnionPort => "onion_port",
            BitcoindConfigOption::Listen => "listen",
            BitcoindConfigOption::ExtraConfig => "extra_config",
            BitcoindConfigOption::User => "user",
            BitcoindConfigOption::Network => "network",
            BitcoindConfigOption::RpcUsers => "rpcUsers",
            BitcoindConfigOption::RpcAddress => "rpc_address",
            BitcoindConfigOption::RpcPort => "rpc_port",
            BitcoindConfigOption::RpcAllowIp => "rpc_allow_ip",
            BitcoindConfigOption::Prune => "prune",
            BitcoindConfigOption::PruneSize => "prune_size",
            BitcoindConfigOption::ExtraCmdLineOptions => "extra_cmd_line_options",
            BitcoindConfigOption::DbCache => "db_cache",
            BitcoindConfigOption::DataDir => "data_dir",
            BitcoindConfigOption::TxIndex => "tx_index",
            BitcoindConfigOption::DisableWallet => "disable_wallet",
            BitcoindConfigOption::ZmqPubRawTx => "zmq_pub_raw_tx",
            BitcoindConfigOption::ZmqPubRawBlock => "zmq_pub_raw_block",
        };
        write!(f, "{}", option_str)
    }
}

/// Represents all available Bitcoin network options
#[derive(Copy, Clone, Debug, Default, Serialize, Deserialize, Eq, PartialEq)]
pub enum BitcoinNetwork {
    #[default]
    /// [default] The mainnet network
    Mainnet,

    /// The regtest network
    Regtest,
}

impl BitcoinNetwork {
    pub fn to_string_array() -> [&'static str; 2] {
        ["Mainnet", "Regtest"]
    }

    pub fn from_string(s: &str) -> Option<BitcoinNetwork> {
        match s {
            "Mainnet" => Some(BitcoinNetwork::Mainnet),
            "Regtest" => Some(BitcoinNetwork::Regtest),
            _ => None,
        }
    }
}

impl fmt::Display for BitcoinNetwork {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BitcoinNetwork::Mainnet => write!(f, "Mainnet"),
            BitcoinNetwork::Regtest => write!(f, "Regtest"),
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
impl PruneOptions {
    pub fn to_string_array() -> [&'static str; 3] {
        ["Disable", "Manual", "Automatic"]
    }
}

impl fmt::Display for PruneOptions {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PruneOptions::Disable => write!(f, "Disable"),
            PruneOptions::Manual => write!(f, "Manual"),
            PruneOptions::Automatic { prune_at } => write!(f, "Automatic({})", prune_at),
        }
    }
}

impl FromStr for PruneOptions {
    type Err = ();

    fn from_str(s: &str) -> std::result::Result<PruneOptions, ()> {
        match s {
            "Disable" => Ok(PruneOptions::Disable),
            "Manual" => Ok(PruneOptions::Manual),
            s if s.starts_with("Automatic(") && s.ends_with(")") => {
                let prune_at_str = &s[10..s.len() - 1];
                if let Ok(prune_at) = prune_at_str.parse::<u32>() {
                    Ok(PruneOptions::Automatic { prune_at })
                } else {
                    Err(())
                }
            }
            "Automatic" => Ok(PruneOptions::Automatic { prune_at: 0 }),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Clone, Copy, EnumCount, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub enum BitcoinDaemonServiceRPCUserConfigOption {
    RpsUserPasswordHmac,
    RpcUserName,
}

impl ToOptionId for BitcoinDaemonServiceRPCUserConfigOption {
    fn to_option_id(&self) -> OptionId {
        OptionId::new(SupportedApps::BitcoinCore, self.to_string())
    }
}

impl FromStr for BitcoinDaemonServiceRPCUserConfigOption {
    type Err = ();

    fn from_str(s: &str) -> std::result::Result<BitcoinDaemonServiceRPCUserConfigOption, ()> {
        match s {
            "rpc_user_password_hmac" => {
                Ok(BitcoinDaemonServiceRPCUserConfigOption::RpsUserPasswordHmac)
            }
            "rpc_user_name" => Ok(BitcoinDaemonServiceRPCUserConfigOption::RpcUserName),
            _ => Err(()),
        }
    }
}

impl fmt::Display for BitcoinDaemonServiceRPCUserConfigOption {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let option_str = match self {
            BitcoinDaemonServiceRPCUserConfigOption::RpsUserPasswordHmac => {
                "rpc_user_password_hmac"
            }
            BitcoinDaemonServiceRPCUserConfigOption::RpcUserName => "rpc_user_name",
        };
        write!(f, "{}", option_str)
    }
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct BitcoinDaemonServiceRPCUser {
    /// Password HMAC-SHA-256 for JSON-RPC connections. Must be a string of the format <SALT-HEX>$<HMAC-HEX>.

    /// Tool (Python script) for HMAC generation is available here:
    /// https://github.com/bitcoin/bitcoin/blob/master/share/rpcauth/rpcauth.py
    pub password_hmac: Box<PasswordOptionData>,

    /// Username for JSON-RPC connections.
    pub name: Box<TextOptionData>,
}

impl BitcoinDaemonServiceRPCUser {
    pub fn new(password_hmac: String, name: String) -> Self {
        Self {
            password_hmac: Box::new(PasswordOptionData::new(
                BitcoinDaemonServiceRPCUserConfigOption::RpsUserPasswordHmac.to_option_id(),
                password_hmac,
                true,
                8,
                false,
                "".into(),
            )),
            name: Box::new(TextOptionData::new(
                BitcoinDaemonServiceRPCUserConfigOption::RpcUserName.to_option_id(),
                name,
                1,
                false,
                "".into(),
            )),
        }
    }

    pub fn get_options(&self) -> Vec<OptionData> {
        vec![
            OptionData::PasswordEdit(self.password_hmac.clone()),
            OptionData::TextEdit(self.name.clone()),
        ]
    }
}

#[derive(Debug, Validate, Serialize, Deserialize, PartialEq)]
pub struct BitcoinDaemonService {
    /// Whether the service is enabled or not
    #[garde(skip)]
    pub enable: Box<BoolOptionData>,

    /// Address to listen for peer connections
    #[garde(skip)]
    pub address: Box<NetAddressOptionData>,

    /// Port to listen for peer connections.
    ///
    /// Default: mainnet 8333
    ///          regtest 18444
    #[garde(skip)]
    pub port: Box<PortOptionData>,

    /// Port to listen for Tor peer connections.
    /// If set, inbound connections to this port are tagged as onion peers.
    ///
    /// Default: None
    ///          mainnet 8334
    ///          regtest 18445
    #[garde(skip)]
    pub onion_port: Box<PortOptionData>,

    /// Listen for peer connections at `address:port`
    /// and `address:onionPort` (if {option}`onionPort` is set).
    ///
    /// Default: false
    #[garde(skip)]
    pub listen: Box<BoolOptionData>,

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
    pub extra_config: Box<TextOptionData>,

    /// The user as which to run bitcoind.
    //#[garde(length(min = 3))]
    #[garde(skip)]
    pub user: Box<TextOptionData>,

    /// Which chiain to use
    #[garde(skip)]
    pub network: Box<StringListOptionData>,

    /// Allowed users for JSON-RPC connections.
    #[garde(skip)]
    pub rpc_users: Box<Vec<BitcoinDaemonServiceRPCUser>>,

    /// Address to listen for rpc connections
    #[garde(skip)]
    pub rpc_address: Box<NetAddressOptionData>,

    /// Override the default port on which to listen for JSON-RPC connections.
    /// Default: 8332
    //#[garde(range(min = 1024, max = 65535))]
    #[garde(skip)]
    pub rpc_port: Box<PortOptionData>,

    /// Hosts that should be allowed to connect to the RPC server
    ///
    /// Example: "192.168.0.0/16"
    /// Default: None
    #[garde(skip)]
    pub rpc_allow_ip: Box<Vec<NetAddressOptionData>>,

    /// Whether to prune the node
    // #[garde(custom(_check_prune(&self)))]
    #[garde(skip)]
    pub prune: Box<StringListOptionData>,

    /// The size in MiB at which the blockchain on disk will be pruned.
    ///
    /// * Only active if prune is set to automatic
    /// * Must be at least 500 MiB
    //#[garde(range(min = 500))]
    #[garde(skip)]
    pub prune_size: Box<NumberOptionData>,

    /// Extra command line options to pass to bitcoind. Run bitcoind –help to list all available options.
    #[garde(skip)]
    pub extra_cmd_line_options: Box<TextOptionData>,

    /// Override the default database cache size in MiB.
    /// Integer between 4 and 16384 (both inclusive)
    ///
    /// Example: 4000
    /// Default: None
    //#[garde(range(min = 4, max = 16384))]
    #[garde(skip)]
    pub db_cache: Box<NumberOptionData>,

    /// The data directory for bitcoind.
    ///
    /// Default: "/var/lib/bitcoind"
    #[garde(skip)]
    pub data_dir: Box<TextOptionData>,

    /// Whether to enable the tx index
    #[garde(skip)]
    pub tx_index: Box<BoolOptionData>,

    /// Whether to enable the integrated wallet
    #[garde(skip)]
    pub disable_wallet: Box<BoolOptionData>,

    /// ZMQ address for zmqpubrawtx notifications
    ///
    /// # Example
    /// "tcp://127.0.0.1:28333"
    #[garde(skip)]
    pub zmqpubrawtx: Box<NetAddressOptionData>,

    /// ZMQ address for zmqpubrawblock notifications
    ///
    /// # Example
    /// "tcp://127.0.0.1:28332"
    #[garde(skip)]
    pub zmqpubrawblock: Box<NetAddressOptionData>,
}

impl Default for BitcoinDaemonService {
    fn default() -> Self {
        Self {
            enable: Box::new(BoolOptionData::new(
                BitcoindConfigOption::Enable.to_option_id(),
                false,
            )),
            address: Box::new(NetAddressOptionData::new(
                BitcoindConfigOption::Address.to_option_id(),
                Some(IpAddr::from_str("127.0.0.1").unwrap()),
            )),
            port: Box::new(PortOptionData::new(
                BitcoindConfigOption::Port.to_option_id(),
                NumberValue::U16(Some(8333)),
            )),
            onion_port: Box::new(PortOptionData::new(
                BitcoindConfigOption::OnionPort.to_option_id(),
                NumberValue::U16(None),
            )),
            listen: Box::new(BoolOptionData::new(
                BitcoindConfigOption::Listen.to_option_id(),
                false,
            )),
            extra_config: Box::new(TextOptionData::new(
                BitcoindConfigOption::ExtraConfig.to_option_id(),
                "".into(),
                10000,
                false,
                "".into(),
            )),
            user: Box::new(TextOptionData::new(
                BitcoindConfigOption::User.to_option_id(),
                "admin".into(),
                0,
                false,
                "".into(),
            )),
            network: Box::new(StringListOptionData::new(
                BitcoindConfigOption::Network.to_option_id(),
                BitcoinNetwork::Mainnet.to_string(),
                BitcoinNetwork::to_string_array()
                    .iter()
                    .map(|n| StringListOptionItem::new(n.to_string(), n.to_string()))
                    .collect(),
            )),
            rpc_users: Box::new(Vec::new()),
            rpc_address: Box::new(NetAddressOptionData::new(
                BitcoindConfigOption::RpcAddress.to_option_id(),
                Some(IpAddr::from_str("127.0.0.1").unwrap()),
            )),
            rpc_port: Box::new(PortOptionData::new(
                BitcoindConfigOption::RpcPort.to_option_id(),
                NumberValue::U16(Some(8332)),
            )),
            rpc_allow_ip: Box::new(Vec::new()),
            prune: Box::new(StringListOptionData::new(
                BitcoindConfigOption::Prune.to_option_id(),
                PruneOptions::Disable.to_string(),
                PruneOptions::to_string_array()
                    .iter()
                    .map(|o| StringListOptionItem::new(o.to_string(), o.to_string()))
                    .collect(),
            )),
            prune_size: Box::new(
                NumberOptionData::new(
                    BitcoindConfigOption::PruneSize.to_option_id(),
                    NumberValue::UInt(Some(2048)),
                    551,
                    99999,
                    false,
                    NumberValue::UInt(Some(2048)),
                )
                .unwrap(),
            ),
            extra_cmd_line_options: Box::new(TextOptionData::new(
                BitcoindConfigOption::ExtraCmdLineOptions.to_option_id(),
                "".to_string(),
                9999,
                false,
                "".to_string(),
            )),
            db_cache: Box::new(
                NumberOptionData::new(
                    BitcoindConfigOption::DbCache.to_option_id(),
                    NumberValue::U16(None),
                    4,
                    16384,
                    false,
                    NumberValue::U16(None),
                )
                .unwrap(),
            ),
            data_dir: Box::new(TextOptionData::new(
                BitcoindConfigOption::DataDir.to_option_id(),
                "/var/lib/bitcoind".into(),
                1,
                false,
                "".into(),
            )),
            tx_index: Box::new(BoolOptionData::new(
                BitcoindConfigOption::TxIndex.to_option_id(),
                false,
            )),
            disable_wallet: Box::new(BoolOptionData::new(
                BitcoindConfigOption::DisableWallet.to_option_id(),
                true,
            )),
            zmqpubrawtx: Box::new(NetAddressOptionData::new(
                BitcoindConfigOption::ZmqPubRawTx.to_option_id(),
                None,
            )),
            zmqpubrawblock: Box::new(NetAddressOptionData::new(
                BitcoindConfigOption::ZmqPubRawBlock.to_option_id(),
                None,
            )),
        }
    }
}

impl BitcoinDaemonService {
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
                .attach_printable(format!("File {TEMPLATE_FILE_NAME} not found in template")))
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
            (
                // nix-bitcoin only supports mainnet and regtest at the moment
                "regtest",
                (self.network.value() == BitcoinNetwork::Regtest.to_string()).to_string(),
            ),
            ("enable", self.enable.value().to_string()),
            ("tx_index", self.tx_index.value().to_string()),
            ("disable_wallet", self.disable_wallet.value().to_string()),
            ("address", self.address.to_nix_string(true)),
            ("listen", self.listen.value().to_string()),
            ("port", self.port.value().to_string_or("8333")),
            ("rpc_address", self.rpc_address.to_nix_string(true)),
            ("rpc_port", self.rpc_port.value().to_string_or("8332")),
            (
                "rpc_allow_ip",
                self.rpc_allow_ip
                    .iter()
                    .map(|s| format!("\"{}\"", s.to_nix_string(true)))
                    .collect::<Vec<_>>()
                    .join("\n"),
            ),
            (
                "rpc_users",
                self.rpc_users
                    .iter()
                    .map(|s| {
                        format!(
                            "{}={{ passwordHMAC = \"{}\"; }};",
                            s.name.value(),
                            s.password_hmac.hashed_value()
                        )
                    })
                    .collect::<Vec<_>>()
                    .join("\n"),
            ),
            ("zmqpubrawblock", self.zmqpubrawblock.to_nix_string(true)),
            ("zmqpubrawtx", self.zmqpubrawtx.to_nix_string(true)),
        ]);

        let res = handlebars
            .render(TEMPLATE_FILE_NAME, &data)
            .attach_printable("Failed to render bitcoin daemon template".to_string())
            .change_context(TemplatingError::Render)?;

        let (status, text) = format::in_memory("<convert bitcoind>".to_string(), res);

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

    pub(crate) fn from_json(json_data: &str) -> Result<BitcoinDaemonService, TemplatingError> {
        serde_json::from_str(json_data).change_context(TemplatingError::JsonLoadError)
    }

    pub fn get_options(&self) -> Vec<OptionData> {
        vec![
            OptionData::Bool(self.enable.clone()),
            OptionData::NetAddress(self.address.clone()),
            OptionData::Port(self.port.clone()),
            OptionData::Port(self.onion_port.clone()),
            OptionData::Bool(self.listen.clone()),
            OptionData::TextEdit(self.extra_config.clone()),
            OptionData::TextEdit(self.user.clone()),
            OptionData::StringList(Box::new(StringListOptionData::new(
                BitcoindConfigOption::Network.to_option_id(),
                self.network.value().to_string(),
                BitcoinNetwork::to_string_array()
                    .map(|entry| StringListOptionItem::new(entry.to_string(), entry.to_string()))
                    .to_vec(),
            ))),
            //// TODO: implement me
            ////OptionData::RpcUsers(self.rpc_users.clone()),
            OptionData::NetAddress(self.rpc_address.clone()),
            OptionData::Port(self.rpc_port.clone()),
            //// TODO: implement me
            //OptionData::IpList(self.rpc_allow_ip.clone()),
            OptionData::StringList(Box::new(StringListOptionData::new(
                BitcoindConfigOption::Prune.to_option_id(),
                self.prune.value().to_string(),
                PruneOptions::to_string_array()
                    .map(|entry| StringListOptionItem::new(entry.to_string(), entry.to_string()))
                    .to_vec(),
            ))),
            OptionData::NumberEdit(self.prune_size.clone()),
            OptionData::TextEdit(self.extra_cmd_line_options.clone()),
            OptionData::NumberEdit(self.db_cache.clone()),
            OptionData::TextEdit(self.data_dir.clone()),
            OptionData::Bool(self.tx_index.clone()),
            OptionData::Bool(self.disable_wallet.clone()),
            OptionData::NetAddress(self.zmqpubrawtx.clone()),
            OptionData::NetAddress(self.zmqpubrawblock.clone()),
        ]
    }
}

impl AppConfig for BitcoinDaemonService {
    fn app_option_changed(
        &mut self,
        id: &OptionId,
        option: &OptionDataChangeNotification,
    ) -> Result<bool, ProjectError> {
        if let Ok(opt) = BitcoindConfigOption::from_str(&id.option) {
            let mut res = Ok(false);
            if opt == BitcoindConfigOption::Enable {
                if let OptionDataChangeNotification::Bool(val) = option {
                    res = Ok(self.enable.value() != val.value);
                    self.enable.set_value(val.value);
                } else {
                    Err(Report::new(ProjectError::ChangeOptionValueError(
                        opt.to_string(),
                    )))?;
                }
            } else if opt == BitcoindConfigOption::Address {
                if let OptionDataChangeNotification::NetAddress(val) = option {
                    res = Ok(self.address.value() != val.value);
                    self.address.set_value(val.value);
                } else {
                    Err(
                        Report::new(ProjectError::ChangeOptionValueError(opt.to_string()))
                            .attach_printable(format!("{:?}", option)),
                    )?;
                }
            } else if opt == BitcoindConfigOption::Port {
                if let OptionDataChangeNotification::Port(val) = option {
                    res = Ok(*self.port.value() != val.value);
                    self.port.set_value(val.value.clone());
                } else {
                    Err(
                        Report::new(ProjectError::ChangeOptionValueError(opt.to_string()))
                            .attach_printable(format!("{:?}", option)),
                    )?;
                }
            } else if opt == BitcoindConfigOption::OnionPort {
                if let OptionDataChangeNotification::Port(val) = option {
                    res = Ok(*self.onion_port.value() != val.value);
                    self.onion_port.set_value(val.value.clone());
                } else {
                    Err(
                        Report::new(ProjectError::ChangeOptionValueError(opt.to_string()))
                            .attach_printable(format!("{:?}", option)),
                    )?;
                }
            } else if opt == BitcoindConfigOption::Listen {
                if let OptionDataChangeNotification::Bool(val) = option {
                    res = Ok(self.listen.value() != val.value);
                    self.listen.set_value(val.value);
                } else {
                    Err(
                        Report::new(ProjectError::ChangeOptionValueError(opt.to_string()))
                            .attach_printable(format!("{:?}", option)),
                    )?;
                }
            } else if opt == BitcoindConfigOption::ExtraConfig {
                if let OptionDataChangeNotification::TextEdit(val) = option {
                    res = Ok(self.extra_config.value() != val.value);
                    self.extra_config.set_value(val.value.clone());
                } else {
                    Err(
                        Report::new(ProjectError::ChangeOptionValueError(opt.to_string()))
                            .attach_printable(format!("{:?}", option)),
                    )?;
                }
            } else if opt == BitcoindConfigOption::User {
                if let OptionDataChangeNotification::TextEdit(val) = option {
                    res = Ok(self.user.value() != val.value);
                    self.user.set_value(val.value.clone());
                } else {
                    Err(
                        Report::new(ProjectError::ChangeOptionValueError(opt.to_string()))
                            .attach_printable(format!("{:?}", option)),
                    )?;
                }
            } else if opt == BitcoindConfigOption::Network {
                if let OptionDataChangeNotification::StringList(val) = option {
                    if BitcoinNetwork::from_string(val.value.as_str()).is_none() {
                        Err(Report::new(ProjectError::ChangeOptionValueError(
                            BitcoindConfigOption::Network.to_string(),
                        ))
                        .attach_printable(format!("{:?}", option)))?
                    }
                    res = Ok(self.network.value() != val.value);
                    self.network.set_value(val.value.clone());
                } else {
                    Err(
                        Report::new(ProjectError::ChangeOptionValueError(opt.to_string()))
                            .attach_printable(format!("{:?}", option)),
                    )?;
                }
            } else if opt == BitcoindConfigOption::RpcAddress {
                if let OptionDataChangeNotification::NetAddress(val) = option {
                    res = Ok(self.rpc_address.value() != val.value);
                    self.rpc_address.set_value(val.value);
                } else {
                    Err(
                        Report::new(ProjectError::ChangeOptionValueError(opt.to_string()))
                            .attach_printable(format!("{:?}", option)),
                    )?;
                }
            } else if opt == BitcoindConfigOption::RpcPort {
                if let OptionDataChangeNotification::Port(val) = option {
                    res = Ok(*self.port.value() != val.value);
                    self.port.set_value(val.value.clone());
                } else {
                    Err(
                        Report::new(ProjectError::ChangeOptionValueError(opt.to_string()))
                            .attach_printable(format!("{:?}", option)),
                    )?;
                }
            } else if opt == BitcoindConfigOption::Prune {
                if let OptionDataChangeNotification::StringList(val) = option {
                    if PruneOptions::from_str(val.value.as_str()).is_err() {
                        Err(
                            Report::new(ProjectError::ChangeOptionValueError(opt.to_string()))
                                .attach_printable(format!("{:?}", option)),
                        )?
                    }
                    res = Ok(self.prune.value() != val.value);
                    self.prune.set_value(val.value.clone());
                } else {
                    Err(
                        Report::new(ProjectError::ChangeOptionValueError(opt.to_string()))
                            .attach_printable(format!("{:?}", option)),
                    )?;
                }
            } else if opt == BitcoindConfigOption::PruneSize {
                if let OptionDataChangeNotification::Number(val) = option {
                    res = Ok(*self.prune_size.value() != val.value);
                    self.prune_size.set_value(val.value.clone());
                } else {
                    Err(
                        Report::new(ProjectError::ChangeOptionValueError(opt.to_string()))
                            .attach_printable(format!("{:?}", option)),
                    )?;
                }
            } else if opt == BitcoindConfigOption::ExtraCmdLineOptions {
                if let OptionDataChangeNotification::TextEdit(val) = option {
                    res = Ok(self.extra_cmd_line_options.value() != val.value);
                    self.extra_cmd_line_options.set_value(val.value.clone());
                } else {
                    Err(
                        Report::new(ProjectError::ChangeOptionValueError(opt.to_string()))
                            .attach_printable(format!("{:?}", option)),
                    )?;
                }
            } else if opt == BitcoindConfigOption::DbCache {
                if let OptionDataChangeNotification::Number(val) = option {
                    res = Ok(*self.db_cache.value() != val.value);
                    self.db_cache.set_value(val.value.clone());
                } else {
                    Err(
                        Report::new(ProjectError::ChangeOptionValueError(opt.to_string()))
                            .attach_printable(format!("{:?}", option)),
                    )?;
                }
            } else if opt == BitcoindConfigOption::DataDir {
                if let OptionDataChangeNotification::TextEdit(val) = option {
                    res = Ok(self.data_dir.value() != val.value);
                    self.data_dir.set_value(val.value.clone());
                } else {
                    Err(
                        Report::new(ProjectError::ChangeOptionValueError(opt.to_string()))
                            .attach_printable(format!("{:?}", option)),
                    )?;
                }
            } else if opt == BitcoindConfigOption::TxIndex {
                if let OptionDataChangeNotification::Bool(val) = option {
                    res = Ok(self.tx_index.value() != val.value);
                    info!("txindex: {:?}", res);
                    self.tx_index.set_value(val.value);
                } else {
                    Err(
                        Report::new(ProjectError::ChangeOptionValueError(opt.to_string()))
                            .attach_printable(format!("{:?}", option)),
                    )?;
                }
            } else if opt == BitcoindConfigOption::DisableWallet {
                if let OptionDataChangeNotification::Bool(val) = option {
                    res = Ok(self.disable_wallet.value() != val.value);
                    info!("disablewallet: {:?}", res);
                    self.disable_wallet.set_value(val.value);
                } else {
                    Err(
                        Report::new(ProjectError::ChangeOptionValueError(opt.to_string()))
                            .attach_printable(format!("{:?}", option)),
                    )?;
                }
            } else if opt == BitcoindConfigOption::ZmqPubRawTx {
                if let OptionDataChangeNotification::NetAddress(val) = option {
                    res = Ok(self.zmqpubrawtx.value() != val.value);
                    self.zmqpubrawtx.set_value(val.value);
                } else {
                    Err(
                        Report::new(ProjectError::ChangeOptionValueError(opt.to_string()))
                            .attach_printable(format!("{:?}", option)),
                    )?;
                }
            } else if opt == BitcoindConfigOption::ZmqPubRawBlock {
                if let OptionDataChangeNotification::NetAddress(val) = option {
                    res = Ok(self.zmqpubrawblock.value() != val.value);
                    self.zmqpubrawblock.set_value(val.value);
                } else {
                    Err(
                        Report::new(ProjectError::ChangeOptionValueError(opt.to_string()))
                            .attach_printable(format!("{:?}", option)),
                    )?;
                }
            } else {
                Err(
                    Report::new(ProjectError::ChangeOptionValueError(opt.to_string()))
                        .attach_printable(format!("Unknown option: {}", opt,)),
                )?
            }

            return res;
        }

        Ok(false)
    }
}

//#[cfg(test)]
//pub mod tests {
//    use super::*;
//
//    fn get_test_daemon() -> BitcoinDaemonService {
//        let enable = true;
//        let address = IpAddr::from_str("127.0.0.1").unwrap();
//        let port = 8333;
//        let network = BitcoinNetwork::Testnet;
//        let tx_index = true;
//        let onion_port = Some(1551);
//        let listen = false;
//        let extra_config = "extra_config_value".to_string();
//        let user = "user_name".to_string();
//        let rpc_users = vec![
//            BitcoinDaemonServiceRPCUser::new("rpc_user1".into(), "dsfsdf".into()),
//            BitcoinDaemonServiceRPCUser::new("rpc_user2".into(), "owieru".into()),
//        ];
//
//        let rpc_address = IpAddr::from_str("128.22.22.4").unwrap();
//        let rpc_port = 8332;
//        let rpc_allow_ip: Vec<IpAddr> = vec![
//            IpAddr::from_str("192.168.1.100").unwrap(),
//            IpAddr::from_str("192.168.1.111").unwrap(),
//        ];
//        let prune = PruneOptions::Automatic { prune_at: 2500 };
//        let prune_size = 500;
//        let extra_cmd_line_options: Vec<String> =
//            vec!["option1".to_string(), "option2=value".to_string()];
//        let db_cache = Some(2048);
//        let data_dir = "/path/to/data/dir".to_string();
//        let disable_wallet = true;
//        let zmqpubrawtx = Some("zmqpubrawtx_test".to_string());
//        let zmqpubrawblock = Some("zmqpubrawblock_test".to_string());
//
//        BitcoinDaemonService {
//            enable,
//            address,
//            port,
//            network,
//            tx_index,
//            onion_port,
//            listen,
//            extra_config: extra_config.clone(),
//            user: user.clone(),
//            rpc_users,
//            rpc_address,
//            rpc_port,
//            rpc_allow_ip,
//            prune,
//            prune_size,
//            extra_cmd_line_options,
//            db_cache,
//            data_dir: data_dir.clone(),
//            disable_wallet,
//            zmqpubrawtx,
//            zmqpubrawblock,
//        }
//    }
//
//    #[test]
//    fn test_bitcoin_daemon_service_defaults() {
//        let default_service = BitcoinDaemonService::default();
//        let default_ip = IpAddr::from_str("127.0.0.1").unwrap();
//        assert!(!default_service.enable);
//        assert_eq!(default_service.address, default_ip);
//        assert_eq!(default_service.port, 8333);
//        assert_eq!(default_service.rpc_address, default_ip);
//        assert_eq!(default_service.rpc_port, 8332);
//        assert_eq!(default_service.user, "admin");
//        assert_eq!(default_service.data_dir, "/var/lib/bitcoind");
//        assert_eq!(default_service.network, BitcoinNetwork::Mainnet);
//        assert_eq!(default_service.prune, PruneOptions::Disable);
//        assert_eq!(default_service.prune_size, 2000);
//    }
//
//    #[test]
//    fn test_to_json_string() {
//        let d = get_test_daemon();
//        let json_str = d.to_json_string().unwrap();
//        println!("{}", json_str);
//
//        assert!(json_str.contains(&format!("\"enable\":{}", d.enable)));
//        assert!(json_str.contains(&format!("\"address\":\"{}\"", d.address)));
//        assert!(json_str.contains(&format!("\"port\":{}", d.port)));
//        assert!(json_str.contains(&format!("\"network\":\"{}\"", d.network)));
//        assert!(json_str.contains(&format!("\"tx_index\":{}", d.tx_index)));
//        assert!(json_str.contains(&format!("\"onion_port\":{}", d.onion_port.unwrap())));
//        assert!(json_str.contains(&format!("\"listen\":{}", d.listen)));
//        assert!(json_str.contains(&format!("\"extra_config\":\"{}\"", d.extra_config)));
//        assert!(json_str.contains(&format!("\"user\":\"{}\"", d.user)));
//        let rpc_users_string = d
//            .rpc_users
//            .iter()
//            .map(|u| {
//                format!(
//                    "{{\"password_hmac\":\"{}\",\"name\":\"{}\"}}",
//                    u.password_hmac, u.name
//                )
//            })
//            .collect::<Vec<_>>()
//            .join(",");
//        assert!(json_str.contains(&format!("\"rpc_users\":[{}]", rpc_users_string)));
//        assert!(json_str.contains(&format!("\"rpc_address\":\"{}\"", d.rpc_address)));
//        assert!(json_str.contains(&format!("\"rpc_port\":{}", d.rpc_port)));
//        assert!(json_str
//            .contains(&"\"rpc_allow_ip\":[\"192.168.1.100\",\"192.168.1.111\"]".to_string()));
//        assert!(json_str.contains(&"\"prune\":{\"Automatic\":{\"prune_at\":2500}".to_string()));
//        assert!(json_str.contains(&format!("\"prune_size\":{}", d.prune_size)));
//        assert!(json_str
//            .contains(&"\"extra_cmd_line_options\":[\"option1\",\"option2=value\"]".to_string()));
//        assert!(json_str.contains(&format!("\"db_cache\":{}", d.db_cache.unwrap())));
//        assert!(json_str.contains(&format!("\"data_dir\":\"{}\"", d.data_dir)));
//        assert!(json_str.contains(&format!("\"disable_wallet\":{}", d.disable_wallet)));
//        assert!(json_str.contains(&format!("\"zmqpubrawtx\":\"{}\"", d.zmqpubrawtx.unwrap())));
//        assert!(json_str.contains(&format!(
//            "\"zmqpubrawblock\":\"{}\"",
//            d.zmqpubrawblock.unwrap()
//        )));
//    }
//
//    #[test]
//    fn test_from_json_string() {
//        let source = get_test_daemon();
//        let data = source.to_json_string().unwrap();
//
//        let service = BitcoinDaemonService::from_json(&data);
//        assert!(service.is_ok());
//        let target = service.unwrap();
//        assert!(source == target);
//    }
//
//    #[test]
//    fn test_render_mainnet() {
//        let d = get_test_daemon();
//
//        let res = d.render().unwrap();
//        assert!(res.contains_key(TEMPLATE_FILE_NAME));
//        let nix_str = res.get(TEMPLATE_FILE_NAME).unwrap();
//
//        assert!(nix_str.contains(&format!("enable = {};", d.enable)));
//        assert!(nix_str.contains(&"regtest = false;".to_string()));
//        assert!(nix_str.contains(&format!("txindex = {}", d.tx_index)));
//        assert!(nix_str.contains(&format!("disablewallet = {};", d.disable_wallet)));
//        assert!(nix_str.contains(&format!("listen = {};", d.listen)));
//        assert!(nix_str.contains(&format!("address = \"{}\";", d.address)));
//        assert!(nix_str.contains(&format!("port = {};", d.port)));
//        assert!(nix_str.contains(&format!(
//            r#"
//    rpc = {{
//      address = "{}";
//      port = {};
//      allowip = [
//        "192.168.1.100"
//        "192.168.1.111"
//      ];
//      users = {{
//        dsfsdf = {{passwordHMAC = "rpc_user1";}};
//        owieru = {{passwordHMAC = "rpc_user2";}};
//      }};
//    }};
//"#,
//            d.rpc_address, d.rpc_port
//        )));
//        assert!(nix_str.contains(&format!(
//            "zmqpubrawblock = \"{}\";",
//            d.zmqpubrawblock.unwrap()
//        )));
//        assert!(nix_str.contains(&format!("zmqpubrawtx = \"{}\";", d.zmqpubrawtx.unwrap())));
//    }
//}
