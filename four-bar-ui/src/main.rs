use crate::app::App;

mod app;

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    use clap::Parser;
    use image::ImageFormat;
    #[derive(Parser)]
    #[clap(name = "four-bar", version, author, about)]
    struct Entry {
        /// File path
        files: Vec<String>,
    }
    const ICON: &[u8] = include_bytes!("../assets/favicon.png");
    let icon = image::load_from_memory_with_format(ICON, ImageFormat::Png).unwrap();
    let opt = eframe::NativeOptions {
        icon_data: Some(eframe::IconData {
            width: icon.width(),
            height: icon.height(),
            rgba: icon.into_bytes(),
        }),
        ..Default::default()
    };
    #[cfg(windows)]
    let _ = unsafe { winapi::um::wincon::FreeConsole() };
    let files = Entry::parse().files;
    eframe::run_native("Four-bar", opt, Box::new(|ctx| App::new(ctx, files)))
}

#[cfg(target_arch = "wasm32")]
fn main() {
    console_error_panic_hook::set_once();
    tracing_wasm::set_as_global_default();
    let opt = eframe::WebOptions::default();
    eframe::start_web("app", opt, Box::new(|ctx| App::new(ctx, Vec::new())))
        .expect("failed to startup");
}
