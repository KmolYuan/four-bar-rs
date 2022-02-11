use std::{
    env::current_exe,
    fs::File,
    io::{copy, Error, ErrorKind, Result},
    path::Path,
};
use ureq::agent;
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

pub fn update() -> Result<()> {
    println!(concat!["Downloading archive from ", wasm_url!()]);
    let archive = current_exe()?.with_file_name(concat![archive!(), ".zip"]);
    match agent().get(wasm_url!()).call() {
        Ok(r) => copy(&mut r.into_reader(), &mut File::create(archive)?),
        _ => Err(Error::new(ErrorKind::NotFound, "Fetch failed")),
    }?;
    println!("Done");
    Ok(())
}

pub fn extract<D>(d: D) -> Result<()>
where
    D: AsRef<Path>,
{
    let path = current_exe()?.with_file_name(concat![archive!(), ".zip"]);
    if !path.exists() {
        update()?;
    }
    ZipArchive::new(File::open(path)?)
        .unwrap()
        .extract(d.as_ref())
        .unwrap();
    Ok(())
}
