#![forbid(unsafe_code)]
#![cfg_attr(not(debug_assertions), deny(warnings))] // Forbid warnings in release builds
#![warn(clippy::all, rust_2018_idioms)]
use four_bar_ui::{
    icon::{HEIGHT, ICON, WIDTH},
    App,
};

/// Native entry point.
fn main() {
    let app = Box::new(App::default());
    eframe::run_native(
        app,
        eframe::NativeOptions {
            icon_data: Some(eframe::epi::IconData {
                rgba: Vec::from(ICON),
                width: WIDTH,
                height: HEIGHT,
            }),
            ..Default::default()
        },
    );
}
