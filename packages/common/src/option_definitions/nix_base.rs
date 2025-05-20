use std::{fmt::Display, str::FromStr};

use serde::{Deserialize, Serialize};
use strum_macros::{EnumCount, VariantArray};

use crate::{
    app_option_data::option_data::{OptionId, ToOptionId},
    apps::SupportedApps,
};

#[derive(
    Debug, Clone, Copy, EnumCount, VariantArray, Hash, PartialEq, Eq, Serialize, Deserialize,
)]
pub enum NixBaseConfigOption {
    AllowUnfree,
    TimeZone,
    DefaultLocale,
    DiskoDevice,
    Username,
    InitialPassword,
    SystemPlatform,
}

impl ToOptionId for NixBaseConfigOption {
    fn to_option_id(&self) -> OptionId {
        OptionId::new(SupportedApps::NixOS, self.to_string())
    }
}

impl FromStr for NixBaseConfigOption {
    type Err = ();

    fn from_str(s: &str) -> std::result::Result<NixBaseConfigOption, ()> {
        match s {
            "allow_unfree" => Ok(NixBaseConfigOption::AllowUnfree),
            "time_zone" => Ok(NixBaseConfigOption::TimeZone),
            "default_locale" => Ok(NixBaseConfigOption::DefaultLocale),
            "disko_device" => Ok(NixBaseConfigOption::DiskoDevice),
            "username" => Ok(NixBaseConfigOption::Username),
            "initial_password" => Ok(NixBaseConfigOption::InitialPassword),
            "platform" => Ok(NixBaseConfigOption::SystemPlatform),
            _ => Err(()),
        }
    }
}

impl Display for NixBaseConfigOption {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            NixBaseConfigOption::AllowUnfree => "allow_unfree",
            NixBaseConfigOption::TimeZone => "time_zone",
            NixBaseConfigOption::DefaultLocale => "default_locale",
            NixBaseConfigOption::DiskoDevice => "disko_device",
            NixBaseConfigOption::Username => "username",
            NixBaseConfigOption::InitialPassword => "initial_password",
            NixBaseConfigOption::SystemPlatform => "platform",
        };
        write!(f, "{}", s)
    }
}
