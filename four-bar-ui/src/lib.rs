#![forbid(unsafe_code)]
pub use crate::app::App;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::{wasm_bindgen, JsValue};

mod app;

/// WebAssembly entry point.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn start(id: &str) -> Result<(), JsValue> {
    eframe::start_web(id, Box::new(|ctx| App::new(ctx, Vec::new())))
}
