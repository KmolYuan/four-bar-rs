mod app;
#[cfg(not(target_arch = "wasm32"))]
mod cli;
mod io;
mod syn_cmd;

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
            .start("app", opt, app::App::create(Vec::new()))
            .await
            .expect("Startup failed");
    });
}
