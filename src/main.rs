mod config;
mod docker;
mod monitor;
mod port_scanner;
mod services;
mod ssl;
mod terminal;
mod tray;
mod ui;
mod utils;

use ui::app::DockStackApp;

fn main() -> eframe::Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .format_timestamp_secs()
        .init();

    log::info!("Starting DockStack v0.1.0");

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1280.0, 800.0])
            .with_min_inner_size([900.0, 600.0])
            .with_title("DockStack - DevStack Manager")
            .with_app_id("com.dockstack.manager"),
        persistence_path: None,
        ..Default::default()
    };

    eframe::run_native(
        "DockStack",
        options,
        Box::new(|cc| Ok(Box::new(DockStackApp::new(cc)))),
    )
}
