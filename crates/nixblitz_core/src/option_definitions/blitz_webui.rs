use core::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::{
    app_option_data::option_data::{OptionId, ToOptionId},
    apps::SupportedApps,
};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum BlitzWebUiConfigOption {
    Enable,
    NginxEnable,
}

impl ToOptionId for BlitzWebUiConfigOption {
    fn to_option_id(&self) -> OptionId {
        OptionId::new(SupportedApps::WebUI, self.to_string())
    }
}
impl FromStr for BlitzWebUiConfigOption {
    type Err = ();

    fn from_str(s: &str) -> std::result::Result<BlitzWebUiConfigOption, ()> {
        match s {
            "enable" => Ok(BlitzWebUiConfigOption::Enable),
            "nginx_enable" => Ok(BlitzWebUiConfigOption::NginxEnable),
            _ => Err(()),
        }
    }
}

impl fmt::Display for BlitzWebUiConfigOption {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let option_str = match self {
            BlitzWebUiConfigOption::Enable => "enable",
            BlitzWebUiConfigOption::NginxEnable => "nginx_enable",
        };
        write!(f, "{}", option_str)
    }
}
