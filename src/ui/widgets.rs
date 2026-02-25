#![allow(dead_code)]
use egui::{Color32, Pos2, Stroke, Ui, Vec2, RichText};
use crate::ui::theme::*;

/// Draw a status indicator dot
pub fn status_dot(ui: &mut Ui, running: bool) -> egui::Response {
    let size = Vec2::new(10.0, 10.0);
    let (rect, response) = ui.allocate_exact_size(size, egui::Sense::hover());

    if ui.is_rect_visible(rect) {
        let center = rect.center();
        let color = if running { COLOR_SUCCESS } else { COLOR_TEXT_MUTED };

        if running {
            ui.painter().circle_filled(center, 6.0, COLOR_SUCCESS.gamma_multiply(0.3));
        }
        ui.painter().circle_filled(center, 4.0, color);
    }

    response
}

/// Draw a card container
/// Draw a card container - Modern Minimalist
pub fn card_frame(ui: &mut Ui, add_contents: impl FnOnce(&mut Ui)) {
    egui::Frame::new()
        .fill(COLOR_BG_CARD)
        .corner_radius(egui::CornerRadius::same(12))
        .stroke(Stroke::new(1.0, COLOR_BORDER))
        .shadow(egui::epaint::Shadow {
            offset: [0, 4],
            blur: 15,
            spread: 0,
            color: Color32::from_black_alpha(10),
        })
        .inner_margin(egui::Margin::same(20))
        .show(ui, |ui| {
            add_contents(ui);
        });
}

/// Draw a styled button - Primary
pub fn primary_button(ui: &mut Ui, text: &str) -> egui::Response {
    let button = egui::Button::new(
        egui::RichText::new(text).color(Color32::WHITE).size(13.0).strong(),
    )
    .fill(COLOR_PRIMARY)
    .corner_radius(egui::CornerRadius::same(8))
    .min_size(Vec2::new(0.0, 36.0)) // Taller button
    .stroke(Stroke::NONE);

    ui.add(button)
}

/// Draw a styled button - Danger
pub fn danger_button(ui: &mut Ui, text: &str) -> egui::Response {
    let button = egui::Button::new(
        egui::RichText::new(text).color(Color32::WHITE).size(13.0).strong(),
    )
    .fill(COLOR_ERROR)
    .corner_radius(egui::CornerRadius::same(8))
    .min_size(Vec2::new(0.0, 36.0))
    .stroke(Stroke::NONE);

    ui.add(button)
}

/// Draw a styled button - Secondary
pub fn secondary_button(ui: &mut Ui, text: &str) -> egui::Response {
    let button = egui::Button::new(
        egui::RichText::new(text).color(COLOR_TEXT).size(13.0),
    )
    .fill(Color32::TRANSPARENT) // Ghost button style
    .corner_radius(egui::CornerRadius::same(6))
    .min_size(Vec2::new(0.0, 32.0))
    .stroke(Stroke::new(1.0, COLOR_BORDER));

    ui.add(button)
}

/// Draw a simple sparkline graph
pub fn sparkline(ui: &mut Ui, values: &[f32], max_val: f32, color: Color32, size: Vec2) {
    let (rect, _response) = ui.allocate_exact_size(size, egui::Sense::hover());

    if ui.is_rect_visible(rect) && !values.is_empty() {
        let painter = ui.painter();

        painter.rect_filled(rect, egui::CornerRadius::same(4), COLOR_BG_CARD);

        let n = values.len();
        if n < 2 {
            return;
        }

        let mut points = Vec::with_capacity(n);
        for (i, &val) in values.iter().enumerate() {
            let x = rect.left() + (i as f32 / (n - 1) as f32) * rect.width();
            let y = rect.bottom() - (val / max_val).clamp(0.0, 1.0) * rect.height();
            points.push(Pos2::new(x, y));
        }

        // Draw fill (trapezoids)
        let fill_color = Color32::from_rgba_unmultiplied(
            color.r(),
            color.g(),
            color.b(),
            20,
        );

        for i in 1..points.len() {
             let p1 = points[i - 1];
             let p2 = points[i];
             let b1 = Pos2::new(p1.x, rect.bottom());
             let b2 = Pos2::new(p2.x, rect.bottom());

             painter.add(egui::Shape::convex_polygon(
                 vec![p1, p2, b2, b1],
                 fill_color,
                 Stroke::NONE,
             ));
        }

        // Draw line
        painter.add(egui::Shape::line(points, Stroke::new(1.5, color)));
    }
}

