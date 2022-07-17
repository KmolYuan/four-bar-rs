use clap::Parser;
use eframe::{IconData, NativeOptions};
use four_bar_ui::App;
use image::ImageFormat;

#[derive(Parser)]
#[clap(name = "four-bar", version, author, about)]
struct Entry {
    /// File path
    files: Vec<String>,
}

fn main() -> ! {
    const ICON: &[u8] = include_bytes!("../../assets/favicon.png");
    let icon = image::load_from_memory_with_format(ICON, ImageFormat::Png).unwrap();
    let opt = NativeOptions {
        icon_data: Some(IconData {
            width: icon.width(),
            height: icon.height(),
            rgba: icon.into_bytes(),
        }),
        ..Default::default()
    };
    #[cfg(windows)]
    let _ = unsafe { winapi::um::wincon::FreeConsole() };
    let files = Entry::parse().files;
    eframe::run_native("Four bar", opt, Box::new(|ctx| App::new(ctx, files)))
}
