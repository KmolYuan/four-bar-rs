#![forbid(unsafe_code)]
pub use crate::{
    app::{sha512, App, LoginInfo},
    csv_io::{dump_csv, parse_csv},
};
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::{wasm_bindgen, JsValue};

mod app;
mod as_values;
mod atomic;
mod csv_io;

/// WebAssembly entry point.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn start(id: &str) -> Result<(), JsValue> {
    eframe::start_web(id, Box::new(|ctx| Box::new(App::new(ctx, Vec::new()))))
}
