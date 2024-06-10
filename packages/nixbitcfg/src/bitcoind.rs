use core::str;

use alejandra::format;

use crate::utils::{self};
use garde::Validate;
use serde::{Deserialize, Serialize};

/// Represents all available Bitcoin network options
#[derive(Copy, Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
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

/// Prune options for a blockchain.
///
/// This enum defines the different pruning strategies that can be used.
/// # Variants
/// - Disable: Pruning is disabled. This is the default option.
/// - Manual: Pruning is performed manually by the user.
/// - Automatic: Pruning is performed automatically when the blockchain reaches a certain size. ///
/// # Examples
/// ```
/// use nixbitcoin_config::bitcoind::PruneOptions;
///
/// let options = PruneOptions::Automatic { prune_at: 1024 };
/// ```
#[derive(Debug, Validate, Default, Serialize, Deserialize, PartialEq)]
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

#[derive(Debug, Validate, Serialize, Deserialize)]
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

#[derive(Debug, Validate, Serialize, Deserialize, Default)]
pub struct BitcoinDaemonService {
    /// Whether the service is enabled or not
    #[garde(skip)]
    pub enabled: bool,

    /// The name of the instance.
    #[garde(skip)]
    pub name: Option<String>,

    /// The user as which to run bitcoind.
    #[garde(length(min = 3))]
    pub user: Option<String>,

    /// Whether to use the testnet instead of mainnet.
    #[garde(skip)]
    pub network: BitcoinNetwork,

    /// RPC user information for JSON-RPC connections.
    #[garde(skip)]
    pub rpc_users: Vec<BitcoinDaemonServiceRPCUser>,

    /// Override the default port on which to listen for JSON-RPC connections.
    #[garde(range(min = 1024, max = 65535))]
    pub rpc_port: Option<u16>,

    /// Whether to prune the node
    // #[garde(custom(_check_prune(&self)))]
    #[garde(skip)]
    pub prune: Option<PruneOptions>,

    /// The size in MiB at which the blockchain on disk will be pruned.
    ///
    /// * Only active if prune is set to automatic
    /// * Must be at least 1 MiB
    #[garde(range(min = 1))]
    pub prune_size: Option<u16>,

    /// Override the default port on which to listen for connections.
    #[garde(range(min = 1024, max = 65535))]
    pub port: Option<u16>,

    /// The bitcoind package to use.
    #[garde(skip)]
    pub package: Option<String>,

    /// The group ta which to run bitcoind.
    #[garde(skip)]
    pub group: Option<String>,

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
    pub extra_config: Option<String>,

    /// Extra command line options to pass to bitcoind. Run bitcoind –help to list all available options.
    #[garde(skip)]
    pub extra_cmd_line_options: Option<Vec<String>>,

    /// Override the default database cache size in MiB.
    /// Integer between 4 and 16384 (both inclusive)
    #[garde(range(min = 4, max = 16384))]
    pub db_cache: Option<i16>,

    /// The data directory for bitcoind.
    #[garde(skip)]
    pub data_dir: Option<String>,

