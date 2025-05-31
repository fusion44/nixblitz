use components::app::App;
use dioxus::prelude::launch;

mod backend;
mod components;
mod constants;
mod pages;
mod routes;

fn main() {
    dioxus::launch(App);
}
