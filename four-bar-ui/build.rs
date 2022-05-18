#[cfg(target_arch = "wasm32")]
fn main() {}

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    println!("cargo:rerun-if-changed=src/assets/*");
    let out = std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap());
    let ico = image::open("src/assets/favicon.png").unwrap();
    let doc = format!(
        "\
pub const WIDTH: u32 = {};
pub const HEIGHT: u32 = {};
pub const ICON: &[u8] = &{:?};",
        ico.width(),
        ico.height(),
        ico.as_bytes()
    );
    std::fs::write(out.join("icon.rs"), doc).unwrap();
    #[cfg(windows)]
    {
        let ico_path = out.join("icon.ico");
        ico.save(&ico_path).unwrap();
        winres::WindowsResource::new()
            .set_icon(ico_path.to_str().unwrap())
            .compile()
            .unwrap();
    }
}
