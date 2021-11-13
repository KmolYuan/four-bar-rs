#![forbid(unsafe_code)]
pub use crate::app::App;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::{wasm_bindgen, JsValue};

mod app;
mod as_values;
#[cfg(not(target_arch = "wasm32"))]
pub mod icon;
mod linkage;
#[cfg(not(target_arch = "wasm32"))]
mod synthesis;

/// This is the entry-point for all the web-assembly.
///
/// This is called once from the HTML.
/// It loads the app, installs some callbacks, then returns.
/// You can add more callbacks like this if you want to call in to your code.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn start(id: &str, save_fn: &js_sys::Function) -> Result<(), JsValue> {
    eframe::start_web(id, Box::new(App::with_hook(save_fn.clone())))
}
