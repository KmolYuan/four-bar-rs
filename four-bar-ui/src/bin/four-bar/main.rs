use clap::Parser;
use eframe::{epi::IconData, NativeOptions};
use four_bar_ui::App;
use std::io::Result;

mod serve;
mod update;
mod icon {
    include!(concat!(env!("OUT_DIR"), "/icon.rs"));
}

#[derive(clap::Parser)]
#[clap(
    name = "four-bar",
    version = env!("CARGO_PKG_VERSION"),
    author = env!("CARGO_PKG_AUTHORS"),
    about = env!("CARGO_PKG_DESCRIPTION"),
)]
struct Entry {
    /// File path
    file: Option<String>,
    #[clap(subcommand)]
    subcommand: Option<Subcommand>,
}

#[derive(clap::Subcommand)]
enum Subcommand {
    /// Download the latest WebAssembly archive
    Update,
    /// Start web server to host WebAssembly UI program
    Serve {
        /// Set port
        #[clap(long, default_value = "8080")]
        port: u16,
    },
    /// Run native UI program (default)
    Ui {
        /// File path
        file: Option<String>,
    },
}

fn main() -> Result<()> {
    let args = Entry::parse();
    match args.subcommand {
        Some(Subcommand::Update) => update::update(),
        Some(Subcommand::Serve { port }) => serve::serve(port),
        Some(Subcommand::Ui { file }) => run_native(file),
        None => run_native(args.file),
    }
}

fn run_native(file: Option<String>) -> ! {
    let app = Box::new(App::open(file));
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
