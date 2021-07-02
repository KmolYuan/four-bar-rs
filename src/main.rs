#![forbid(unsafe_code)]
#![cfg_attr(not(debug_assertions), deny(warnings))] // Forbid warnings in release builds
#![warn(clippy::all, rust_2018_idioms)]
use four_bar::App;

/// Native entry point.
#[cfg(not(target_arch = "wasm32"))]
fn main() {
    let app = App::default();
    let opt = eframe::NativeOptions::default();
    eframe::run_native(Box::new(app), opt);
}