/// Section header
pub fn section_header(ui: &mut Ui, text: &str) {
    ui.add_space(4.0);
    ui.label(
        egui::RichText::new(text)
            .size(16.0)
            .color(COLOR_TEXT)
            .strong(),
    );
    ui.add_space(2.0);
}

/// Styled toggle switch
pub fn toggle_switch(ui: &mut Ui, on: &mut bool) -> egui::Response {
    let desired_size = Vec2::new(36.0, 20.0);
    let (rect, mut response) = ui.allocate_exact_size(desired_size, egui::Sense::click());

    if response.clicked() {
        *on = !*on;
        response.mark_changed();
    }

    if ui.is_rect_visible(rect) {
        let how_on = ui.ctx().animate_bool_with_time(response.id, *on, 0.15);

        let bg_color = Color32::from_rgb(
            (COLOR_BG_HOVER.r() as f32 + (COLOR_PRIMARY.r() as f32 - COLOR_BG_HOVER.r() as f32) * how_on) as u8,
            (COLOR_BG_HOVER.g() as f32 + (COLOR_PRIMARY.g() as f32 - COLOR_BG_HOVER.g() as f32) * how_on) as u8,
            (COLOR_BG_HOVER.b() as f32 + (COLOR_PRIMARY.b() as f32 - COLOR_BG_HOVER.b() as f32) * how_on) as u8,
        );

        let circle_x = egui::lerp((rect.left() + 10.0)..=(rect.right() - 10.0), how_on);
        let circle_center = Pos2::new(circle_x, rect.center().y);

        ui.painter().rect_filled(rect, egui::CornerRadius::same(10), bg_color);
        ui.painter().circle_filled(circle_center, 7.0, Color32::WHITE);
    }

    response
}

/// Draw a stat card for dashboard
pub fn stat_card(ui: &mut Ui, label: &str, value: &str, icon: &str, color: Color32) {
    egui::Frame::new()
        .fill(COLOR_BG_CARD)
        .corner_radius(egui::CornerRadius::same(12))
        .stroke(Stroke::new(1.0, COLOR_BORDER))
        .inner_margin(egui::Margin::same(16))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                 ui.label(RichText::new(icon).size(20.0).color(color));
                 ui.vertical(|ui| {
                      ui.label(RichText::new(label).size(11.0).color(COLOR_TEXT_MUTED));
                      ui.label(RichText::new(value).size(20.0).strong().color(COLOR_TEXT));
                 });
            });
        });
}

/// Draw a compact service card for dashboard
pub fn service_card_compact(ui: &mut Ui, name: &str, icon: &str, version: &str, port: u16, running: bool) {
    egui::Frame::new()
        .fill(COLOR_BG_CARD)
        .corner_radius(egui::CornerRadius::same(10))
        .stroke(Stroke::new(1.0, COLOR_BORDER))
        .inner_margin(egui::Margin::symmetric(14, 10))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new(icon.replace("\u{FE0F}", "")).size(18.0));
                ui.add_space(8.0);
                ui.vertical(|ui| {
                    ui.label(RichText::new(name).size(14.0).strong().color(COLOR_TEXT));
                    ui.horizontal(|ui| {
                        ui.label(RichText::new(format!("v{} ● Port: {}", version, port)).size(10.0).color(COLOR_TEXT_DIM));
                        if running {
                             ui.add_space(8.0);
                             ui.label(RichText::new("●").size(10.0).color(COLOR_SUCCESS));
                        }
                    });
                });

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if running {
                        ui.label(RichText::new("UP").size(9.0).strong().color(COLOR_SUCCESS));
                    } else {
                        ui.label(RichText::new("DOWN").size(9.0).strong().color(COLOR_TEXT_MUTED));
                    }
                });
            });
        });
}
