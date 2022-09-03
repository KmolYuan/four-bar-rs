#[cfg(not(windows))]
fn main() {}

#[cfg(windows)]
fn main() {
    let profile = std::env::var("PROFILE").unwrap();
    let target_env = std::env::var("CARGO_CFG_TARGET_ENV").unwrap();
    if target_env == "gnu" || target_env == "msvc" && profile == "release" {
        println!("cargo:rerun-if-changed=assets/*");
        let out = std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap());
        let ico = image::open("assets/favicon.png").unwrap();
        let ico_path = out.join("icon.ico");
        ico.save(&ico_path).unwrap();
        winres::WindowsResource::new()
            .set_icon(ico_path.to_str().unwrap())
            .compile()
            .unwrap();
    }
}