    /// Whether to enable the tx index
    #[garde(skip)]
    pub tx_index: Option<bool>,

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

pub(crate) fn clean_string(value: &str) -> Result<String, String> {
    if value.starts_with('"') && value.ends_with('"') {
        return Ok(value[1..value.len() - 1].to_string());
    }

    if value.starts_with("''") && value.ends_with("''") {
        return Ok(value[2..value.len() - 2].to_string());
    }

    Err(format!(
        "Unable to detemine string type of string: {}",
        value
    ))
}

fn parse_multiline_string(beginn: usize, data: &str) -> Result<String, String> {
    let mut new_str = String::from("");
    for line in data.lines().skip(beginn + 1) {
        if line.contains("'';") {
            return Ok(new_str.trim().to_string());
        }
        new_str.push_str(format!("{}\n", line.trim()).as_str());
    }

    Err("Unable to parse String at".to_string())
}

impl BitcoinDaemonService {
    pub fn from_string(data: &str) -> Result<BitcoinDaemonService, String> {
        let mut config = BitcoinDaemonService::default();

        let lines = data.lines();
        for (n, line) in lines.enumerate() {
            let new_line = &line
                .replace("services.bitcoin.", "")
                .replace(';', "")
                .trim()
                .to_string();

            if let Some((key, value)) = new_line.split_once('=') {
                match key.trim() {
                    "enable" => match value.trim().parse() {
                        Ok(parsed_value) => config.enabled = parsed_value,
                        Err(_) => return Err(format!("Unable to parse line: {}", line)),
                    },
                    "name" => match clean_string(value.trim()) {
                        Ok(cleaned) => config.name = Some(cleaned),
                        Err(msg) => return Err(msg),
                    },
                    "user" => match clean_string(value.trim()) {
                        Ok(cleaned) => config.user = Some(cleaned),
                        Err(msg) => return Err(msg),
                    },
                    "group" => match clean_string(value.trim()) {
                        Ok(cleaned) => config.group = Some(cleaned),
                        Err(msg) => return Err(msg),
                    },
                    "package" => match clean_string(value.trim()) {
                        Ok(cleaned) => config.package = Some(cleaned),
                        Err(msg) => return Err(msg),
                    },
                    "port" => match value.trim().parse() {
                        Ok(parsed_value) => config.port = Some(parsed_value),
                        Err(_) => return Err(format!("Unable to parse line: {}", line)),
                    },
                    "txindex" => match value.trim().parse() {
                        Ok(parsed_value) => config.tx_index = Some(parsed_value),
                        Err(_) => return Err(format!("Unable to parse line: {}", line)),
                    },
                    "dbCache" => match value.trim().parse() {
                        Ok(parsed_value) => config.db_cache = Some(parsed_value),
                        Err(_) => return Err(format!("Unable to parse line: {}", line)),
                    },
                    "dataDir" => match clean_string(value.trim()) {
                        Ok(cleaned) => config.data_dir = Some(cleaned),
                        Err(msg) => return Err(msg),
                    },
                    "rpc.port" => match value.trim().parse() {
                        Ok(parsed_value) => config.rpc_port = Some(parsed_value),
                        Err(_) => return Err(format!("Unable to parse line: {}", line)),
                    },

                    "mainnet" => match value.trim().parse() {
                        Ok(true) => config.network = BitcoinNetwork::Mainnet,
                        Ok(false) => {}
                        Err(_) => return Err(format!("Unable to parse line: {}", line)),
                    },
                    "testnet" => match value.trim().parse() {
                        Ok(true) => config.network = BitcoinNetwork::Testnet,
                        Ok(false) => {}
                        Err(_) => return Err(format!("Unable to parse line: {}", line)),
                    },
                    "regtest" => match value.trim().parse() {
                        Ok(true) => config.network = BitcoinNetwork::Regtest,
                        Ok(false) => {}
                        Err(_) => return Err(format!("Unable to parse line: {}", line)),
                    },
                    "signet" => match value.trim().parse() {
                        Ok(true) => config.network = BitcoinNetwork::Signet,
                        Ok(false) => {}
                        Err(_) => return Err(format!("Unable to parse line: {}", line)),
                    },
                    "zmqpubrawtx" => match clean_string(value.trim()) {
                        Ok(cleaned) => config.zmqpubrawtx = Some(cleaned),
                        Err(msg) => return Err(msg),
                    },
                    "zmqpubrawblock" => match clean_string(value.trim()) {
                        Ok(cleaned) => config.zmqpubrawblock = Some(cleaned),
                        Err(msg) => return Err(msg),
                    },
                    "extraConfig" => match parse_multiline_string(n, data) {
                        Ok(cleaned) => config.extra_config = Some(cleaned),
                        Err(msg) => return Err(msg),
                    },

                    _ => continue,
                }
            }
        }

        Ok(config)
    }

