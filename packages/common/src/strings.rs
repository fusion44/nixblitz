use std::{collections::HashMap, fmt::Display};

use once_cell::sync::Lazy;

use crate::{
    app_option_data::option_data::{OptionId, ToOptionId},
    errors::StringErrors,
    option_definitions::{
        bitcoind::BitcoindConfigOption, blitz_api::BlitzApiConfigOption,
        blitz_webui::BlitzWebUiConfigOption, cln::ClnConfigOption, lnd::LndConfigOption,
        nix_base::NixBaseConfigOption,
    },
    utils::GetStringOrCliError,
};

pub static DECIMAL_SIGN: char = ',';

#[derive(Debug, Hash, Eq, PartialEq)]
pub enum CommonStrings {
    PasswordInputPlaceholderMain,
    PasswordInputPlaceholderConfirm,
}

impl Display for CommonStrings {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CommonStrings::PasswordInputPlaceholderMain => {
                f.write_str("PasswordInputPlaceholderMain")
            }
            CommonStrings::PasswordInputPlaceholderConfirm => {
                f.write_str("PasswordInputPlaceholderConfirm")
            }
        }
    }
}

impl GetStringOrCliError for CommonStrings {
    fn get_or_err(&self) -> Result<&str, StringErrors> {
        match self {
            CommonStrings::PasswordInputPlaceholderMain => Ok(STRINGS
                .get(self)
                .ok_or(StringErrors::StringRetrievalError(self.to_string()))?),
            CommonStrings::PasswordInputPlaceholderConfirm => Ok(STRINGS
                .get(self)
                .ok_or(StringErrors::StringRetrievalError(self.to_string()))?),
        }
    }
}

pub static STRINGS: Lazy<HashMap<CommonStrings, &str>> = Lazy::new(|| {
    let mut map = HashMap::new();
    map.insert(
        CommonStrings::PasswordInputPlaceholderMain,
        "Please enter your password",
    );
    map.insert(
        CommonStrings::PasswordInputPlaceholderConfirm,
        "Please confirm your password",
    );
    map
});

