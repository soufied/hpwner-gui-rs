mod app;
mod backend;
mod settings;
mod ui;

use app::HpwnrApp;
use eframe::egui;
use settings::Settings;

fn main() -> eframe::Result<()> {
    let settings = Settings::load();

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([settings.window_width, settings.window_height])
            .with_min_inner_size([540.0, 420.0])
            .with_title("hpwnr GUI"),
        ..Default::default()
    };

    eframe::run_native(
        "hpwnr GUI",
        native_options,
        Box::new(move |cc| Box::new(HpwnrApp::new(cc, settings))),
    )
}
