use clap::clap_app;
use eframe::{epi::IconData, NativeOptions};
use four_bar_ui::App;
use std::io::Result;

mod serve;
mod update;
mod icon {
    include!(concat!(env!("OUT_DIR"), "/icon.rs"));
}

/// Native entry point.
#[actix_web::main]
async fn main() -> Result<()> {
    let args = clap_app! {
        ("four-bar") =>
        (version: env!("CARGO_PKG_VERSION"))
        (author: env!("CARGO_PKG_AUTHORS"))
        (about: env!("CARGO_PKG_DESCRIPTION"))
        (@subcommand ui =>
            (about: "Run native UI program (default)")
        )
        (@subcommand update =>
            (about: "Download the latest WASM archive")
        )
        (@subcommand serve =>
            (about: "Start web server to host WASM UI program")
            (@arg PORT: --port +takes_value "Set port")
        )
    }
    .get_matches();
    if args.subcommand_matches("update").is_some() {
        update::update().await
    } else if let Some(cmd) = args.subcommand_matches("serve") {
        let port = cmd
            .value_of("PORT")
            .unwrap_or("8080")
            .parse()
            .expect("invalid port");
        serve::serve(port).await
    } else {
        let app = Box::new(App::default());
        let opt = NativeOptions {
            icon_data: Some(IconData {
                rgba: icon::ICON.to_vec(),
                width: icon::WIDTH,
                height: icon::HEIGHT,
            }),
            ..Default::default()
        };
        eframe::run_native(app, opt)
    }
}
