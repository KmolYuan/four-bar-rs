#![forbid(unsafe_code)]
pub use crate::{
    app::{sha512, App, LoginInfo},
    csv_io::{dump_csv, parse_csv},
};
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::{wasm_bindgen, JsValue};

mod app;
mod as_values;
mod csv_io;

/// WebAssembly entry point.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn start(id: &str) -> Result<(), JsValue> {
    let app = Box::new(App::open(None));
    eframe::start_web(id, app)
}
