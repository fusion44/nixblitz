use components::app::App;
use dioxus::prelude::launch;

mod backend;
mod classes;
mod components;
mod constants;
mod installer_engine_connection;
mod pages;
mod routes;
mod string_formatters;
mod system_engine_connection;

fn main() {
    dioxus::launch(App);
}
