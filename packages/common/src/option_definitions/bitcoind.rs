use core::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};
use strum_macros::EnumCount;

use crate::{
    app_option_data::option_data::{OptionId, ToOptionId},
    apps::SupportedApps,
};

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
/// use common::option_definitions::bitcoind::PruneOptions;
///
/// let options = PruneOptions::Automatic { prune_at: 1024 };
/// ```
#[derive(Debug, Default, Serialize, Deserialize, Eq, PartialEq)]
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
