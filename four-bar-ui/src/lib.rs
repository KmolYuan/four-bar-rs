#![forbid(unsafe_code)]
#![warn(clippy::all, rust_2018_idioms)]
pub use crate::app::App;

mod app;
#[cfg(not(target_arch = "wasm32"))]
pub mod icon;
mod linkage;
#[cfg(not(target_arch = "wasm32"))]
mod synthesis;

#[cfg(target_arch = "wasm32")]
use eframe::wasm_bindgen::{self, prelude::*};

/// This is the entry-point for all the web-assembly.
/// This is called once from the HTML.
/// It loads the app, installs some callbacks, then returns.
/// You can add more callbacks like this if you want to call in to your code.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn start(id: &str) -> Result<(), eframe::wasm_bindgen::JsValue> {
    eframe::start_web(id, Box::new(App::default()))
}
