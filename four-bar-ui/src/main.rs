mod app;
#[cfg(not(target_arch = "wasm32"))]
mod cli;
mod csv;

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    cli::Entry::parse()
}

#[cfg(target_arch = "wasm32")]
fn main() {
    console_error_panic_hook::set_once();
    tracing_wasm::set_as_global_default();
    let opt = eframe::WebOptions::default();
    eframe::start_web("app", opt, Box::new(|ctx| app::App::new(ctx, Vec::new())))
        .expect("failed to startup");
}
