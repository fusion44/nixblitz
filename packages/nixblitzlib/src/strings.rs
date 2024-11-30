use std::collections::HashMap;

use once_cell::sync::Lazy;

use crate::{
    app_option_data::option_data::OptionId, apps::SupportedApps,
    nix_base_config::NixBaseConfigOption,
};

pub static OPTION_TITLES: Lazy<HashMap<OptionId, &str>> = Lazy::new(|| {
    let mut map = HashMap::new();
    map.insert(
        OptionId::new(
            SupportedApps::NixOS,
            NixBaseConfigOption::AllowUnfree.to_string(),
        ),
        "Allow Unfree Packages",
    );
    map.insert(
        OptionId::new(
            SupportedApps::NixOS,
            NixBaseConfigOption::TimeZone.to_string(),
        ),
        "Time Zone",
    );
    map.insert(
        OptionId::new(
            SupportedApps::NixOS,
            NixBaseConfigOption::DefaultLocale.to_string(),
        ),
        "Default Locale",
    );
    map.insert(
        OptionId::new(
            SupportedApps::NixOS,
            NixBaseConfigOption::Username.to_string(),
        ),
        "Username",
    );
    map.insert(
        OptionId::new(
            SupportedApps::NixOS,
            NixBaseConfigOption::InitialPassword.to_string(),
        ),
        "Initial Password",
    );

    map
});
