use components::app::App;
use dioxus::prelude::launch;

mod backend;
mod classes;
mod components;
mod constants;
mod installer_engine_connection;
mod pages;
mod routes;

fn main() {
    dioxus::launch(App);
}
