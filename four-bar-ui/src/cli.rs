use clap::Parser;
use four_bar::FourBar;
use std::path::PathBuf;

#[derive(Parser)]
#[clap(name = "four-bar", version, author, about)]
pub struct Entry {
    /// Open file path
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
    /// Synthesis without GUI
    Syn {
        /// Target file paths
        files: Vec<PathBuf>,
    },
}

impl Entry {
    pub fn parse() {
        let entry = <Self as Parser>::parse();
        match entry.cmd {
            None => native(entry.files),
            Some(Cmd::Ui { files }) => native(files),
            Some(Cmd::Syn { files }) => syn(files),
        }
    }
}

fn native(files: Vec<PathBuf>) {
    use image::ImageFormat;
    let opt = {
        const ICON: &[u8] = include_bytes!("../assets/favicon.png");
        let icon = image::load_from_memory_with_format(ICON, ImageFormat::Png).unwrap();
        eframe::NativeOptions {
            icon_data: Some(eframe::IconData {
                width: icon.width(),
                height: icon.height(),
                rgba: icon.into_bytes(),
            }),
            ..Default::default()
        }
    };
    #[cfg(windows)]
    let _ = unsafe { winapi::um::wincon::FreeConsole() };
    eframe::run_native(
        "Four-bar",
        opt,
        Box::new(|ctx| crate::app::App::new(ctx, files)),
    )
}

fn syn(files: Vec<PathBuf>) {
    for file in files {
        println!("============");
        println!("File: \"{}\"", file.display());
        if let Err(e) = syn_inner(file) {
            println!("Error: \"{e}\"");
        }
    }
}

fn syn_inner(file: PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    if file.ends_with(".ron") {
        let _fb = ron::from_str::<FourBar>(&std::fs::read_to_string(file)?)?;
    } else if file.ends_with(".csv") {
        ron::from_str::<Vec<[f64; 2]>>(&std::fs::read_to_string(file)?)?;
    } else {
        return Err("unsupported format".into());
    }
    todo!()
}
