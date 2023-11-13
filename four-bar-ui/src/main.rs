#![warn(clippy::semicolon_if_nothing_returned)]

mod app;
#[cfg(not(target_arch = "wasm32"))]
mod cli;
mod io;
mod syn_cmd;

const APP_NAME: &str = env!("CARGO_PKG_NAME");

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    cli::Entry::main();
}

#[cfg(target_arch = "wasm32")]
fn main() {
    console_error_panic_hook::set_once();
    tracing_wasm::set_as_global_default();
    wasm_bindgen_futures::spawn_local(async {
        let opt = eframe::WebOptions::default();
        eframe::WebRunner::new()
            .start(APP_NAME, opt, app::App::create(Vec::new()))
            .await
            .expect("startup failed");
    });
}
