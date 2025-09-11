use components::app::App;

mod backend;
mod classes;
mod components;
mod constants;
mod pages;
mod routes;
mod string_formatters;

fn main() {
    dioxus::launch(App);
}
