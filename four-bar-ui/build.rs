fn main() {
    let mut version = format!("v{}", env!("CARGO_PKG_VERSION"));
    if let Ok(hash) = std::process::Command::new("git")
        .args(["rev-parse", "HEAD"])
        .output()
    {
        if hash.status.success() {
            let hash = String::from_utf8_lossy(&hash.stdout[..7]);
            version.push_str(" (");
            version.push_str(&hash);
            version.push(')');
        }
    }
    println!("cargo:rustc-env=APP_VERSION={version}");
    #[cfg(windows)]
    {
        let profile = std::env::var("PROFILE").unwrap();
        let target_env = std::env::var("CARGO_CFG_TARGET_ENV").unwrap();
        if target_env == "gnu" || target_env == "msvc" && profile == "release" {
            let ico = image::open("assets/favicon.png").unwrap();
            let path =
                std::env::var("OUT_DIR").unwrap() + std::path::MAIN_SEPARATOR_STR + "icon.ico";
            ico.save(&path).unwrap();
            winres::WindowsResource::new()
                .set_icon(&path)
                .compile()
                .unwrap_or_default();
        }
    }
}
