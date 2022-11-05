#![warn(clippy::semicolon_if_nothing_returned)]

mod app;
#[cfg(not(target_arch = "wasm32"))]
mod cli;
mod csv;
mod syn_method;

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    self::cli::Entry::parse();
}

#[cfg(target_arch = "wasm32")]
fn main() {
    use self::app::App;
    console_error_panic_hook::set_once();
    tracing_wasm::set_as_global_default();
    let opt = eframe::WebOptions::default();
    eframe::start_web("app", opt, Box::new(|ctx| App::new(ctx, Vec::new())))
        .expect("failed to startup");
}
