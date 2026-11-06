//! The website: one more host of the core. The core decides, this draws and performs its effects.

mod adapters;
mod host;
mod ports;
mod state;
mod surfaces;

fn main() {
    console_error_panic_hook::set_once();
    leptos::mount::mount_to_body(surfaces::App);
}
