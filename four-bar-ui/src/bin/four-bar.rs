use clap::{clap_app, AppSettings};
use eframe::{epi::IconData, NativeOptions};
use four_bar_ui::{
    server::{serve, update},
    App,
};
use std::io::Result;

mod icon {
    include!(concat!(env!("OUT_DIR"), "/icon.rs"));
}

/// Native entry point.
#[actix_web::main]
async fn main() -> Result<()> {
    let args = clap_app! {
        (env!("CARGO_PKG_NAME")) =>
        (version: env!("CARGO_PKG_VERSION"))
        (author: env!("CARGO_PKG_AUTHORS"))
        (about: env!("CARGO_PKG_DESCRIPTION"))
        (setting: AppSettings::ArgRequiredElseHelp)
        (@subcommand ui =>
            (about: "Run native UI program")
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
    if args.subcommand_matches("ui").is_some() {
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
    } else if args.subcommand_matches("update").is_some() {
        update().await
    } else if let Some(cmd) = args.subcommand_matches("serve") {
        let port = cmd
            .value_of("PORT")
            .unwrap_or("8080")
            .parse()
            .expect("invalid port");
        serve(port).await
    } else {
        unreachable!()
    }
}
