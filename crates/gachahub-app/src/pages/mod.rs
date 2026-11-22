mod catalog;
mod downloads;
pub mod game;
mod mods;
mod runners;
mod settings;

use crate::theme;
use eframe::egui::{self, Color32, Margin, Pos2, Rect, RichText};

/// Standard scrolling page with a soft top glow.
pub fn page_frame(ui: &mut egui::Ui, add: impl FnOnce(&mut egui::Ui)) {
    let rect = ui.max_rect();
    let p = ui.painter_at(rect);
    theme::gradient_rect(
        &p,
        rect,
        Color32::from_rgb(8, 11, 17),
        Color32::from_rgb(7, 9, 14),
        false,
    );
    theme::radial_glow(
        &p,
        Pos2::new(rect.right() - rect.width() * 0.15, rect.top()),
        egui::Vec2::new(rect.width() * 0.45, rect.height() * 0.4),
        theme::alpha(Color32::from_rgb(68, 102, 160), 0.14),
    );
    let _ = Rect::NOTHING;
    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            egui::Frame::new()
                .inner_margin(Margin {
                    left: 40,
                    right: 40,
                    top: 34,
                    bottom: 40,
                })
                .show(ui, |ui| {
                    ui.set_max_width(1180.0);
                    add(ui);
                });
        });
}

pub fn empty_state(ui: &mut egui::Ui, icon: &str, title: &str, text: &str) {
    egui::Frame::new()
        .stroke(egui::Stroke::new(1.0, theme::BORDER))
        .fill(theme::alpha(theme::PANEL, 0.35))
        .corner_radius(16)
        .inner_margin(Margin::same(28))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.vertical_centered(|ui| {
                ui.label(
                    RichText::new(icon)
                        .font(theme::font(26.0))
                        .color(theme::FAINT),
                );
                ui.label(
                    RichText::new(title)
                        .font(theme::font_semi(14.0))
                        .color(theme::TEXT_DIM),
                );
                ui.label(
                    RichText::new(text)
                        .font(theme::font(12.0))
                        .color(theme::MUTED),
                );
            });
        });
}
