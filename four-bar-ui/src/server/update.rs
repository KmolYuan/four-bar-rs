use reqwest::get;
use std::{
    env::current_exe,
    fs::{write, File},
    io::Result,
    path::Path,
};
use zip::ZipArchive;

macro_rules! archive {
    () => {
        "four-bar-wasm-unknown"
    };
}

macro_rules! wasm_url {
    () => {
        "https://github.com/KmolYuan/four-bar-rs/releases/latest/download/four-bar-wasm-unknown.zip"
    };
}

pub async fn update() -> Result<()> {
    println!(concat!("Downloading archive from ", wasm_url!()));
    let b = get(wasm_url!()).await.unwrap().bytes().await.unwrap();
    let archive = current_exe()?.with_file_name(concat!(archive!(), ".zip"));
    write(archive, b)?;
    println!("Done");
    Ok(())
}

pub(crate) async fn extract<D>(d: D) -> Result<()>
where
    D: AsRef<Path>,
{
    let path = current_exe()?.with_file_name(concat!(archive!(), ".zip"));
    if !path.exists() {
        update().await?;
    }
    ZipArchive::new(File::open(path)?)
        .unwrap()
        .extract(d.as_ref())
        .unwrap();
    Ok(())
}
