fn main() {
    #[cfg(not(target_arch = "wasm32"))]
    make_favicon();
}

#[cfg(not(target_arch = "wasm32"))]
fn make_favicon() {
    use image::{load_from_memory_with_format, DynamicImage, ImageFormat};
    use std::fs::write;
    const ICON: &[u8] = include_bytes!("./src/assets/favicon.png");
    let buf = if let DynamicImage::ImageRgba8(buf) =
        load_from_memory_with_format(ICON, ImageFormat::Png).unwrap()
    {
        buf
    } else {
        unreachable!()
    };
    let doc = format!(
        "\
pub const WIDTH: u32 = {};
pub const HEIGHT: u32 = {};
pub const ICON: &[u8] = &{:?};
",
        buf.width(),
        buf.height(),
        buf.to_vec()
    );
    write("./src/icon.rs", doc).unwrap();
}
