use std::collections::HashMap;

use once_cell::sync::Lazy;
use strum::Display;

use crate::{
    app_option_data::option_data::{OptionId, ToOptionId},
    bitcoind::BitcoindConfigOption,
    cln::ClnConfigOption,
    nix_base_config::NixBaseConfigOption,
};

// default password: "nixblitz"
pub(crate) static INITIAL_PASSWORD: &str = "$6$rounds=10000$moY2rIPxoNODYRxz$1DESwWYweHNkoB6zBxI3DUJwUfvA6UkZYskLOHQ9ulxItgg/hP5CRn2Fr4iQGO7FE16YpJAPMulrAuYJnRC9B.";

pub static DECIMAL_SIGN: char = ',';

#[derive(Debug, Display, Hash, Eq, PartialEq)]
pub enum Strings {
    PasswordInputPlaceholderMain,
    PasswordInputPlaceholderConfirm,
}

pub static STRINGS: Lazy<HashMap<Strings, &str>> = Lazy::new(|| {
    let mut map = HashMap::new();
    map.insert(
        Strings::PasswordInputPlaceholderMain,
        "Please enter your password",
    );
    map.insert(
        Strings::PasswordInputPlaceholderConfirm,
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
    map.insert(NixBaseConfigOption::Username.to_option_id(), "Username");
    map.insert(
        NixBaseConfigOption::InitialPassword.to_option_id(),
        "Initial Password",
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
    map.insert(ClnConfigOption::User.to_option_id(), "Service User");
    map.insert(ClnConfigOption::Group.to_option_id(), "Service Group");
    map.insert(
        ClnConfigOption::GetPublicAddressCmd.to_option_id(),
        "Get Public Address Command",
    );
    map
});
