#![allow(dead_code)]
// System tray integration
// Note: tray-icon requires the event loop to run on the main thread.
// We provide the setup functions and menu builders here.

use tray_icon::{
    menu::{Menu, MenuEvent, MenuItem, PredefinedMenuItem},
    TrayIcon, TrayIconBuilder,
};
use crossbeam_channel::{Sender, Receiver};

#[derive(Debug, Clone)]
pub enum TrayCommand {
    Start,
    Stop,
    Restart,
    OpenUI,
    Quit,
}

pub struct SystemTray {
    pub command_tx: Sender<TrayCommand>,
    pub command_rx: Receiver<TrayCommand>,
    tray_icon: Option<TrayIcon>,
}

impl SystemTray {
    pub fn new() -> Self {
        let (command_tx, command_rx) = crossbeam_channel::unbounded();
        Self {
            command_tx,
            command_rx,
            tray_icon: None,
        }
    }

    pub fn setup(&mut self) -> Result<(), String> {
        let menu = Menu::new();

        let start_item = MenuItem::new("▶ Start Services", true, None);
        let stop_item = MenuItem::new("⏹ Stop Services", true, None);
        let restart_item = MenuItem::new("🔄 Restart Services", true, None);
        let separator = PredefinedMenuItem::separator();
        let open_item = MenuItem::new("📱 Open DockStack", true, None);
        let separator2 = PredefinedMenuItem::separator();
        let quit_item = MenuItem::new("❌ Quit", true, None);

        menu.append(&start_item).map_err(|e| e.to_string())?;
        menu.append(&stop_item).map_err(|e| e.to_string())?;
        menu.append(&restart_item).map_err(|e| e.to_string())?;
        menu.append(&separator).map_err(|e| e.to_string())?;
        menu.append(&open_item).map_err(|e| e.to_string())?;
        menu.append(&separator2).map_err(|e| e.to_string())?;
        menu.append(&quit_item).map_err(|e| e.to_string())?;

        let start_id = start_item.id().clone();
        let stop_id = stop_item.id().clone();
        let restart_id = restart_item.id().clone();
        let open_id = open_item.id().clone();
        let quit_id = quit_item.id().clone();

        // Use the app icon if available, otherwise fallback to generated icon
        let icon = if let Some(icon_data) = crate::utils::load_icon() {
            tray_icon::Icon::from_rgba(icon_data.rgba, icon_data.width, icon_data.height)
                .map_err(|e| format!("Failed to create tray icon: {}", e))?
        } else {
            let icon_rgba = create_tray_icon_data();
            tray_icon::Icon::from_rgba(icon_rgba, 16, 16)
                .map_err(|e| format!("Failed to create fallback icon: {}", e))?
        };

        let tray = TrayIconBuilder::new()
            .with_menu(Box::new(menu))
            .with_tooltip("DockStack - DevStack Manager")
            .with_icon(icon)
            .build()
            .map_err(|e| format!("Failed to build tray icon: {}", e))?;

        self.tray_icon = Some(tray);

        // Spawn menu event handler
        let tx = self.command_tx.clone();
        std::thread::spawn(move || {
            loop {
                if let Ok(event) = MenuEvent::receiver().recv() {
                    if event.id() == &start_id {
                        tx.send(TrayCommand::Start).ok();
                    } else if event.id() == &stop_id {
                        tx.send(TrayCommand::Stop).ok();
                    } else if event.id() == &restart_id {
                        tx.send(TrayCommand::Restart).ok();
                    } else if event.id() == &open_id {
                        tx.send(TrayCommand::OpenUI).ok();
                    } else if event.id() == &quit_id {
                        tx.send(TrayCommand::Quit).ok();
                    }
                }
            }
        });

        Ok(())
    }
}

fn create_tray_icon_data() -> Vec<u8> {
    let size = 16usize;
    let mut data = Vec::with_capacity(size * size * 4);
    for y in 0..size {
        for x in 0..size {
            // Create a simple rounded square icon with gradient
            let cx = (x as f32 - 7.5).abs();
            let cy = (y as f32 - 7.5).abs();
            let dist = cx.max(cy);

            if dist < 6.0 {
                // Inner area - gradient from teal to blue
                let t = (y as f32) / size as f32;
                let r = (40.0 + t * 20.0) as u8;
                let g = (180.0 - t * 40.0) as u8;
                let b = (220.0 + t * 35.0) as u8;
                data.extend_from_slice(&[r, g, b, 255]);
            } else if dist < 7.0 {
                // Border
                data.extend_from_slice(&[60, 160, 200, 200]);
            } else {
                // Transparent
                data.extend_from_slice(&[0, 0, 0, 0]);
            }
        }
    }
    data
}
