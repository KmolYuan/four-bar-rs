use crate::app::App;
use std::path::PathBuf;

mod atlas;
mod syn;

const APP_NAME: &str = env!("CARGO_PKG_NAME");

#[derive(clap::Parser)]
#[clap(name = APP_NAME, version, author, about)]
pub(crate) struct Entry {
    /// Default to startup GUI then open file paths
    files: Vec<PathBuf>,
    #[clap(subcommand)]
    cmd: Option<Cmd>,
}

#[derive(clap::Subcommand)]
enum Cmd {
    /// Startup GUI
    Ui {
        /// Open file path
        files: Vec<PathBuf>,
    },
    /// Synthesis function without GUI
    Syn(syn::Syn),
    /// Generate atlas database without GUI
    Atlas(atlas::AtlasCfg),
}

impl Entry {
    pub(super) fn main() {
        let entry = <Self as clap::Parser>::parse_from(wild::args());
        match entry.cmd {
            None => native(entry.files),
            Some(Cmd::Ui { files }) => native(files),
            Some(Cmd::Syn(syn)) => syn::syn(syn),
            Some(Cmd::Atlas(atlas)) => atlas::atlas(atlas),
        }
    }
}

fn native(files: Vec<PathBuf>) {
    let opt = {
        use image::ImageFormat::Png;
        const ICON: &[u8] = include_bytes!("../assets/favicon.png");
        let icon = image::load_from_memory_with_format(ICON, Png).unwrap();
        eframe::NativeOptions {
            icon_data: Some(eframe::IconData {
                width: icon.width(),
                height: icon.height(),
                rgba: icon.into_bytes(),
            }),
            ..Default::default()
        }
    };
    #[cfg(all(windows, feature = "native-win-release"))]
    unsafe {
        winapi::um::wincon::FreeConsole();
    }
    eframe::run_native(APP_NAME, opt, App::create(files)).unwrap();
}
