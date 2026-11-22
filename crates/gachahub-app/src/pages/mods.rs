use crate::{
    app::{GachaHubApp, Section, SettingsTab},
    backend::{Msg, PickTarget},
    i18n::tr,
    theme::{self, alpha, mix},
};
use eframe::egui::{self, CornerRadius, RichText, Sense, Stroke, StrokeKind, Vec2};

impl GachaHubApp {
    pub fn page_mods(&mut self, ui: &mut egui::Ui) {
        super::page_frame(ui, |ui| {
            theme::page_header(
                ui,
                tr("Mod-Manager", "Mod manager"),
                "Mods",
                tr(
                    "Bibliothek getrennt vom Spiel, Aktivierung per Kopie oder Symlink, Archiv-Import (ZIP/7z/RAR).",
                    "Library kept apart from the game, activation via copy or symlink, archive import (ZIP/7z/RAR).",
                ),
            );

            ui.horizontal_top(|ui| {
                // ------------------------------------------------ game list
                egui::Frame::new()
                    .fill(alpha(theme::PANEL, 0.7))
                    .stroke(Stroke::new(1.0, theme::BORDER_SOFT))
                    .corner_radius(16)
                    .inner_margin(8)
                    .show(ui, |ui| {
                        ui.set_width(230.0);
                        ui.set_min_height(420.0);
                        ui.vertical(|ui| {
                        for id in self.cfg.library.clone() {
                            let Some(game) = self.game(&id).cloned() else { continue };
                            let (a, b) = Self::accents(&game);
                            let active = self.mods_game == id;
                            let (rect, resp) = ui.allocate_exact_size(Vec2::new(214.0, 52.0), Sense::click());
                            let hover = ui.ctx().animate_bool(resp.id, resp.hovered());
                            if active || hover > 0.0 {
                                ui.painter().rect(
                                    rect,
                                    CornerRadius::same(12),
                                    if active { alpha(a, 0.1) } else { alpha(theme::PANEL_HOVER, hover) },
                                    Stroke::new(1.0, if active { alpha(a, 0.35) } else { egui::Color32::TRANSPARENT }),
                                    StrokeKind::Inside,
                                );
                            }
                            let badge = egui::Rect::from_min_size(rect.min + Vec2::new(8.0, 8.0), Vec2::splat(36.0));
                            theme::paint_game_badge(ui, badge, &game.short_name, a, b, active, hover);
                            ui.painter().text(
                                badge.right_center() + Vec2::new(10.0, -7.0),
                                egui::Align2::LEFT_CENTER,
                                &game.name,
                                theme::font_semi(12.5),
                                if active { theme::TEXT } else { theme::TEXT_DIM },
                            );
                            let policy = match game.mod_policy.as_str() {
                                "supported" => tr("unterstützt", "supported"),
                                "restricted" => tr("eingeschränkt", "restricted"),
                                _ => tr("deaktiviert", "disabled"),
                            };
                            ui.painter().text(badge.right_center() + Vec2::new(10.0, 9.0), egui::Align2::LEFT_CENTER, policy, theme::font(10.5), theme::MUTED);
                            if resp.on_hover_cursor(egui::CursorIcon::PointingHand).clicked() {
                                self.mods_game = id.clone();
                                if !self.mods.contains_key(&id) {
                                    self.refresh_mods(&id);
                                }
                            }
                        }
                        });
                    });

                ui.add_space(10.0);

                // ------------------------------------------------ workspace
                ui.vertical(|ui| {
                    let gid = self.mods_game.clone();
                    let Some(game) = self.game(&gid).cloned() else { return };
                    let (a, _) = Self::accents(&game);
                    let r = self.cfg.resolved(&gid);

                    if game.mod_policy == "restricted" {
                        egui::Frame::new()
                            .fill(alpha(theme::WARN, 0.07))
                            .stroke(Stroke::new(1.0, alpha(theme::WARN, 0.3)))
                            .corner_radius(12)
                            .inner_margin(12)
                            .show(ui, |ui| {
                                ui.set_width(ui.available_width());
                                ui.label(
                                    RichText::new(tr(
                                        "⚠  Für dieses Spiel sind Mods eingeschränkt (Anti-Cheat / Nutzungsbedingungen). Nutzung auf eigenes Risiko.",
                                        "⚠  Mods are restricted for this game (anti-cheat / terms of service). Use at your own risk.",
                                    ))
                                    .color(theme::WARN),
                                );
                            });
                        ui.add_space(8.0);
                    }

                    theme::card().show(ui, |ui| {
                        ui.set_width(ui.available_width());
                        egui::Grid::new("mod-paths").num_columns(3).spacing([12.0, 8.0]).show(ui, |ui| {
                            ui.label(RichText::new(tr("Bibliothek", "Library")).color(theme::MUTED));
                            ui.allocate_ui(Vec2::new(460.0, 18.0), |ui| ui.add(egui::Label::new(RichText::new(&r.mod_library_path).monospace()).truncate()));
                            if ui.small_button("📁").clicked() {
                                self.backend.pick_folder(PickTarget::GameModLibrary(gid.clone()), tr("Mod-Bibliothek wählen", "Choose mod library"));
                            }
                            ui.end_row();
                            ui.label(RichText::new(tr("Aktiver Mods-Ordner", "Active mods folder")).color(theme::MUTED));
                            if r.active_mods_path.is_empty() {
                                ui.label(RichText::new(tr("nicht gesetzt – z. B. XXMI/GIMI/Mods", "not set – e.g. XXMI/GIMI/Mods")).color(theme::WARN));
                            } else {
                                ui.allocate_ui(Vec2::new(460.0, 18.0), |ui| ui.add(egui::Label::new(RichText::new(&r.active_mods_path).monospace()).truncate()));
                            }
                            if ui.small_button("📁").clicked() {
                                self.backend.pick_folder(PickTarget::GameActiveMods(gid.clone()), tr("Aktiven Mods-Ordner wählen", "Choose active mods folder"));
                            }
                            ui.end_row();
                        });
                        ui.add_space(6.0);
                        ui.horizontal(|ui| {
                            if ui.button(tr("📂  Ordner importieren", "📂  Import folder")).clicked() {
                                self.backend.pick_folder(PickTarget::ImportModFolder(gid.clone()), tr("Mod-Ordner importieren", "Import mod folder"));
                            }
                            if ui.button(tr("📦  Archiv importieren", "📦  Import archive")).clicked() {
                                self.backend.pick_file(
                                    PickTarget::ImportModArchive(gid.clone()),
                                    tr("Mod-Archiv importieren", "Import mod archive"),
                                    &[("Archive", &["zip", "7z", "rar"])],
                                );
                            }
                            if ui.button(tr("⟳  Neu scannen", "⟳  Rescan")).clicked() {
                                self.refresh_mods(&gid);
                            }
                            if ui.button(tr("⚙  Pfade", "⚙  Paths")).clicked() {
                                self.settings_game = gid.clone();
                                self.settings_tab = SettingsTab::Game;
                                self.section = Section::Settings;
                            }
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                ui.add(egui::TextEdit::singleline(&mut self.mod_filter).hint_text(tr("🔍 Suchen…", "🔍 Search…")).desired_width(180.0));
                            });
                        });
                    });
                    ui.add_space(10.0);

                    let busy = self.busy.contains(&format!("mods:{gid}"));
                    match self.mods.get(&gid).cloned() {
                        None if busy => {
                            ui.spinner();
                        }
                        None => super::empty_state(ui, "🔌", tr("Noch nicht gescannt", "Not scanned yet"), tr("Klicke auf „Neu scannen“.", "Click “Rescan”.")),
                        Some(Err(e)) => super::empty_state(ui, "⚠", tr("Mods konnten nicht gelesen werden", "Could not read mods"), &e),
                        Some(Ok(snap)) => {
                            ui.horizontal(|ui| {
                                theme::pill(ui, &format!("{} {}", snap.stats.total, tr("gesamt", "total")), theme::TEXT_DIM);
                                theme::pill(ui, &format!("{} {}", snap.stats.enabled, tr("aktiv", "enabled")), theme::OK);
                                if snap.stats.conflicts > 0 {
                                    theme::pill(ui, &format!("{} {}", snap.stats.conflicts, tr("Konflikte", "conflicts")), theme::WARN);
                                }
                                if snap.stats.broken > 0 {
                                    theme::pill(ui, &format!("{} {}", snap.stats.broken, tr("defekt", "broken")), theme::ERR);
                                }
                                if busy {
                                    ui.spinner();
                                }
                            });
                            ui.add_space(6.0);
                            if snap.entries.is_empty() {
                                super::empty_state(ui, "🔌", tr("Keine Mods gefunden", "No mods found"), tr("Importiere einen Ordner oder ein Archiv.", "Import a folder or an archive."));
                            }
                            let filter = self.mod_filter.to_lowercase();
                            for entry in snap.entries.iter().filter(|e| filter.is_empty() || e.name.to_lowercase().contains(&filter)) {
                                egui::Frame::new()
                                    .fill(if entry.enabled { mix(theme::PANEL, a, 0.05) } else { alpha(theme::PANEL, 0.8) })
                                    .stroke(Stroke::new(1.0, if entry.enabled { alpha(a, 0.3) } else { theme::BORDER_SOFT }))
                                    .corner_radius(12)
                                    .inner_margin(egui::Margin::symmetric(14, 10))
                                    .show(ui, |ui| {
                                        ui.set_width(ui.available_width());
                                        ui.horizontal(|ui| {
                                            let mut on = entry.enabled;
                                            if theme::toggle(ui, &mut on).changed() && !busy {
                                                let (lib, act, mode) = (r.mod_library_path.clone(), r.active_mods_path.clone(), r.mod_deployment_mode.as_str().to_string());
                                                let (g2, mid) = (gid.clone(), entry.id.clone());
                                                let managed = entry.managed;
                                                self.busy.insert(format!("mods:{gid}"));
                                                self.backend.blocking(move || {
                                                    use gachahub_core::mod_manager as mm;
                                                    let res = if on {
                                                        mm::enable(&g2, &mid, &lib, &act, &mode)
                                                    } else if managed {
                                                        mm::disable(&g2, &mid, &lib, &act)
                                                    } else {
                                                        mm::adopt(&g2, &mid, &lib, &act).and_then(|_| mm::disable(&g2, &mid, &lib, &act))
                                                    };
                                                    Msg::Mods(g2, res)
                                                });
                                            }
                                            ui.add_space(6.0);
                                            ui.vertical(|ui| {
                                                ui.label(RichText::new(&entry.name).font(theme::font_semi(13.5)).color(theme::TEXT));
                                                ui.label(
                                                    RichText::new(format!("{} · {} INI · {}", entry.relative_path, entry.ini_count, theme::fmt_bytes(entry.size_bytes)))
                                                        .font(theme::font(11.0))
                                                        .color(theme::MUTED),
                                                );
                                            });
                                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                                let (txt, col) = match entry.status.as_str() {
                                                    "enabled" => (tr("Aktiv", "Enabled"), theme::OK),
                                                    "conflict" => (tr("Konflikt", "Conflict"), theme::WARN),
                                                    "broken" => (tr("Defekt", "Broken"), theme::ERR),
                                                    _ => (tr("Inaktiv", "Disabled"), theme::MUTED),
                                                };
                                                let pill = theme::pill(ui, txt, col);
                                                if let Some(reason) = &entry.conflict_reason {
                                                    pill.on_hover_text(reason);
                                                }
                                                if !entry.managed && entry.enabled {
                                                    theme::pill(ui, tr("extern", "external"), theme::BRAND)
                                                        .on_hover_text(tr("Liegt im aktiven Ordner, wird aber nicht von GachaHub verwaltet.", "Lives in the active folder but is not managed by GachaHub."));
                                                }
                                            });
                                        });
                                    });
                                ui.add_space(5.0);
                            }
                        }
                    }
                });
            });
        });
    }
}
