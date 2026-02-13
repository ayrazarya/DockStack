#![allow(dead_code)]
use egui::{Color32, Stroke, Vec2, FontDefinitions, epaint::Shadow, Margin};

// Modern Dark Theme (Slate/Midnight inspired)
pub const COLOR_BG_APP: Color32 = Color32::from_rgb(15, 23, 42);       // Slate 900 (Main Background)
pub const COLOR_BG_PANEL: Color32 = Color32::from_rgb(30, 41, 59);     // Slate 800 (Sidebar/Panels)
pub const COLOR_BG_CARD: Color32 = Color32::from_rgb(30, 41, 59);      // Transparent/Lite for cards
pub const COLOR_BG_HOVER: Color32 = Color32::from_rgb(51, 65, 85);     // Slate 700 (Hover state)

pub const COLOR_PRIMARY: Color32 = Color32::from_rgb(56, 189, 248);    // Sky 400 (Primary Brand)
pub const COLOR_PRIMARY_HOVER: Color32 = Color32::from_rgb(14, 165, 233); // Sky 500
pub const COLOR_SECONDARY: Color32 = Color32::from_rgb(168, 85, 247);  // Purple 500 (Secondary)

pub const COLOR_SUCCESS: Color32 = Color32::from_rgb(34, 197, 94);     // Green 500
pub const COLOR_WARNING: Color32 = Color32::from_rgb(234, 179, 8);     // Yellow 500
pub const COLOR_ERROR: Color32 = Color32::from_rgb(239, 68, 68);       // Red 500
pub const COLOR_INFO: Color32 = Color32::from_rgb(59, 130, 246);       // Blue 500

pub const COLOR_TEXT: Color32 = Color32::from_rgb(241, 245, 249);      // Slate 100 (Primary Text)
pub const COLOR_TEXT_DIM: Color32 = Color32::from_rgb(148, 163, 184);  // Slate 400 (Secondary Text)
pub const COLOR_TEXT_MUTED: Color32 = Color32::from_rgb(100, 116, 139); // Slate 500 (Disabled/Muted)

pub const COLOR_BORDER: Color32 = Color32::from_rgb(51, 65, 85);       // Slate 700 (Borders)
pub const COLOR_BORDER_LIGHT: Color32 = Color32::from_rgb(71, 85, 105); // Slate 600

pub const COLOR_SIDEBAR: Color32 = Color32::from_rgb(15, 23, 42);      // Same as App BG for continuity
pub const COLOR_SIDEBAR_ACTIVE: Color32 = Color32::from_rgb(30, 41, 59); // Active tab bg

pub fn apply_theme(ctx: &egui::Context) {
    let mut style = (*ctx.style()).clone();

    // Spacing & Layout
    style.spacing.item_spacing = Vec2::new(12.0, 12.0);
    style.spacing.button_padding = Vec2::new(16.0, 10.0);
    style.spacing.indent = 24.0;
    style.spacing.interact_size = Vec2::new(40.0, 24.0);
    style.spacing.window_margin = Margin::same(16); // Fix: use i8

    // Visuals
    style.visuals.dark_mode = true;
    style.visuals.override_text_color = Some(COLOR_TEXT);
    style.visuals.window_fill = COLOR_BG_APP;
    style.visuals.panel_fill = COLOR_BG_PANEL;
    style.visuals.window_shadow = Shadow {
        offset: [0, 8],
        blur: 16,
        spread: 0,
        color: Color32::from_black_alpha(96),
    };
    style.visuals.popup_shadow = Shadow {
        offset: [0, 4],
        blur: 8,
        spread: 0,
        color: Color32::from_black_alpha(64),
    };
    
    // Selection
    style.visuals.selection.bg_fill = COLOR_PRIMARY;
    style.visuals.selection.stroke = Stroke::new(1.0, COLOR_PRIMARY);

    // Widgets Styling
    style.visuals.widgets.noninteractive.bg_fill = COLOR_BG_PANEL;
    style.visuals.widgets.noninteractive.weak_bg_fill = COLOR_BG_APP;
    style.visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0, COLOR_TEXT_DIM);
    style.visuals.widgets.noninteractive.corner_radius = egui::CornerRadius::same(6); // Fix: corner_radius field
    style.visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0, COLOR_BORDER);

    style.visuals.widgets.inactive.bg_fill = COLOR_BG_PANEL;
    style.visuals.widgets.inactive.weak_bg_fill = COLOR_BG_PANEL;
    style.visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, COLOR_TEXT_DIM);
    style.visuals.widgets.inactive.corner_radius = egui::CornerRadius::same(6);
    style.visuals.widgets.inactive.bg_stroke = Stroke::new(1.0, COLOR_BORDER);

    style.visuals.widgets.hovered.bg_fill = COLOR_BG_HOVER;
    style.visuals.widgets.hovered.weak_bg_fill = COLOR_BG_HOVER;
    style.visuals.widgets.hovered.fg_stroke = Stroke::new(1.0, COLOR_TEXT);
    style.visuals.widgets.hovered.corner_radius = egui::CornerRadius::same(6);
    style.visuals.widgets.hovered.bg_stroke = Stroke::new(1.0, COLOR_BORDER_LIGHT);

    style.visuals.widgets.active.bg_fill = COLOR_PRIMARY;
    style.visuals.widgets.active.weak_bg_fill = COLOR_PRIMARY;
    style.visuals.widgets.active.fg_stroke = Stroke::new(1.0, Color32::WHITE);
    style.visuals.widgets.active.corner_radius = egui::CornerRadius::same(6);
    style.visuals.widgets.active.bg_stroke = Stroke::NONE;
    
    style.visuals.widgets.open.bg_fill = COLOR_BG_PANEL;
    style.visuals.widgets.open.bg_stroke = Stroke::new(1.0, COLOR_BORDER_LIGHT);
    style.visuals.widgets.open.corner_radius = egui::CornerRadius::same(6);

    ctx.set_style(style);
    
    // Use default fonts for now, but style is applied
    let fonts = FontDefinitions::default();
    ctx.set_fonts(fonts);
}

pub fn status_color(running: bool) -> Color32 {
    if running {
        COLOR_SUCCESS
    } else {
        COLOR_TEXT_MUTED
    }
}
