use core::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::{
    app_option_data::option_data::{OptionId, ToOptionId},
    apps::SupportedApps,
};

/// To which node to connect to
pub enum ConnectionType {
    /// LND via GRPC
    LndGrpc,
    /// CLN via Json-RPC
    ClnJrpc,
    /// CLN via GRPC
    ClnGrpc,
    /// Bitcoin only mode
    None,
}

impl ConnectionType {
    pub fn to_string_array() -> [&'static str; 4] {
        ["lnd_grpc", "cln_jrpc", "cln_grpc", "none"]
    }
}

impl fmt::Display for ConnectionType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let conn_type_str = match self {
            ConnectionType::LndGrpc => "lnd_grpc",
            ConnectionType::ClnJrpc => "cln_jrpc",
            ConnectionType::ClnGrpc => "cln_grpc",
            ConnectionType::None => "none",
        };
        write!(f, "{}", conn_type_str)
    }
}

impl FromStr for ConnectionType {
    type Err = ();

    fn from_str(s: &str) -> std::result::Result<ConnectionType, ()> {
        match s {
            "lnd_grpc" => Ok(ConnectionType::LndGrpc),
            "cln_jrpc" => Ok(ConnectionType::ClnJrpc),
            "cln_grpc" => Ok(ConnectionType::ClnGrpc),
            "none" => Ok(ConnectionType::None),
            _ => Err(()),
        }
    }
}

/// Log levels according to the loguru library
/// https://loguru.readthedocs.io/en/stable/api/logger.html
pub enum BlitzApiLogLevel {
    Trace,
    Debug,
    Info,
    Success,
    Warning,
    Error,
    Critical,
}

impl BlitzApiLogLevel {
    pub fn to_string_array() -> [&'static str; 7] {
        [
            "TRACE", "DEBUG", "INFO", "SUCCESS", "WARNING", "ERROR", "CRITICAL",
        ]
    }
}

impl fmt::Display for BlitzApiLogLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let level_str = match self {
            BlitzApiLogLevel::Trace => "TRACE",
            BlitzApiLogLevel::Debug => "DEBUG",
            BlitzApiLogLevel::Info => "INFO",
            BlitzApiLogLevel::Success => "SUCCESS",
            BlitzApiLogLevel::Warning => "WARNING",
            BlitzApiLogLevel::Error => "ERROR",
            BlitzApiLogLevel::Critical => "CRITICAL",
        };
        write!(f, "{}", level_str)
    }
}

impl FromStr for BlitzApiLogLevel {
    type Err = ();

    fn from_str(s: &str) -> std::result::Result<BlitzApiLogLevel, ()> {
        match s {
            "TRACE" => Ok(BlitzApiLogLevel::Trace),
            "DEBUG" => Ok(BlitzApiLogLevel::Debug),
            "INFO" => Ok(BlitzApiLogLevel::Info),
            "SUCCESS" => Ok(BlitzApiLogLevel::Success),
            "WARNING" => Ok(BlitzApiLogLevel::Warning),
            "ERROR" => Ok(BlitzApiLogLevel::Error),
            "CRITICAL" => Ok(BlitzApiLogLevel::Critical),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum BlitzApiConfigOption {
    Enable,
    ConnectionType,
    LogLevel,
    GenerateEnvFile,
    EnvFilePath,
    PasswordFile,
    RootPath,
    NginxEnable,
    NginxOpenFirewall,
    NginxLocation,
}

impl ToOptionId for BlitzApiConfigOption {
    fn to_option_id(&self) -> OptionId {
        OptionId::new(SupportedApps::BlitzAPI, self.to_string())
    }
}
impl FromStr for BlitzApiConfigOption {
    type Err = ();

    fn from_str(s: &str) -> std::result::Result<BlitzApiConfigOption, ()> {
        match s {
            "enable" => Ok(BlitzApiConfigOption::Enable),
            "connection_type" => Ok(BlitzApiConfigOption::ConnectionType),
            "log_level" => Ok(BlitzApiConfigOption::LogLevel),
            "env_file" => Ok(BlitzApiConfigOption::EnvFilePath),
            "password_file" => Ok(BlitzApiConfigOption::PasswordFile),
            "root_path" => Ok(BlitzApiConfigOption::RootPath),
            "nginx_enable" => Ok(BlitzApiConfigOption::NginxEnable),
            "nginx_open_firewall" => Ok(BlitzApiConfigOption::NginxOpenFirewall),
            "nginx_location" => Ok(BlitzApiConfigOption::NginxLocation),
            _ => Err(()),
        }
    }
}

impl fmt::Display for BlitzApiConfigOption {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let option_str = match self {
            BlitzApiConfigOption::Enable => "enable",
            BlitzApiConfigOption::ConnectionType => "connection_type",
            BlitzApiConfigOption::LogLevel => "log_level",
            BlitzApiConfigOption::GenerateEnvFile => "generate_env_file",
            BlitzApiConfigOption::EnvFilePath => "env_file",
            BlitzApiConfigOption::PasswordFile => "password_file",
            BlitzApiConfigOption::RootPath => "root_path",
            BlitzApiConfigOption::NginxEnable => "nginx_enable",
            BlitzApiConfigOption::NginxOpenFirewall => "nginx_open_firewall",
            BlitzApiConfigOption::NginxLocation => "nginx_location",
        };
        write!(f, "{}", option_str)
    }
}
