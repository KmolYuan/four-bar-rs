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
        let document = web_sys::window().unwrap().document().unwrap();

        use wasm_bindgen::JsCast as _;
        let canvas = document
            .get_element_by_id("app")
            .unwrap()
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .unwrap();

        let opt = eframe::WebOptions::default();
        eframe::WebRunner::new()
            .start(canvas, opt, app::App::create(Vec::new()))
            .await
            .expect("Startup failed");
    });
}
