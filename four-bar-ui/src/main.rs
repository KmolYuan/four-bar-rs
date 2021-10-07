#![forbid(unsafe_code)]
#![cfg_attr(not(debug_assertions), deny(warnings))] // Forbid warnings in release builds
#![warn(clippy::all, rust_2018_idioms)]

/// Native entry point.
#[cfg(not(target_arch = "wasm32"))]
fn main() {
    use four_bar_ui::{icon::ICON, App};
    let app = Box::new(App::default());
    let icon_data = Some(eframe::epi::IconData {
        rgba: Vec::from(ICON),
        width: 70,
        height: 76,
    });
    eframe::run_native(
        app,
        eframe::NativeOptions {
            icon_data,
            ..Default::default()
        },
    );
}