    pub fn render(&self) -> (format::Status, String) {
        if let Err(e) = self.validate(&()) {
            panic!("invalid config: {e}")
        }

        let mut res = utils::AutoLineString::from("{");
        match self.enabled {
            true => res.push_line("services.bitcoin.enable = true;"),
            false => res.push_line("services.bitcoin.enable = false;"),
        };

        if let Some(v) = &self.name {
            res.push_line(format!("services.bitcoin.name = \"{}\";", v).as_str())
        }

        if let Some(v) = &self.group {
            res.push_line(format!("services.bitcoin.group = \"{}\";", v).as_str())
        }

        if let Some(v) = &self.package {
            res.push_line(format!("services.bitcoin.package = \"{}\";", v).as_str())
        }

        match self.network {
            // we could omit mainnet as it is default, but we'll add the field for clarity
            BitcoinNetwork::Mainnet => res.push_line("services.bitcoin.mainnet = true;"),
            BitcoinNetwork::Testnet => res.push_line("services.bitcoin.testnet = true;"),
            BitcoinNetwork::Regtest => res.push_line("services.bitcoin.regtest = true;"),
            BitcoinNetwork::Signet => res.push_line("services.bitcoin.signet = true;"),
        };

        if !self.rpc_users.is_empty() {
            res.push_line("services.bitcoin.rpc.users = {");
            for user in &self.rpc_users {
                res.push_line(format!("{} = {{", user.name).as_str());
                res.push_line(format!("passwordHMAC = \"{}\";", user.password_hmac).as_str());
                res.push_line("};");
            }
            res.push_line("};");
        }

        if let Some(v) = &self.rpc_port {
            res.push_line(format!("services.bitcoin.rpc.port = {};", v).as_str());
        }

        if let Some(v) = &self.user {
            res.push_line(format!("services.bitcoin.user = \"{}\";", v).as_str());
        }

        if let Some(v) = &self.prune {
            match v {
                PruneOptions::Disable => {}
                PruneOptions::Manual => res.push_line("services.bitcoin.prune = 1;"),
                PruneOptions::Automatic { prune_at: field } => {
                    res.push_line(format!("services.bitcoin.prune = {};", field).as_str())
                }
            }
        }

        if let Some(v) = &self.port {
            res.push_line(format!("services.bitcoin.port = {};", v).as_str())
        }

        if let Some(v) = &self.db_cache {
            res.push_line(format!("services.bitcoin.dbCache = {};", v).as_str())
        }

        if let Some(v) = &self.tx_index {
            res.push_line(format!("services.bitcoin.txindex = {};", v).as_str())
        }

        if let Some(v) = &self.zmqpubrawblock {
            res.push_line(format!("services.bitcoin.zmqpubrawblock = \"{}\";", v.trim()).as_str())
        }

        if let Some(v) = &self.zmqpubrawtx {
            res.push_line(format!("services.bitcoin.zmqpubrawtx = \"{}\";", v.trim()).as_str())
        }

        if let Some(v) = &self.data_dir {
            res.push_line(format!("services.bitcoin.dataDir = \"{}\";", v).as_str())
        }

        if let Some(v) = &self.extra_config {
            res.push_line(format!("services.bitcoin.extraConfig = ''\n{}\n'';", v.trim()).as_str())
        }

        res.push_line("}");
        format::in_memory("<convert bitcoind>".to_string(), res.to_string())
    }
}

fn _get_network_test_string(network: &BitcoinNetwork) -> String {
    let (status, res) = format::in_memory(
        "".to_string(),
        match network {
            BitcoinNetwork::Mainnet => {
                "{ services.bitcoin.enable = true; services.bitcoin.mainnet = true; }".to_string()
            }
            BitcoinNetwork::Testnet => {
                "{ services.bitcoin.enable = true; services.bitcoin.testnet = true; }".to_string()
            }
            BitcoinNetwork::Regtest => {
                "{ services.bitcoin.enable = true; services.bitcoin.regtest = true; }".to_string()
            }
            BitcoinNetwork::Signet => {
                "{ services.bitcoin.enable = true; services.bitcoin.signet = true; }".to_string()
            }
        },
    );

    match status {
        format::Status::Error(err) => eprintln!("Unable to format {}", err),
        format::Status::Changed(_) => {}
    }

    res
}

#[cfg(test)]
mod tests {
    use garde::rules::contains::Contains;

    use crate::bitcoind::_get_network_test_string;

    use super::*;

    #[test]
    fn test_bitcoin_daemon_service_creation() {
        let service = BitcoinDaemonService {
            name: Some("TestInstance".to_string()),
            user: Some("testuser".to_string()),
            port: Some(8333),
            rpc_port: Some(9333),
            ..BitcoinDaemonService::default()
        };

        assert!(!service.enabled);
        assert_eq!(service.name, Some("TestInstance".to_string()));
        assert_eq!(service.user, Some("testuser".to_string()));
        assert_eq!(service.rpc_port, Some(9333));
        assert_eq!(service.port, Some(8333));
        assert_eq!(service.network, BitcoinNetwork::Mainnet);
        assert!(service.rpc_users.is_empty());
        assert!(service.prune.is_none());
        assert!(service.package.is_none());
        assert!(service.group.is_none());
        assert!(service.extra_config.is_none());
        assert!(service.extra_cmd_line_options.is_none());
        assert!(service.db_cache.is_none());
        assert!(service.data_dir.is_none());
    }

