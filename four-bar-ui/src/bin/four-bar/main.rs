use clap::Parser;
use eframe::{epi::IconData, NativeOptions};
use four_bar_ui::App;
use std::io::Result;

mod serve;
mod update;
mod icon {
    include!(concat![env!("OUT_DIR"), "/icon.rs"]);
}

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
        Some(Subcommand::Serve { port }) => serve::serve(port),
        Some(Subcommand::Ui { files }) => run_native(files),
        None => run_native(args.files),
    }
}

fn run_native(files: Vec<String>) -> ! {
    let app = Box::new(App::open(files));
    let opt = NativeOptions {
        icon_data: Some(IconData {
            rgba: icon::ICON.to_vec(),
            width: icon::WIDTH,
            height: icon::HEIGHT,
        }),
        ..Default::default()
    };
    #[cfg(windows)]
    let _ = unsafe { winapi::um::wincon::FreeConsole() };
    eframe::run_native(app, opt)
}
