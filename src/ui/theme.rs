#![allow(dead_code)]
use egui::{Color32, Stroke, Vec2, FontDefinitions, epaint::Shadow, Margin, Rounding};

// Premium Modern Dark Theme - High Clarity
pub const COLOR_BG_APP: Color32 = Color32::from_rgb(12, 12, 16);         // Deep dark blue-black
pub const COLOR_BG_PANEL: Color32 = Color32::from_rgb(24, 24, 32);       // Distinct dark panel
pub const COLOR_BG_CARD: Color32 = Color32::from_rgb(32, 32, 42);        // Lighter for cards
pub const COLOR_BG_HOVER: Color32 = Color32::from_rgb(45, 45, 55);       // Clear hover state
pub const COLOR_BG_ACTIVE: Color32 = Color32::from_rgb(60, 60, 75);      // Distinct active state

// Accents - More Vivid
pub const COLOR_PRIMARY: Color32 = Color32::from_rgb(100, 110, 255);     // Vivid Indigo-Blue
pub const COLOR_PRIMARY_HOVER: Color32 = Color32::from_rgb(130, 140, 255); // Brighter Indigo
pub const COLOR_SECONDARY: Color32 = Color32::from_rgb(245, 80, 160);    // Vivid Pink
pub const COLOR_ACCENT: Color32 = Color32::from_rgb(14, 180, 245);       // Vivid Sky

// Status - Higher Saturation
pub const COLOR_SUCCESS: Color32 = Color32::from_rgb(40, 215, 100);      // Bright Green
pub const COLOR_WARNING: Color32 = Color32::from_rgb(250, 190, 20);      // Bright Yellow/Orange
pub const COLOR_ERROR: Color32 = Color32::from_rgb(245, 80, 80);         // Bright Red
pub const COLOR_INFO: Color32 = Color32::from_rgb(60, 140, 255);         // Bright Blue

// Text - Higher Contrast
pub const COLOR_TEXT: Color32 = Color32::from_rgb(255, 255, 255);        // Pure White
pub const COLOR_TEXT_DIM: Color32 = Color32::from_rgb(170, 180, 200);    // Light Blue-Grey
pub const COLOR_TEXT_MUTED: Color32 = Color32::from_rgb(100, 110, 130);  // Readable Muted

// Borders & Separators
pub const COLOR_BORDER: Color32 = Color32::from_rgb(50, 50, 60);         // Visible Border
pub const COLOR_BORDER_LIGHT: Color32 = Color32::from_rgb(70, 70, 80);   // Lighter Border

// Sidebar specific
pub const COLOR_SIDEBAR: Color32 = COLOR_BG_PANEL;
pub const COLOR_SIDEBAR_ACTIVE: Color32 = Color32::from_rgba_premultiplied(100, 110, 255, 50); // More visible selection

pub fn apply_theme(ctx: &egui::Context) {
    let mut style = (*ctx.style()).clone();

    // Spacing & Layout
    style.spacing.item_spacing = Vec2::new(10.0, 10.0);
    style.spacing.button_padding = Vec2::new(18.0, 8.0);
    style.spacing.indent = 20.0;
    style.spacing.interact_size = Vec2::new(40.0, 32.0); // Larger touch targets
    style.spacing.window_margin = Margin::same(0); 

    // Visuals
    style.visuals.dark_mode = true;
    style.visuals.override_text_color = Some(COLOR_TEXT);
    style.visuals.window_fill = COLOR_BG_APP;
    style.visuals.panel_fill = COLOR_BG_PANEL;
    
    // Smooth Rounding
    let corner_radius = egui::CornerRadius::same(8);
    style.visuals.window_corner_radius = corner_radius;
    style.visuals.menu_corner_radius = corner_radius;
    
    // Shadows
    style.visuals.window_shadow = Shadow {
        offset: [0, 8],
        blur: 24,
        spread: 0,
        color: Color32::from_black_alpha(120),
    };
    style.visuals.popup_shadow = Shadow {
        offset: [0, 4],
        blur: 12,
        spread: 0,
        color: Color32::from_black_alpha(80),
    };
    
    // Selection
    style.visuals.selection.bg_fill = COLOR_PRIMARY.gamma_multiply(0.3);
    style.visuals.selection.stroke = Stroke::new(1.0, COLOR_PRIMARY);

    // Widget Styles
    style.visuals.widgets.noninteractive.bg_fill = COLOR_BG_PANEL;
    style.visuals.widgets.noninteractive.weak_bg_fill = COLOR_BG_APP;
    style.visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0, COLOR_TEXT_DIM);
    style.visuals.widgets.noninteractive.corner_radius = corner_radius;
    style.visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0, COLOR_BORDER);

    style.visuals.widgets.inactive.bg_fill = COLOR_BG_CARD; // Cards use this
    style.visuals.widgets.inactive.weak_bg_fill = COLOR_BG_PANEL;
    style.visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, COLOR_TEXT_DIM);
    style.visuals.widgets.inactive.corner_radius = corner_radius;
    style.visuals.widgets.inactive.bg_stroke = Stroke::new(1.0, COLOR_BORDER);

    style.visuals.widgets.hovered.bg_fill = COLOR_BG_HOVER;
    style.visuals.widgets.hovered.weak_bg_fill = COLOR_BG_HOVER;
    style.visuals.widgets.hovered.fg_stroke = Stroke::new(1.0, COLOR_TEXT);
    style.visuals.widgets.hovered.corner_radius = corner_radius;
    style.visuals.widgets.hovered.bg_stroke = Stroke::new(1.0, COLOR_BORDER_LIGHT);

    style.visuals.widgets.active.bg_fill = COLOR_BG_ACTIVE;
    style.visuals.widgets.active.weak_bg_fill = COLOR_BG_ACTIVE;
    style.visuals.widgets.active.fg_stroke = Stroke::new(1.0, COLOR_TEXT);
    style.visuals.widgets.active.corner_radius = corner_radius;
    style.visuals.widgets.active.bg_stroke = Stroke::new(1.0, COLOR_PRIMARY);
    
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