    #[test]
    fn test_render_full() {
        let service = BitcoinDaemonService {
            name: Some("TestInstance".to_string()),
            user: Some("testuser".to_string()),
            port: Some(8333),
            rpc_port: Some(9333),
            prune: Some(PruneOptions::Automatic { prune_at: 1024 }),
            rpc_users: vec![
                BitcoinDaemonServiceRPCUser {
                    name: "testuser1".to_string(),
                    password_hmac: "".to_string(),
                },
                BitcoinDaemonServiceRPCUser {
                    name: "testuser2".to_string(),
                    password_hmac: "".to_string(),
                },
            ],
            ..BitcoinDaemonService::default()
        };
        let (format_status, nix_file) = service.render();
        match format_status {
            format::Status::Error(err) => eprintln!("Unable to format {}", err),
            format::Status::Changed(_) => println!("Format success"),
        }

        println!("{}", nix_file);

        // assert!(false);
    }

    #[test]
    fn test_prune_default_options() {
        let service = BitcoinDaemonService {
            prune: None,
            ..BitcoinDaemonService::default()
        };

        let res = service.validate(&());
        assert!(res.is_ok());
        let nix = service.render().1;
        assert!(!nix.validate_contains("prune ="));
    }

    #[test]
    fn test_prune_manual_options() {
        let service = BitcoinDaemonService {
            prune: Some(PruneOptions::Manual),
            ..BitcoinDaemonService::default()
        };

        let res = service.validate(&());
        assert!(res.is_ok());
        let nix = service.render().1;
        assert!(nix.validate_contains("prune = 1;"));
    }

    #[test]
    fn test_read() {
        let orig_service = BitcoinDaemonService {
            enabled: true,
            name: Some("TestInstance".to_string()),
            user: Some("TestUser".to_string()),
            group: Some("TestGroup".to_string()),
            port: Some(8343),
            rpc_port: Some(9343),
            package: Some("TestPackageName".to_string()),
            db_cache: Some(512),
            tx_index: Some(true),
            data_dir: Some("/data/dir".to_string()),
            zmqpubrawtx: Some("zmqpubrawtx".to_string()),
            zmqpubrawblock: Some("zmqpubrawblock".to_string()),
            extra_config: Some(
                "
setting1=2143
setting2=4534
"
                .to_string(),
            ),
            ..BitcoinDaemonService::default()
        };

        let (status, source) = orig_service.render();
        assert!(!matches!(status, format::Status::Error(_)));
        println!("{}", source);
        let res = BitcoinDaemonService::from_string(&source);
        match res {
            Ok(read_service) => {
                assert!(read_service.enabled, "Testing whether enabled is read.");
                assert!(
                    read_service.name.is_some(),
                    "Testing whether name is Some()"
                );
                assert_eq!(
                    read_service.name.as_ref().unwrap(),
                    orig_service.name.as_ref().unwrap(),
                    "Testing whether the correct name is being read."
                );
                assert!(
                    read_service.user.is_some(),
                    "Testing whether user is Some()"
                );
                assert_eq!(
                    read_service.user.as_ref().unwrap(),
                    orig_service.user.as_ref().unwrap(),
                    "Testing whether user is being read."
                );
                assert!(
                    read_service.group.is_some(),
                    "Testing whether group is Some()"
                );
                assert_eq!(
                    read_service.group.as_ref().unwrap(),
                    orig_service.group.as_ref().unwrap(),
                    "Testing whether group is being read."
                );
                assert!(
                    read_service.port.is_some(),
                    "Testing whether port is Some()"
                );
                assert_eq!(
                    read_service.port.as_ref().unwrap(),
                    orig_service.port.as_ref().unwrap(),
                    "Testing whether port is being read."
                );
                assert!(
                    read_service.rpc_port.is_some(),
                    "Testing whether rpc_port is Some()"
                );
                assert_eq!(
                    read_service.rpc_port.as_ref().unwrap(),
                    orig_service.rpc_port.as_ref().unwrap(),
                    "Testing whether port is being read."
                );
                assert!(
                    read_service.package.is_some(),
                    "Testing whether package is Some()"
                );
                assert_eq!(
                    read_service.package.as_ref().unwrap(),
                    orig_service.package.as_ref().unwrap(),
                    "Testing whether port is being read."
                );
                assert!(
                    read_service.db_cache.is_some(),
                    "Testing whether db_cache is Some()"
                );
                assert_eq!(
                    read_service.db_cache.as_ref().unwrap(),
                    orig_service.db_cache.as_ref().unwrap(),
                    "Testing whether port is being read."
                );
                assert!(
                    read_service.tx_index.is_some(),
                    "Testing whether tx_index is Some()"
                );
                assert_eq!(
                    read_service.tx_index.as_ref().unwrap(),
                    orig_service.tx_index.as_ref().unwrap(),
                    "Testing whether port is being read."
                );
                assert!(
                    read_service.data_dir.is_some(),
                    "Testing whether data_dir is Some()"
                );
                assert_eq!(
                    read_service.data_dir.as_ref().unwrap(),
                    orig_service.data_dir.as_ref().unwrap(),
                    "Testing whether port is being read."
                );
                assert!(
                    read_service.zmqpubrawtx.is_some(),
                    "Testing whether zmqpubrawtx is Some()"
                );
                assert_eq!(
                    read_service.zmqpubrawtx.as_ref().unwrap(),
                    orig_service.zmqpubrawtx.as_ref().unwrap(),
                    "Testing whether port is being read."
                );
                assert!(
                    read_service.zmqpubrawblock.is_some(),
                    "Testing whether zmqpubrawblock is Some()"
                );
                assert_eq!(
                    read_service.zmqpubrawblock.as_ref().unwrap(),
                    orig_service.zmqpubrawblock.as_ref().unwrap(),
                    "Testing whether port is being read."
                );
                assert!(
                    read_service.extra_config.is_some(),
                    "Testing whether extra_config is Some()"
                );
                assert_eq!(
                    read_service.extra_config.as_ref().unwrap().trim(),
                    orig_service.extra_config.as_ref().unwrap().trim(),
                    "Testing whether port is being read."
                );
            }
            Err(err) => panic!("Parsing err: {err}"),
        }
    }

