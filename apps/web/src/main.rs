//! The website: one more host of the core. The core decides, this draws and performs its effects.

mod host;
mod player;
mod state;
mod ui;

fn main() {
    console_error_panic_hook::set_once();
    leptos::mount::mount_to_body(ui::App);
}
