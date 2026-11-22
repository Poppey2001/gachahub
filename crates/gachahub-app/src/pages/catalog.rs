//! "Add game" modal: a card grid of every supported title.

use crate::{
    app::GachaHubApp,
    i18n::tr,
    theme::{self, alpha, mix},
};
use eframe::egui::{
    self, Align2, Color32, CornerRadius, Pos2, Rect, RichText, Sense, Stroke, StrokeKind, Vec2,
};

impl GachaHubApp {
    pub fn catalog_modal(&mut self, ctx: &egui::Context) {
        let games = self.games.clone();
        let modal = egui::Modal::new(egui::Id::unique("gachahub-catalog"))
            .backdrop_color(Color32::from_black_alpha(170))
            .frame(
                egui::Frame::new()
                    .fill(theme::BG_RAISED)
                    .stroke(Stroke::new(1.0, theme::BORDER))
                    .corner_radius(22)
                    .inner_margin(28)
                    .shadow(egui::Shadow { offset: [0, 24], blur: 60, spread: 0, color: Color32::from_black_alpha(180) }),
            )
            .show(ctx, |ui| {
                ui.set_width(780.0);
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        ui.label(RichText::new(tr("BIBLIOTHEK", "LIBRARY")).font(theme::font_heavy(10.5)).color(theme::BRAND).extra_letter_spacing(2.0));
                        ui.label(RichText::new(tr("Spiele hinzufügen", "Add games")).font(theme::font_heavy(28.0)).color(theme::TEXT));
                        ui.label(
                            RichText::new(tr("Ein Launcher für alle – wähle, was in deiner Seitenleiste erscheint.", "One launcher for all – pick what shows up in your sidebar."))
                                .color(theme::MUTED),
                        );
                    });
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
                        if ui.button("✖").clicked() {
                            self.catalog_open = false;
                        }
                    });
                });
                ui.add_space(18.0);

                let cols = 3;
                let gap = 14.0;
                let card_w = (ui.available_width() - gap * (cols as f32 - 1.0)) / cols as f32;
                let card_h = 158.0;
                let mut toggle: Option<String> = None;
                let mut open: Option<String> = None;
                for row in games.chunks(cols) {
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = gap;
                        for game in row {
                            let in_lib = self.cfg.library.contains(&game.id);
                            let (a, b) = Self::accents(game);
                            let (rect, resp) = ui.allocate_exact_size(Vec2::new(card_w, card_h), Sense::click());
                            let hover = ui.ctx().animate_bool(resp.id, resp.hovered());
                            let p = ui.painter_at(rect.expand(8.0));
                            let r = rect.translate(Vec2::new(0.0, -2.0 * hover));
                            p.rect_filled(r, CornerRadius::same(16), mix(theme::PANEL, a, 0.06 + hover * 0.04));
                            let clip = p.with_clip_rect(r.shrink(1.0));
                            theme::radial_glow(&clip, r.right_top() + Vec2::new(-20.0, 20.0), Vec2::new(card_w * 0.8, card_h), alpha(a, 0.32 + hover * 0.1));
                            theme::radial_glow(&clip, r.left_bottom(), Vec2::new(card_w * 0.6, card_h * 0.7), alpha(b, 0.16));
                            clip.text(r.right_bottom() + Vec2::new(-10.0, 8.0), Align2::RIGHT_BOTTOM, &game.short_name, theme::font_heavy(64.0), Color32::from_white_alpha(12));
                            // mask the square glow corners back to the rounded card shape
                            p.rect_stroke(r, CornerRadius::same(16), Stroke::new(10.0, theme::BG_RAISED), StrokeKind::Outside);
                            p.rect_stroke(
                                r,
                                CornerRadius::same(16),
                                Stroke::new(1.0, if in_lib { alpha(a, 0.55) } else { mix(theme::BORDER, a, hover * 0.5) }),
                                StrokeKind::Inside,
                            );
                            let pad = 16.0;
                            let icon_rect = Rect::from_min_size(r.left_top() + Vec2::new(pad, pad), Vec2::splat(46.0));
                            self.paint_game_icon(ui, icon_rect, game, in_lib, hover);
                            p.text(icon_rect.right_center() + Vec2::new(12.0, 0.0), Align2::LEFT_CENTER, game.publisher.to_uppercase(), theme::font_heavy(9.0), mix(a, Color32::WHITE, 0.25));
                            let title = p.layout(game.name.clone(), theme::font_heavy(17.0), theme::TEXT, card_w - pad * 2.0);
                            let title_h = title.size().y;
                            let title_y = (r.bottom() - pad - 30.0 - title_h).max(icon_rect.bottom() + 8.0);
                            p.galley(Pos2::new(r.left() + pad, title_y), title, theme::TEXT);
                            p.text(
                                Pos2::new(r.left() + pad, title_y + title_h + 4.0),
                                Align2::LEFT_TOP,
                                &game.subtitle,
                                theme::font(11.5),
                                theme::MUTED,
                            );
                            // add/remove button
                            let btn = Rect::from_min_size(Pos2::new(r.right() - pad - 28.0, r.top() + pad - 4.0), Vec2::splat(28.0));
                            let btn_resp = ui.interact(btn, resp.id.with("btn"), Sense::click());
                            let bh = ui.ctx().animate_bool(btn_resp.id, btn_resp.hovered());
                            p.circle(
                                btn.center(),
                                14.0,
                                if in_lib { alpha(a, 0.9) } else { Color32::from_rgba_unmultiplied(255, 255, 255, (14.0 + bh * 20.0) as u8) },
                                Stroke::new(1.0, Color32::from_white_alpha(30)),
                            );
                            p.text(
                                btn.center(),
                                Align2::CENTER_CENTER,
                                if in_lib { "✔" } else { "+" },
                                theme::font_heavy(14.0),
                                if in_lib { theme::on_color(a) } else { theme::TEXT },
                            );
                            if btn_resp
                                .on_hover_text(if in_lib { tr("Aus Bibliothek entfernen", "Remove from library") } else { tr("Zur Bibliothek hinzufügen", "Add to library") })
                                .clicked()
                            {
                                toggle = Some(game.id.clone());
                            } else if resp.on_hover_cursor(egui::CursorIcon::PointingHand).clicked() {
                                open = Some(game.id.clone());
                            }
                        }
                    });
                    ui.add_space(gap);
                }
                if let Some(id) = toggle {
                    if self.cfg.library.contains(&id) {
                        if self.cfg.library.len() > 1 {
                            self.cfg.library.retain(|g| *g != id);
                            if self.cfg.selected_game == id {
                                self.cfg.selected_game = self.cfg.library[0].clone();
                            }
                        }
                    } else {
                        self.cfg.library.push(id);
                    }
                    self.mark_dirty(ui.ctx());
                }
                if let Some(id) = open {
                    self.select_game(&id);
                    self.catalog_open = false;
                }
            });
        if modal.should_close() {
            self.catalog_open = false;
        }
    }
}
