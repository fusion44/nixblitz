use core::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::{
    app_option_data::option_data::{OptionId, ToOptionId},
    apps::SupportedApps,
};

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
            ClnConfigOption::GetPublicAddressCmd => "get_public_address_cmd",
        };
        write!(f, "{}", option_str)
    }
}
