use core::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::{
    app_option_data::option_data::{OptionId, ToOptionId},
    apps::SupportedApps,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LndConfigOption {
    Enable,
    Address,
    Port,
    RpcAddress,
    RpcPort,
    RestAddress,
    RestPort,
    DataDir,
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
            "rpc_address" => Ok(LndConfigOption::RpcAddress),
            "rpc_port" => Ok(LndConfigOption::RpcPort),
            "rest_address" => Ok(LndConfigOption::RestAddress),
            "rest_port" => Ok(LndConfigOption::RestPort),
            "data_dir" => Ok(LndConfigOption::DataDir),
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
            LndConfigOption::RpcAddress => "rpc_address",
            LndConfigOption::RpcPort => "rpc_port",
            LndConfigOption::RestAddress => "rest_address",
            LndConfigOption::RestPort => "rest_port",
            LndConfigOption::DataDir => "data_dir",
            LndConfigOption::CertExtraIps => "cert_extra_ips",
            LndConfigOption::CertExtraDomains => "cert_extra_domains",
            LndConfigOption::ExtraConfig => "extra_config",
        };
        write!(f, "{}", option_str)
    }
}
