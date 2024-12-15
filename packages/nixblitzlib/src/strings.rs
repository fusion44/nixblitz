use std::collections::HashMap;

use once_cell::sync::Lazy;
use strum::Display;

use crate::{
    app_option_data::option_data::{OptionId, ToOptionId},
    nix_base_config::NixBaseConfigOption,
};

// default password: "nixblitz"
pub(crate) static INITIAL_PASSWORD: &str = "$6$rounds=10000$moY2rIPxoNODYRxz$1DESwWYweHNkoB6zBxI3DUJwUfvA6UkZYskLOHQ9ulxItgg/hP5CRn2Fr4iQGO7FE16YpJAPMulrAuYJnRC9B.";

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

    map
});
