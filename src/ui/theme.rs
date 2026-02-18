#![allow(dead_code)]
use egui::{Color32, Stroke, Vec2, FontDefinitions, epaint::Shadow, Margin};

// Premium Midnight Tech Theme - Deep Slate & Cyber Accents
pub const COLOR_BG_APP: Color32 = Color32::from_rgb(10, 12, 18);          // Deep Space
pub const COLOR_BG_PANEL: Color32 = Color32::from_rgb(18, 20, 30);        // Midnight Blue-Grey
pub const COLOR_BG_CARD: Color32 = Color32::from_rgb(26, 29, 44);         // Sophisticated Navy
pub const COLOR_BG_HOVER: Color32 = Color32::from_rgb(38, 42, 62);        // Subtle elevation
pub const COLOR_BG_ACTIVE: Color32 = Color32::from_rgb(45, 50, 75);       // Clear active state

// Accents - SHARP & MODERN
pub const COLOR_PRIMARY: Color32 = Color32::from_rgb(0, 220, 255);        // Cyber Cyan
pub const COLOR_PRIMARY_HOVER: Color32 = Color32::from_rgb(100, 240, 255); 
pub const COLOR_SECONDARY: Color32 = Color32::from_rgb(180, 100, 255);    // Modern Purple
pub const COLOR_ACCENT: Color32 = Color32::from_rgb(255, 60, 140);        // Vivid Rose

// Status - Refined but Clear
pub const COLOR_SUCCESS: Color32 = Color32::from_rgb(0, 255, 140);        // Spring Green
pub const COLOR_WARNING: Color32 = Color32::from_rgb(255, 200, 50);       // Amber
pub const COLOR_ERROR: Color32 = Color32::from_rgb(255, 70, 100);         // Coral Red
pub const COLOR_INFO: Color32 = Color32::from_rgb(50, 150, 255);          // Sky Blue

// Text - Optimal Contrast
pub const COLOR_TEXT: Color32 = Color32::from_rgb(255, 255, 255);         // True White
pub const COLOR_TEXT_DIM: Color32 = Color32::from_rgb(160, 175, 200);     // Cool Grey
pub const COLOR_TEXT_MUTED: Color32 = Color32::from_rgb(90, 105, 125);    // Dark Slate Grey

// Borders & Separators - Distinct Definition
pub const COLOR_BORDER: Color32 = Color32::from_rgb(45, 52, 70);          // Slate Border
pub const COLOR_BORDER_LIGHT: Color32 = Color32::from_rgb(70, 80, 110);   // Glowing Border

// Sidebar specific
pub const COLOR_SIDEBAR: Color32 = COLOR_BG_PANEL;
pub const COLOR_SIDEBAR_ACTIVE: Color32 = Color32::from_rgb(25, 30, 45); // Solid dark navy
pub const COLOR_SIDEBAR_BORDER: Color32 = Color32::from_rgb(0, 180, 220); // Muted cyan for border

pub fn apply_theme(ctx: &egui::Context) {
    let mut style = (*ctx.style()).clone();

    // Spacing & Layout - Premium Flow
    style.spacing.item_spacing = Vec2::new(14.0, 14.0);
    style.spacing.button_padding = Vec2::new(22.0, 12.0);
    style.spacing.indent = 24.0;
    style.spacing.interact_size = Vec2::new(44.0, 38.0);
    style.spacing.window_margin = Margin::same(0); 

    // Visuals
    style.visuals.dark_mode = true;
    style.visuals.override_text_color = Some(COLOR_TEXT);
    style.visuals.window_fill = COLOR_BG_APP;
    style.visuals.panel_fill = COLOR_BG_PANEL;
    
    // Smooth Rounding - Modern Curves
    let corner_radius = egui::CornerRadius::same(12);
    style.visuals.window_corner_radius = corner_radius;
    style.visuals.menu_corner_radius = corner_radius;
    
    // Shadows - Sophisticated Depth
    style.visuals.window_shadow = Shadow {
        offset: [0, 14],
        blur: 40,
        spread: 0,
        color: Color32::from_black_alpha(180),
    };
    style.visuals.popup_shadow = Shadow {
        offset: [0, 6],
        blur: 16,
        spread: 0,
        color: Color32::from_black_alpha(120),
    };
    
    // Selection
    style.visuals.selection.bg_fill = COLOR_PRIMARY.gamma_multiply(0.2);
    style.visuals.selection.stroke = Stroke::new(2.0, COLOR_PRIMARY);

    // Widget Styles - Definition
    style.visuals.widgets.noninteractive.bg_fill = COLOR_BG_PANEL;
    style.visuals.widgets.noninteractive.weak_bg_fill = COLOR_BG_APP;
    style.visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0, COLOR_TEXT_DIM);
    style.visuals.widgets.noninteractive.corner_radius = corner_radius;
    style.visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0, COLOR_BORDER);

    style.visuals.widgets.inactive.bg_fill = COLOR_BG_CARD;
    style.visuals.widgets.inactive.weak_bg_fill = COLOR_BG_PANEL;
    style.visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, COLOR_TEXT_DIM); // Brighter text on buttons
    style.visuals.widgets.inactive.corner_radius = corner_radius;
    style.visuals.widgets.inactive.bg_stroke = Stroke::new(1.0, COLOR_BORDER); // Visible borders on buttons

    style.visuals.widgets.hovered.bg_fill = COLOR_BG_HOVER;
    style.visuals.widgets.hovered.weak_bg_fill = COLOR_BG_HOVER;
    style.visuals.widgets.hovered.fg_stroke = Stroke::new(1.0, COLOR_TEXT);
    style.visuals.widgets.hovered.corner_radius = corner_radius;
    style.visuals.widgets.hovered.bg_stroke = Stroke::new(1.5, COLOR_BORDER_LIGHT); // Glowing border on hover

    style.visuals.widgets.active.bg_fill = COLOR_BG_ACTIVE;
    style.visuals.widgets.active.weak_bg_fill = COLOR_BG_ACTIVE;
    style.visuals.widgets.active.fg_stroke = Stroke::new(1.0, COLOR_TEXT);
    style.visuals.widgets.active.corner_radius = corner_radius;
    style.visuals.widgets.active.bg_stroke = Stroke::new(2.0, COLOR_PRIMARY); // Sharp accent on active
    
    style.visuals.widgets.open.bg_fill = COLOR_BG_PANEL;
    style.visuals.widgets.open.bg_stroke = Stroke::new(1.0, COLOR_BORDER_LIGHT);
    style.visuals.widgets.open.corner_radius = corner_radius;

    ctx.set_style(style);
    
    // Font setup (using default egui fonts but configured if we had assets)
    let fonts = FontDefinitions::default();
    ctx.set_fonts(fonts);
}

pub fn status_color(running: bool) -> Color32 {
    if running {
        COLOR_SUCCESS
    } else {
        COLOR_TEXT_DIM // Muted looks better for stopped than Red
    }
}
