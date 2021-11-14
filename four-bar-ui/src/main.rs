use eframe::{epi::IconData, NativeOptions};
use four_bar_ui::{icon, App};

/// Native entry point.
fn main() -> ! {
    let app = Box::new(App::default());
    let opt = NativeOptions {
        icon_data: Some(IconData {
            rgba: Vec::from(icon::ICON),
            width: icon::WIDTH,
            height: icon::HEIGHT,
        }),
        ..Default::default()
    };
    eframe::run_native(app, opt)
}
