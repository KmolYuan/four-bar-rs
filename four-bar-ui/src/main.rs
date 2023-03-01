#![warn(clippy::semicolon_if_nothing_returned)]

mod app;
#[cfg(not(target_arch = "wasm32"))]
mod cli;
mod syn_cmd;

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    self::cli::Entry::parse();
}

#[cfg(target_arch = "wasm32")]
fn main() {
    use self::app::App;
    console_error_panic_hook::set_once();
    tracing_wasm::set_as_global_default();
    let fut = async {
        let opt = eframe::WebOptions::default();
        eframe::start_web("app", opt, Box::new(|ctx| App::new(ctx, Vec::new())))
            .await
            .expect("failed to startup");
    };
    wasm_bindgen_futures::spawn_local(fut);
}
