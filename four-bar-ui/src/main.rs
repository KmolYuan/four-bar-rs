use eframe::{epi::IconData, NativeOptions};
use four_bar_ui::App;

mod icon {
    include!(concat!(env!("OUT_DIR"), "/icon.rs"));
}

/// Native entry point.
fn main() -> ! {
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
