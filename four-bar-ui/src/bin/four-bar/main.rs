use clap::Parser;
use eframe::{IconData, NativeOptions};
use four_bar_ui::App;
use image::ImageFormat;
use std::io::Result;

mod serve;
mod update;

const ICON: &[u8] = include_bytes!("../../../assets/favicon.png");

#[derive(Parser)]
#[clap(name = "four-bar", version, author, about)]
struct Entry {
    /// File path
    files: Vec<String>,
    #[clap(subcommand)]
    cmd: Option<Cmd>,
}

#[derive(clap::Subcommand)]
enum Cmd {
    /// Download the latest WebAssembly archive
    Update,
    /// Start web server to host WebAssembly UI program
    Serve {
        /// Port number
        #[clap(long, default_value_t = 8080)]
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
    match args.cmd {
        Some(Cmd::Update) => update::update(),
        Some(Cmd::Serve { port, open }) => serve::serve(port, open),
        Some(Cmd::Ui { files }) => run_native(files),
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
    eframe::run_native("Four bar", opt, Box::new(|ctx| App::new(ctx, files)))
}
