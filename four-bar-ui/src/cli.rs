use clap::Parser;
use std::path::PathBuf;

mod syn;

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
    /// Synthesis function without GUI
    Syn {
        /// Target file paths in "[path]/[name].[mode].[ron|csv|txt]" pattern
        files: Vec<PathBuf>,
        /// Disable parallel for all task
        #[clap(long)]
        no_parallel: bool,
        #[clap(flatten)]
        syn: Syn,
    },
}

#[derive(clap::Args, Clone)]
struct Syn {
    /// Number of the points (resolution) in curve production
    #[clap(short, long = "res", default_value_t = 90)]
    n: usize,
    /// Number of generation
    #[clap(short, long, default_value_t = 50)]
    gen: u64,
    /// Number of population
    #[clap(short, long, default_value_t = 400)]
    pop: usize,
}

impl Entry {
    pub fn parse() {
        let entry = <Self as Parser>::parse_from(wild::args());
        match entry.cmd {
            None => native(entry.files),
            Some(Cmd::Ui { files }) => native(files),
            Some(Cmd::Syn { files, no_parallel, syn }) => syn::syn(files, no_parallel, syn),
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
    #[cfg(windows)]
    let _ = unsafe { winapi::um::wincon::FreeConsole() };
    eframe::run_native(
        "Four-bar",
        opt,
        Box::new(|ctx| crate::app::App::new(ctx, files)),
    )
}