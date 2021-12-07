use ehttp::{fetch_blocking, Request};
use std::{
    env::current_exe,
    fs::{write, File},
    io::{Error, ErrorKind, Result},
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
    let archive = current_exe()?.with_file_name(concat!(archive!(), ".zip"));
    match fetch_blocking(&Request::get(wasm_url!())) {
        Ok(r) if r.ok => write(archive, r.bytes),
        _ => Err(Error::new(ErrorKind::NotFound, "Fetch failed")),
    }?;
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