pub static OPTION_TITLES: Lazy<HashMap<OptionId, &str>> = Lazy::new(|| {
    let mut map = HashMap::new();
    // NIX BASE CONFIG
    map.insert(
        NixBaseConfigOption::AllowUnfree.to_option_id(),
        "Allow Unfree Packages",
    );
    map.insert(NixBaseConfigOption::TimeZone.to_option_id(), "Time Zone");
    map.insert(
        NixBaseConfigOption::DefaultLocale.to_option_id(),
        "Default Locale",
    );
    map.insert(
        NixBaseConfigOption::DiskoDevice.to_option_id(),
        "Disko Device",
    );
    map.insert(NixBaseConfigOption::Username.to_option_id(), "Username");
    map.insert(
        NixBaseConfigOption::InitialPassword.to_option_id(),
        "Initial Password",
    );
    map.insert(
        NixBaseConfigOption::SystemPlatform.to_option_id(),
        "System Platform",
    );
    map.insert(
        NixBaseConfigOption::SshAuthKeys.to_option_id(),
        "SSH Auth Keys",
    );

    // BITCOIN CORE
    map.insert(BitcoindConfigOption::Enable.to_option_id(), "Enable");
    map.insert(
        BitcoindConfigOption::Address.to_option_id(),
        "Network Address",
    );
    map.insert(BitcoindConfigOption::Port.to_option_id(), "listen port");
    map.insert(
        BitcoindConfigOption::OnionPort.to_option_id(),
        "Tor peer connections port",
    );
    map.insert(
        BitcoindConfigOption::Listen.to_option_id(),
        "Listen for peer connections",
    );
    map.insert(
        BitcoindConfigOption::ExtraConfig.to_option_id(),
        "Extra config",
    );
    map.insert(BitcoindConfigOption::User.to_option_id(), "Service user");
    map.insert(
        BitcoindConfigOption::Network.to_option_id(),
        "bitcoin network",
    );
    map.insert(BitcoindConfigOption::RpcUsers.to_option_id(), "RPC users");
    map.insert(
        BitcoindConfigOption::RpcAddress.to_option_id(),
        "RPC address",
    );
    map.insert(BitcoindConfigOption::RpcPort.to_option_id(), "RPC port");
    map.insert(
        BitcoindConfigOption::RpcAllowIp.to_option_id(),
        "Ips allowed to access RPC",
    );
    map.insert(
        BitcoindConfigOption::Prune.to_option_id(),
        "Whether to prune",
    );
    map.insert(
        BitcoindConfigOption::PruneSize.to_option_id(),
        "Size at which to prune",
    );
    map.insert(
        BitcoindConfigOption::ExtraCmdLineOptions.to_option_id(),
        "Extra command line options",
    );
    map.insert(
        BitcoindConfigOption::DbCache.to_option_id(),
        "Database cache size",
    );
    map.insert(
        BitcoindConfigOption::DataDir.to_option_id(),
        "The data directory",
    );
    map.insert(
        BitcoindConfigOption::TxIndex.to_option_id(),
        "Enable txindex",
    );
    map.insert(
        BitcoindConfigOption::DisableWallet.to_option_id(),
        "disable the wallet",
    );
    map.insert(
        BitcoindConfigOption::ZmqPubRawTx.to_option_id(),
        "ZMQ address for zmqpubrawtx",
    );
    map.insert(
        BitcoindConfigOption::ZmqPubRawBlock.to_option_id(),
        "ZMQ address for zmqpubrawblock",
    );

    // CORE LIGHTNING
    map.insert(
        ClnConfigOption::Enable.to_option_id(),
        "Whether to enable the service",
    );
    map.insert(ClnConfigOption::Address.to_option_id(), "Network Address");
    map.insert(ClnConfigOption::Port.to_option_id(), "Listen Port");
    map.insert(ClnConfigOption::Proxy.to_option_id(), "Proxy Server");
    map.insert(
        ClnConfigOption::AlwaysUseProxy.to_option_id(),
        "Always Use Proxy",
    );
    map.insert(ClnConfigOption::DataDir.to_option_id(), "Data Directory");
    map.insert(
        ClnConfigOption::Wallet.to_option_id(),
        "Wallet Configuration",
    );
    map.insert(
        ClnConfigOption::ExtraConfig.to_option_id(),
        "Extra Configuration",
    );
    map.insert(
        ClnConfigOption::GetPublicAddressCmd.to_option_id(),
        "Get Public Address Command",
    );

    // LIGHTNING NETWORK DAEMON
    map.insert(
        LndConfigOption::Enable.to_option_id(),
        "Whether to enable the service",
    );
    map.insert(LndConfigOption::Address.to_option_id(), "Network Address");
    map.insert(LndConfigOption::Port.to_option_id(), "Listen Port");
    map.insert(LndConfigOption::RpcAddress.to_option_id(), "RPC Address");
    map.insert(LndConfigOption::RpcPort.to_option_id(), "RPC Port");
    map.insert(LndConfigOption::RestAddress.to_option_id(), "REST Address");
    map.insert(LndConfigOption::RestPort.to_option_id(), "REST Port");
    map.insert(LndConfigOption::DataDir.to_option_id(), "Data Directory");
    map.insert(
        LndConfigOption::CertExtraIps.to_option_id(),
        "Certificate Extra IPs",
    );
    map.insert(
        LndConfigOption::CertExtraDomains.to_option_id(),
        "Certificate Extra Domains",
    );
    map.insert(
        LndConfigOption::ExtraConfig.to_option_id(),
        "Extra Configuration",
    );

    // BLITZ API
    map.insert(
        BlitzApiConfigOption::Enable.to_option_id(),
        "Enable Blitz API",
    );
    map.insert(
        BlitzApiConfigOption::ConnectionType.to_option_id(),
        "The node to connect to",
    );
    map.insert(
        BlitzApiConfigOption::GenerateEnvFile.to_option_id(),
        "Whether to generate a .env file",
    );
    map.insert(
        BlitzApiConfigOption::LogLevel.to_option_id(),
        "The log level",
    );
    map.insert(
        BlitzApiConfigOption::EnvFilePath.to_option_id(),
        "Environment file path",
    );
    map.insert(
        BlitzApiConfigOption::PasswordFile.to_option_id(),
        "Password file path",
    );
    map.insert(
        BlitzApiConfigOption::RootPath.to_option_id(),
        "The root path",
    );
    map.insert(
        BlitzApiConfigOption::NginxEnable.to_option_id(),
        "Expose the API via nginx",
    );
    map.insert(
        BlitzApiConfigOption::NginxOpenFirewall.to_option_id(),
        "Open the nginx port",
    );
    map.insert(
        BlitzApiConfigOption::NginxLocation.to_option_id(),
        "The nginx path",
    );

    // BLITZ WEB UI
    map.insert(
        BlitzWebUiConfigOption::Enable.to_option_id(),
        "Enable Blitz WEB UI",
    );
    map.insert(
        BlitzWebUiConfigOption::NginxEnable.to_option_id(),
        "Expose via nginx",
    );

    map
});
