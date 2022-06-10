use clap::Parser;
use eframe::{IconData, NativeOptions};
use four_bar_ui::App;
use image::ImageFormat;
use std::io::Result;

mod serve;
mod update;

const ICON: &[u8] = include_bytes!("../../../assets/favicon.png");

#[derive(Parser)]
#[clap(
    name = "four-bar",
    version = env!("CARGO_PKG_VERSION"),
    author = env!("CARGO_PKG_AUTHORS"),
    about = env!("CARGO_PKG_DESCRIPTION"),
)]
struct Entry {
    /// File path
    files: Vec<String>,
    #[clap(subcommand)]
    subcommand: Option<Subcommand>,
}

#[derive(clap::Subcommand)]
enum Subcommand {
    /// Download the latest WebAssembly archive
    Update,
    /// Start web server to host WebAssembly UI program
    Serve {
        /// Port number
        #[clap(long, default_value = "8080")]
        port: u16,
        /// Open the server
        #[clap(long)]
        open: bool,
    },
    /// Run native UI program (default)
    Ui {
        /// File path
        files: Vec<String>,
    },
}

fn main() -> Result<()> {
    let args = Entry::parse();
    match args.subcommand {
        Some(Subcommand::Update) => update::update(),
        Some(Subcommand::Serve { port, open }) => serve::serve(port, open),
        Some(Subcommand::Ui { files }) => run_native(files),
        None => run_native(args.files),
    }
}

fn run_native(files: Vec<String>) -> ! {
    let icon = image::load_from_memory_with_format(ICON, ImageFormat::Png).unwrap();
    let opt = NativeOptions {
        icon_data: Some(IconData {
            width: icon.width(),
            height: icon.height(),
            rgba: icon.into_bytes(),
        }),
        ..Default::default()
    };
    #[cfg(windows)]
    let _ = unsafe { winapi::um::wincon::FreeConsole() };
    eframe::run_native(
        "Four bar",
        opt,
        Box::new(|ctx| Box::new(App::new(ctx, files))),
    )
}
