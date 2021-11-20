use std::error::Error;
#[cfg(not(target_arch = "wasm32"))]
use {
    image::{io::Reader, GenericImageView},
    std::{fs::write, path::PathBuf},
};

fn main() -> Result<(), Box<dyn Error>> {
    #[cfg(not(target_arch = "wasm32"))]
    make_favicon()?;
    Ok(())
}

#[cfg(not(target_arch = "wasm32"))]
fn make_favicon() -> Result<(), Box<dyn Error>> {
    let out_dir = PathBuf::from(std::env::var("OUT_DIR")?);
    let img = Reader::open("src/assets/favicon.png")?.decode()?;
    let doc = format!(
        "\
pub const WIDTH: u32 = {};
pub const HEIGHT: u32 = {};
pub const ICON: &[u8] = &{:?};",
        img.width(),
        img.height(),
        img.as_bytes()
    );
    write(out_dir.join("icon.rs"), doc)?;
    Ok(())
}
