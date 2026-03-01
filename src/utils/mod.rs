// utils/mod.rs
#[allow(dead_code)]
pub fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.1} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.1} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.1} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

pub fn truncate_string(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len - 3])
    }
}

pub fn open_url(url: &str) {
    if let Err(e) = open::that(url) {
        log::error!("Failed to open URL {}: {}", url, e);
    }
}

pub fn open_directory(path: &str) {
    let path_buf = std::path::PathBuf::from(path);
    if !path_buf.exists() {
        if let Err(e) = std::fs::create_dir_all(&path_buf) {
            log::error!("Failed to create directory {}: {}", path, e);
            return;
        }
    }

    if let Err(e) = open::that(path) {
        log::error!("Failed to open directory {}: {}", path, e);
    }
}

pub fn load_icon() -> Option<egui::IconData> {
    let icon_data = include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/assets/images/icon.png"
    ));
    match image::load_from_memory(icon_data) {
        Ok(img) => {
            let rgba = img.to_rgba8();
            let (width, height) = rgba.dimensions();
            Some(egui::IconData {
                rgba: rgba.into_raw(),
                width,
                height,
            })
        }
        Err(e) => {
            log::error!("Failed to load icon from memory: {}", e);
            None
        }
    }
}

