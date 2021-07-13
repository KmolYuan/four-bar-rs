#![forbid(unsafe_code)]
#![cfg_attr(not(debug_assertions), deny(warnings))] // Forbid warnings in release builds
#![warn(clippy::all, rust_2018_idioms)]

/// Native entry point.
#[cfg(not(target_arch = "wasm32"))]
fn main() {
    use four_bar_ui::App;
    use image::{load_from_memory_with_format, DynamicImage, ImageFormat};
    const ICON: &[u8] = include_bytes!("assets/icon.png");
    let icon = if let DynamicImage::ImageRgba8(buf) =
        load_from_memory_with_format(ICON, ImageFormat::Png).unwrap()
    {
        eframe::epi::IconData {
            rgba: buf.to_vec(),
            width: buf.width(),
            height: buf.height(),
        }
    } else {
        panic!("Never failed");
    };
    let app = App::default();
    let opt = eframe::NativeOptions {
        icon_data: Some(icon),
        ..Default::default()
    };
    eframe::run_native(Box::new(app), opt);
}
