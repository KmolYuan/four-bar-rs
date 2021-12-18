#[cfg(target_arch = "wasm32")]
fn main() {}

#[cfg(not(target_arch = "wasm32"))]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    use image::{io::Reader, GenericImageView};
    use std::{fs::write, path::PathBuf};

    println!("cargo:rerun-if-changed=src/assets/favicon.png");
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
    #[cfg(windows)]
    {
        let ico_path = out_dir.join("icon.ico");
        img.save(&ico_path)?;
        winres::WindowsResource::new()
            .set_icon(ico_path.to_str().unwrap())
            .compile()
            .unwrap();
    }
    Ok(())
}
