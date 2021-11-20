use image::GenericImageView;

fn main() {
    #[cfg(not(target_arch = "wasm32"))]
    make_favicon();
}

#[cfg(not(target_arch = "wasm32"))]
fn make_favicon() {
    use image::io::Reader;
    use std::fs::write;
    let img = Reader::open("src/assets/favicon.png")
        .unwrap()
        .decode()
        .unwrap();
    let doc = format!(
        "\
pub const WIDTH: u32 = {};
pub const HEIGHT: u32 = {};
pub const ICON: &[u8] = &{:?};",
        img.width(),
        img.height(),
        img.as_bytes()
    );
    write("src/icon.rs", doc).unwrap();
}
