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
    #[cfg(target_os = "linux")]
    {
        if let Err(e) = gtk::init() {
            log::error!("Failed to initialize GTK: {}", e);
        }
    }

    env_logger::Builder::from_env(
        env_logger::Env::default().default_filter_or("warn,dockstack=info"),
    )
    .format_timestamp_secs()
    .init();

    log::info!("Starting DockStack v0.1.0");

    let icon = utils::load_icon();

    let mut viewport = egui::ViewportBuilder::default()
        .with_inner_size([1280.0, 800.0])
        .with_min_inner_size([900.0, 600.0])
        .with_title("DockStack - DevStack Manager")
        .with_app_id("com.dockstack.manager");

    if let Some(icon) = icon {
        viewport = viewport.with_icon(icon);
    }

    let options = eframe::NativeOptions {
        viewport,
        persistence_path: None,
        ..Default::default()
    };

    eframe::run_native(
        "DockStack",
        options,
        Box::new(|cc| Ok(Box::new(DockStackApp::new(cc)))),
    )
}
