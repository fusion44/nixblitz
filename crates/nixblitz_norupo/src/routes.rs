use crate::pages::{Config, Install};
use dioxus::prelude::*;

#[derive(Debug, Clone, Routable, PartialEq)]
#[rustfmt::skip]
pub enum Route {
    #[route("/")]
    Config {},

    #[route("/install")]
    Install {},
}