    // #[test]
    // fn test_read_rpc_users() {
    //     let service = BitcoinDaemonService {
    //         rpc_users: vec![
    //             BitcoinDaemonServiceRPCUser {
    //                 name: "testuser1".to_string(),
    //                 password_hmac: "".to_string(),
    //             },
    //             BitcoinDaemonServiceRPCUser {
    //                 name: "testuser2".to_string(),
    //                 password_hmac: "".to_string(),
    //             },
    //         ],
    //         ..BitcoinDaemonService::default()
    //     };
    //
    //     let (status, source) = service.render();
    //     assert!(!matches!(status, format::Status::Error(_)));
    //     let res = BitcoinDaemonService::from_string(&source);
    //     match res {
    //         Ok(v) => assert!(v.enabled),
    //         Err(err) => panic!("Parsing err: {err}"),
    //     }
    // }

    #[test]
    fn test_read_network() {
        let mut service = BitcoinDaemonService::default();

        for network in [
            BitcoinNetwork::Mainnet,
            BitcoinNetwork::Testnet,
            BitcoinNetwork::Regtest,
            BitcoinNetwork::Signet,
        ] {
            service.network = network;
            let (status, source) = service.render();
            assert!(!matches!(status, format::Status::Error(_)));
            let res = BitcoinDaemonService::from_string(&source);
            match res {
                Ok(v) => assert_eq!(v.network, network),
                Err(err) => panic!("Parsing err: {err}"),
            }
        }
    }

    // #[test]
    // fn test_render_prune_automatic_options() {
    //     let service = BitcoinDaemonService {
    //         prune: Some(PruneOptions::Automatic { prune_at: 1024 }),
    //         ..BitcoinDaemonService::default()
    //     };
    //
    //     let res = service.validate(&());
    //     assert!(res.is_ok());
    //     let nix = service.render().1;
    //     assert!(nix.validate_contains("prune = 1024;"));
    //
    //     let service_nok = BitcoinDaemonService {
    //         prune: Some(PruneOptions::Automatic { prune_at: 234 }),
    //         ..BitcoinDaemonService::default()
    //     };
    //     let res = service_nok.validate(&());
    //     assert!(res.is_err());
    // }

    #[test]
    fn test_render_network() {
        for network in [
            BitcoinNetwork::Mainnet,
            BitcoinNetwork::Testnet,
            BitcoinNetwork::Regtest,
            BitcoinNetwork::Signet,
        ] {
            let nix = _get_network_test_string(&network);
            let service = BitcoinDaemonService {
                enabled: true,
                network,
                ..Default::default()
            };

            let (status, res) = service.render();

            assert!(!matches!(status, format::Status::Error(_)));
            assert_eq!(res.trim(), nix.trim());
        }
    }
}
