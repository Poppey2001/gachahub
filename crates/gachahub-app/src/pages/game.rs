//! The game stage: full-bleed animated backdrop, hero block with the big
//! Play/Install button, live download progress and a stats footer.

use crate::{
    app::{GachaHubApp, Section, SettingsTab},
    backend::PickTarget,
    i18n::tr,
    theme::{self, alpha, mix, CtaButton},
};
use eframe::egui::{
    self, Align2, Color32, CornerRadius, Pos2, Rect, RichText, Sense, Stroke, Vec2,
};
use gachahub_core::{compat, download_manager::JobKind};

impl GachaHubApp {
    pub fn page_game(&mut self, ui: &mut egui::Ui) {
        let game = self.selected_game();
        let (a, b) = Self::accents(&game);
        let rect = ui.max_rect();
        let time = ui.input(|i| i.time) as f32;
        let animated = self.cfg.global.animated_backgrounds;
        let dim = self.cfg.global.background_dim as f32 / 100.0;

        // ---------------------------------------------------------- backdrop
        {
            let p = ui.painter_at(rect);
            p.rect_filled(rect, 0, Color32::from_rgb(8, 10, 15));
            let art = self
                .artwork
                .get(&game.id)
                .and_then(|x| x.background.clone());
            if let Some(uri) = art {
                // cover-fit the artwork
                let img = egui::Image::new(uri.clone());
                let size = img
                    .load_and_calc_size(ui, Vec2::splat(f32::INFINITY))
                    .unwrap_or(rect.size());
                let scale = (rect.width() / size.x).max(rect.height() / size.y);
                let drift = if animated {
                    Vec2::new((time * 0.05).sin() * 12.0, (time * 0.04).cos() * 8.0)
                } else {
                    Vec2::ZERO
                };
                let draw = Rect::from_center_size(rect.center() + drift, size * scale * 1.04);
                img.paint_at(&ui.new_child(egui::UiBuilder::new().max_rect(rect)), draw);
                p.rect_filled(rect, 0, Color32::from_black_alpha((dim * 150.0) as u8));
            } else {
                // procedural: drifting accent glows
                let t = if animated { time } else { 0.0 };
                let w = rect.width();
                let h = rect.height();
                theme::gradient_rect(
                    &p,
                    rect,
                    Color32::from_rgb(9, 12, 19),
                    mix(Color32::from_rgb(10, 12, 18), b, 0.12),
                    true,
                );
                theme::radial_glow(
                    &p,
                    rect.min
                        + Vec2::new(
                            w * (0.78 + (t * 0.11).sin() * 0.04),
                            h * (0.42 + (t * 0.08).cos() * 0.05),
                        ),
                    Vec2::new(w * 0.42, h * 0.62),
                    alpha(a, 0.36),
                );
                theme::radial_glow(
                    &p,
                    rect.min
                        + Vec2::new(
                            w * (0.6 + (t * 0.07).cos() * 0.05),
                            h * (0.22 + (t * 0.09).sin() * 0.04),
                        ),
                    Vec2::new(w * 0.36, h * 0.45),
                    alpha(b, 0.26),
                );
                theme::radial_glow(
                    &p,
                    rect.min + Vec2::new(w * (0.92 + (t * 0.05).sin() * 0.03), h * 0.88),
                    Vec2::new(w * 0.25, h * 0.3),
                    alpha(mix(a, Color32::WHITE, 0.3), 0.08),
                );
                // grid
                let step = 56.0;
                let mut x = rect.left() + (rect.width() * 0.45 / step).floor() * step;
                while x < rect.right() {
                    let fade = ((x - rect.left()) / rect.width() - 0.45).clamp(0.0, 0.3) / 0.3;
                    p.line_segment(
                        [Pos2::new(x, rect.top()), Pos2::new(x, rect.bottom())],
                        Stroke::new(1.0, Color32::from_white_alpha((7.0 * fade) as u8)),
                    );
                    x += step;
                }
                let mut y = rect.top();
                while y < rect.bottom() {
                    let x0 = rect.left() + rect.width() * 0.45;
                    theme::gradient_rect(
                        &p,
                        Rect::from_min_max(Pos2::new(x0, y), Pos2::new(rect.right(), y + 1.0)),
                        Color32::TRANSPARENT,
                        Color32::from_white_alpha(7),
                        true,
                    );
                    y += step;
                }
                // floating particles
                if animated {
                    for i in 0..18 {
                        let fi = i as f32;
                        let px = rect.left() + w * (0.45 + ((fi * 0.618).fract() * 0.55));
                        let speed = 6.0 + (fi * 1.3) % 9.0;
                        let py = rect.bottom() - ((t * speed + fi * 97.0) % (h + 40.0)) + 20.0;
                        let r = 1.0 + (fi * 0.37).fract() * 1.8;
                        p.circle_filled(
                            Pos2::new(px + (t * 0.3 + fi).sin() * 10.0, py),
                            r,
                            alpha(mix(a, Color32::WHITE, 0.5), 0.25),
                        );
                    }
                }
            }
            // watermark
            let wm_size = (rect.width() * 0.26).clamp(150.0, 380.0);
            p.text(
                Pos2::new(
                    rect.right() - rect.width() * 0.05,
                    rect.top() + rect.height() * 0.12,
                ),
                Align2::RIGHT_TOP,
                &game.short_name,
                theme::font_heavy(wm_size),
                Color32::from_white_alpha(9),
            );
            // vignettes for legibility
            theme::gradient_stops(
                &p,
                rect,
                &[
                    (0.0, Color32::from_rgba_unmultiplied(5, 7, 11, 250)),
                    (
                        0.32,
                        Color32::from_rgba_unmultiplied(5, 7, 11, (150.0 + dim * 60.0) as u8),
                    ),
                    (0.66, Color32::from_rgba_unmultiplied(5, 7, 11, 30)),
                    (1.0, Color32::from_rgba_unmultiplied(5, 7, 11, 120)),
                ],
                true,
            );
            theme::gradient_rect(
                &p,
                Rect::from_min_max(
                    Pos2::new(rect.left(), rect.bottom() - rect.height() * 0.38),
                    rect.right_bottom(),
                ),
                Color32::TRANSPARENT,
                Color32::from_rgba_unmultiplied(5, 7, 11, 245),
                false,
            );
            theme::gradient_rect(
                &p,
                Rect::from_min_max(rect.min, Pos2::new(rect.right(), rect.top() + 90.0)),
                Color32::from_rgba_unmultiplied(5, 7, 11, 160),
                Color32::TRANSPARENT,
                false,
            );
        }
        if animated {
            ui.ctx()
                .request_repaint_after(std::time::Duration::from_millis(33));
        }

        // ---------------------------------------------------------- top bar
        let top = Rect::from_min_size(rect.min, Vec2::new(rect.width(), 68.0));
        ui.scope_builder(
            egui::UiBuilder::new().max_rect(top.shrink2(Vec2::new(30.0, 0.0))),
            |ui| {
                ui.horizontal_centered(|ui| {
                    ui.label(
                        RichText::new("GachaHub")
                            .font(theme::font_heavy(15.0))
                            .color(theme::TEXT),
                    );
                    ui.label(
                        RichText::new(format!("v{}", gachahub_core::VERSION))
                            .font(theme::font(11.0))
                            .color(theme::FAINT),
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if theme::ghost_button_sized(ui, "⚙", "", 36.0, false)
                            .on_hover_text(tr("Spiel-Einstellungen", "Game settings"))
                            .clicked()
                        {
                            self.settings_game = game.id.clone();
                            self.settings_tab = SettingsTab::Game;
                            self.section = Section::Settings;
                        }
                        ui.add_space(4.0);
                        let lang_resp = theme::ghost_button_sized(
                            ui,
                            "🌐",
                            self.cfg.global.language.code(),
                            36.0,
                            false,
                        );
                        let mut picked = None;
                        egui::Popup::menu(&lang_resp).show(|ui| {
                            for l in gachahub_core::settings::Language::ALL {
                                if ui
                                    .selectable_label(
                                        self.cfg.global.language == l,
                                        l.native_name(),
                                    )
                                    .clicked()
                                {
                                    picked = Some(l);
                                }
                            }
                        });
                        if let Some(l) = picked {
                            self.cfg.global.language = l;
                            self.mark_dirty(ui.ctx());
                        }
                        ui.add_space(6.0);
                        self.app_update_button(ui);
                        if self.running.contains_key(&game.id) {
                            theme::pill(ui, tr("● Läuft", "● Running"), theme::OK);
                        }
                    });
                });
            },
        );

        // ---------------------------------------------------------- hero
        let left = rect.left() + (rect.width() * 0.05).clamp(40.0, 80.0);
        let hero_w = (rect.width() * 0.56).clamp(460.0, 700.0);
        let footer_h = 74.0;
        let hero_rect = Rect::from_min_max(
            Pos2::new(left, top.bottom() + 10.0),
            Pos2::new(left + hero_w, rect.bottom() - footer_h - 16.0),
        );
        let resolved = self.cfg.resolved(&game.id);
        let compat = compat::preset_for(&game.id);
        let job = self.job_for(&game.id).cloned();
        let installed = !resolved.executable_path.is_empty();
        let downloadable = gachahub_core::download_manager::supports_direct_download(&game.id);
        let xxmi_ready = game
            .xxmi_importer
            .as_ref()
            .and_then(|imp| {
                self.xxmi
                    .as_ref()
                    .and_then(|s| s.importers.iter().find(|i| &i.id == imp))
            })
            .is_some_and(|i| i.installed);
        let xxmi_installed = self.xxmi.as_ref().is_some_and(|s| s.installed);

        ui.scope_builder(egui::UiBuilder::new().max_rect(hero_rect), |ui| {
            ui.with_layout(egui::Layout::bottom_up(egui::Align::Min), |ui| {
                ui.add_space(26.0);
                // ---------- actions / progress (bottom-most, built first in bottom_up)
                let show_progress = job.as_ref().is_some_and(|j| {
                    !matches!(
                        j.phase.as_str(),
                        "ready" | "error" | "cancelled" | "downloaded"
                    )
                });
                if let (true, Some(job)) = (show_progress, job.as_ref()) {
                    theme::glass().show(ui, |ui| {
                        ui.set_width(hero_w.min(620.0));
                        ui.horizontal(|ui| {
                            ui.label(
                                RichText::new(format!("{:.0}%", job.progress * 100.0))
                                    .font(theme::font_heavy(24.0))
                                    .color(theme::TEXT),
                            );
                            ui.label(
                                RichText::new(phase_label(&job.phase))
                                    .font(theme::font_semi(12.0))
                                    .color(a),
                            );
                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    if ui
                                        .small_button("✖")
                                        .on_hover_text(tr("Abbrechen", "Cancel"))
                                        .clicked()
                                    {
                                        let _ = gachahub_core::download_manager::cancel(
                                            &self.sink, &job.id,
                                        );
                                    }
                                    let paused = job.phase == "paused";
                                    if ui
                                        .small_button(if paused { "▶" } else { "⏸" })
                                        .on_hover_text(if paused {
                                            tr("Fortsetzen", "Resume")
                                        } else {
                                            tr("Pausieren", "Pause")
                                        })
                                        .clicked()
                                    {
                                        let _ = gachahub_core::download_manager::pause(
                                            &self.sink, &job.id, !paused,
                                        );
                                    }
                                    ui.label(
                                        RichText::new(format!(
                                            "{} / {} · {}/s",
                                            theme::fmt_bytes(job.downloaded_bytes),
                                            theme::fmt_bytes(job.total_bytes),
                                            theme::fmt_bytes(job.speed_bytes)
                                        ))
                                        .font(theme::font(11.5))
                                        .color(theme::MUTED),
                                    );
                                },
                            );
                        });
                        ui.add_space(4.0);
                        theme::progress_bar(ui, job.progress, a, b, 6.0);
                        if let Some(m) = &job.message {
                            ui.add_space(2.0);
                            ui.label(RichText::new(m).font(theme::font(11.0)).color(theme::MUTED));
                        }
                    });
                } else {
                    ui.horizontal(|ui| {
                        let install_busy = self.busy.contains(&format!("install:{}", game.id));
                        let launch_busy = self.busy.contains(&format!("launch:{}", game.id))
                            || self.busy.contains(&format!("postinstall:{}", game.id));
                        let running = self.running.contains_key(&game.id);
                        let update = if installed { self.update_available(&game.id) } else { None };
                        let resumable = !installed && downloadable && self.partial_installs.contains(&game.id);
                        let official = !downloadable && !game.official_launchers.is_empty();
                        let steam_only = !downloadable && !official && !game.steam_names.is_empty();
                        let installer = self.installer_progress.get(&game.id).copied();
                        let watching = self.official_watch.contains_key(&game.id);
                        let installer_label;
                        let update_label;
                        let (icon, label) = if launch_busy {
                            ("⏳", tr("Startet…", "Starting…"))
                        } else if install_busy {
                            ("⏳", tr("Vorbereiten…", "Preparing…"))
                        } else if running {
                            ("●", tr("Läuft", "Running"))
                        } else if let Some(v) = &update {
                            update_label = format!("UPDATE {v}");
                            ("⬆", update_label.as_str())
                        } else if installed {
                            ("▶", tr("SPIELEN", "PLAY"))
                        } else if resumable {
                            ("⬇", tr("FORTSETZEN", "RESUME"))
                        } else if downloadable {
                            ("⬇", tr("INSTALLIEREN", "INSTALL"))
                        } else if let Some((done, total)) = installer {
                            installer_label = if total > 0 {
                                format!("{} {:.0}%", tr("INSTALLER", "INSTALLER"), done as f64 * 100.0 / total as f64)
                            } else {
                                tr("INSTALLER…", "INSTALLER…").to_string()
                            };
                            ("⏳", installer_label.as_str())
                        } else if official {
                            ("⬇", tr("INSTALLIEREN", "INSTALL"))
                        } else if steam_only {
                            ("🎮", tr("ÜBER STEAM INSTALLIEREN", "INSTALL VIA STEAM"))
                        } else {
                            ("📂", tr("SPIEL IMPORTIEREN", "IMPORT GAME"))
                        };
                        let resp = ui.add(
                            CtaButton::new(icon, label, a, b)
                                .enabled(!install_busy && !launch_busy && !running && installer.is_none()),
                        );
                        let resp = if watching && !installed {
                            resp.on_hover_text(tr(
                                "GachaHub sucht im Hintergrund nach der fertigen Installation.",
                                "GachaHub is looking for the finished installation in the background.",
                            ))
                        } else {
                            resp
                        };
                        if resp.clicked() {
                            if update.is_some() {
                                self.start_job(&game.id, JobKind::Update);
                            } else if installed {
                                self.launch(&game.id);
                            } else if downloadable {
                                self.start_install(&game.id);
                            } else if official {
                                self.edition_picker = Some(game.id.clone());
                            } else if steam_only {
                                let now = self.now;
                                self.install_via_steam(&game, now);
                            } else {
                                self.backend.pick_folder(
                                    PickTarget::ImportExisting(game.id.clone()),
                                    &format!(
                                        "{}: {}",
                                        game.name,
                                        tr("Installationsordner wählen", "choose install folder")
                                    ),
                                );
                            }
                        }
                        ui.add_space(4.0);
                        let idle = !install_busy && !launch_busy && !running;
                        if running
                            && self.sessions.contains_key(&game.id)
                            && theme::ghost_button(ui, "⏹", tr("Beenden", "Stop")).clicked()
                        {
                            self.stop_game(&game.id);
                        }
                        if update.is_some() {
                            // Updates are optional: still allow playing the installed version.
                            if theme::ghost_button(ui, "▶", tr("Spielen", "Play")).clicked() && idle {
                                self.launch(&game.id);
                            }
                        } else if installed && downloadable && self.installed_versions.contains_key(&game.id) {
                            if theme::ghost_button(ui, "🔍", tr("Dateien prüfen", "Verify files"))
                                .on_hover_text(tr(
                                    "Prüft alle Spieldateien und lädt beschädigte oder fehlende neu.",
                                    "Checks all game files and re-downloads damaged or missing ones.",
                                ))
                                .clicked()
                                && idle
                            {
                                self.start_job(&game.id, JobKind::Verify);
                            }
                        } else if (installed || downloadable || steam_only || official)
                            && theme::ghost_button(
                                ui,
                                "📂",
                                tr("Vorhandene Installation", "Existing install"),
                            )
                            .clicked()
                        {
                            self.backend.pick_folder(
                                PickTarget::ImportExisting(game.id.clone()),
                                &format!(
                                    "{}: {}",
                                    game.name,
                                    tr("Installationsordner wählen", "choose install folder")
                                ),
                            );
                        }
                        if game.mod_policy != "disabled"
                            && theme::ghost_button(ui, "🔌", "Mods").clicked()
                        {
                            self.mods_game = game.id.clone();
                            self.section = Section::Mods;
                            let id = game.id.clone();
                            self.refresh_mods(&id);
                        }
                        if let Some(importer) = game.xxmi_importer.clone() {
                            let enabled = resolved.xxmi_enabled && xxmi_ready;
                            let label = if !xxmi_installed {
                                tr("XXMI installieren", "Install XXMI").to_string()
                            } else if !xxmi_ready {
                                format!("{importer} Setup")
                            } else {
                                format!(
                                    "XXMI {}",
                                    if enabled {
                                        tr("AN", "ON")
                                    } else {
                                        tr("AUS", "OFF")
                                    }
                                )
                            };
                            let busy = self.busy.contains("xxmi-install");
                            let resp = theme::ghost_button_sized(
                                ui,
                                "✨",
                                if busy { "XXMI…" } else { &label },
                                54.0,
                                enabled,
                            );
                            if resp.clicked() && !busy {
                                if !xxmi_installed {
                                    self.install_xxmi();
                                } else if !xxmi_ready {
                                    self.open_xxmi_gui(&game.id);
                                } else {
                                    let gs = self.cfg.game_mut(&game.id);
                                    gs.xxmi_enabled = !gs.xxmi_enabled;
                                    self.mark_dirty(ui.ctx());
                                }
                            }
                        }
                        self.game_more_menu(ui, &game, installed);
                    });
                }
                ui.add_space(16.0);

                // ---------- install path row
                theme::glass().show(ui, |ui| {
                    ui.set_width(hero_w.min(620.0));
                    ui.horizontal(|ui| {
                        ui.vertical(|ui| {
                            ui.label(
                                RichText::new(tr("INSTALLATIONSPFAD", "INSTALL PATH"))
                                    .font(theme::font_semi(9.5))
                                    .color(theme::FAINT)
                                    .extra_letter_spacing(1.4),
                            );
                            let path = if installed {
                                resolved.executable_path.clone()
                            } else {
                                resolved.install_path.clone()
                            };
                            ui.add(
                                egui::Label::new(
                                    RichText::new(path)
                                        .font(theme::font(12.5))
                                        .color(theme::TEXT_DIM),
                                )
                                .truncate(),
                            );
                        });
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui
                                .button(tr("📁  Ordner wählen", "📁  Choose folder"))
                                .clicked()
                            {
                                self.backend.pick_folder(
                                    PickTarget::GameInstallPath(game.id.clone()),
                                    &format!(
                                        "{}: {}",
                                        tr("Installationsordner", "Install folder"),
                                        game.name
                                    ),
                                );
                            }
                        });
                    });
                });
                ui.add_space(12.0);

                // ---------- notices
                if let Some(err) = self.game_errors.get(&game.id).cloned() {
                    notice(ui, "⚠", &err, theme::ERR, hero_w);
                    ui.add_space(4.0);
                    if !installed
                        && theme::ghost_button(ui, "🔎", tr("Spiel-EXE selbst auswählen", "Pick the game executable"))
                            .clicked()
                    {
                        self.backend.pick_file(
                            PickTarget::GameExecutable(game.id.clone()),
                            &format!("{}: {}", game.name, tr("Spiel-EXE wählen", "choose game executable")),
                            &[("Windows", &["exe"])],
                        );
                    }
                    ui.add_space(8.0);
                }
                if let Some(notes) = game.notes_for(true).map(|de| crate::i18n::text(de, game.notes.as_deref().unwrap_or(de))) {
                    notice(ui, "ℹ", notes, theme::TEXT_DIM, hero_w);
                    ui.add_space(8.0);
                }
                ui.add_space(8.0);

                // ---------- meta chips
                ui.horizontal(|ui| {
                    theme::pill(ui, &game.publisher, theme::TEXT_DIM);
                    if let Some(p) = game.platforms.first() {
                        theme::pill(ui, p, theme::TEXT_DIM);
                    }
                    let (mods_text, mods_col) = match game.mod_policy.as_str() {
                        "supported" => (tr("Mods unterstützt", "Mods supported"), theme::OK),
                        "restricted" => (tr("Mods eingeschränkt", "Mods restricted"), theme::WARN),
                        _ => (tr("Keine Mods", "No mods"), theme::MUTED),
                    };
                    theme::pill(ui, mods_text, mods_col);
                    if let Some(c) = compat {
                        if cfg!(target_os = "linux") {
                            let col = match c.linux.state.as_str() {
                                "native" | "playable" => theme::OK,
                                "playable-with-caveats" | "experimental" => theme::WARN,
                                "unsupported" => theme::ERR,
                                _ => theme::MUTED,
                            };
                            let short = match c.linux.state.as_str() {
                                "native" => tr("nativ", "native"),
                                "playable" => tr("spielbar", "playable"),
                                "playable-with-caveats" => {
                                    tr("spielbar mit Hinweisen", "playable with caveats")
                                }
                                "experimental" => tr("experimentell", "experimental"),
                                "unsupported" => tr("nicht unterstützt", "unsupported"),
                                _ => tr("unbekannt", "unknown"),
                            };
                            theme::pill(ui, &format!("Linux: {short}"), col).on_hover_text(
                                format!(
                                    "{}\n\n{}",
                                    crate::i18n::text(c.linux.label_for(true), c.linux.label_for(false)),
                                    crate::i18n::texts(c.linux.notes_for(true), c.linux.notes_for(false)).join("\n\n")
                                ),
                            );
                        }
                    }
                    if let Some(imp) = &game.xxmi_importer {
                        let on = resolved.xxmi_enabled;
                        theme::pill(
                            ui,
                            &format!(
                                "XXMI {imp} {}",
                                if on { tr("AN", "ON") } else { tr("AUS", "OFF") }
                            ),
                            if on { theme::BRAND } else { theme::MUTED },
                        );
                    }
                });
                ui.add_space(14.0);

                // ---------- titles
                ui.label(
                    RichText::new(crate::i18n::text(game.subtitle_for(true), &game.subtitle))
                        .font(theme::font(19.0))
                        .color(theme::TEXT_DIM),
                );
                ui.add_space(2.0);
                if let Some(logo) = self.artwork.get(&game.id).and_then(|x| x.logo.clone()) {
                    ui.add(egui::Image::new(logo).max_height(120.0).max_width(hero_w));
                } else {
                    let size = (rect.width() * 0.055).clamp(44.0, 78.0);
                    ui.add(
                        egui::Label::new(
                            RichText::new(&game.name)
                                .font(theme::font_heavy(size))
                                .color(theme::TEXT),
                        )
                        .wrap(),
                    );
                }
                ui.add_space(4.0);
                ui.label(
                    RichText::new(format!(
                        "{}  ·  {}",
                        game.provider.to_uppercase(),
                        self.installed_versions
                            .get(&game.id)
                            .map(|v| format!("v{v}"))
                            .or_else(|| game.version.clone())
                            .unwrap_or_else(|| "stable".into())
                            .to_uppercase()
                    ))
                    .font(theme::font_heavy(10.5))
                    .color(mix(a, Color32::WHITE, 0.2))
                    .extra_letter_spacing(2.2),
                );
            });
        });

        // ---------------------------------------------------------- footer stats
        let footer = Rect::from_min_max(
            Pos2::new(left, rect.bottom() - footer_h),
            Pos2::new(rect.right() - 34.0, rect.bottom() - 18.0),
        );
        let p = ui.painter();
        p.line_segment(
            [footer.left_top(), footer.right_top()],
            Stroke::new(1.0, Color32::from_white_alpha(14)),
        );
        let gs = self.cfg.games.get(&game.id);
        let mut playtime = gs.map(|g| g.playtime_seconds).unwrap_or(0);
        if let Some(start) = self.running.get(&game.id) {
            playtime += (ui.input(|i| i.time) - start) as u64;
        }
        let status = if self.running.contains_key(&game.id) {
            tr("Läuft", "Running").to_string()
        } else if let Some(j) = &job {
            phase_label(&j.phase).to_string()
        } else if installed {
            tr("Bereit", "Ready").to_string()
        } else {
            tr("Nicht installiert", "Not installed").to_string()
        };
        let stats = [
            (tr("STATUS", "STATUS"), status),
            (
                tr("INSTALLATIONSART", "INSTALL TYPE"),
                if self.is_steam_game(&game.id) {
                    "Steam".to_string()
                } else {
                    game.install_modes.first().cloned().unwrap_or_default()
                },
            ),
            (
                tr("LAUFZEIT", "RUNTIME"),
                if self.is_steam_game(&game.id) {
                    tr("Steam (eigenes Proton)", "Steam (its own Proton)").to_string()
                } else if cfg!(windows) {
                    tr("Nativ (Windows)", "Native (Windows)").to_string()
                } else {
                    format!(
                        "{} · {}",
                        crate::i18n::runtime_label(resolved.runtime),
                        if resolved.runner_version.is_empty() {
                            "default"
                        } else {
                            &resolved.runner_version
                        }
                    )
                },
            ),
            (tr("SPIELZEIT", "PLAYTIME"), theme::fmt_duration(playtime)),
        ];
        let col_w = (footer.width() / 4.0).min(230.0);
        for (i, (k, v)) in stats.iter().enumerate() {
            let x = footer.left() + i as f32 * col_w;
            if i > 0 {
                p.line_segment(
                    [
                        Pos2::new(x - 12.0, footer.top() + 16.0),
                        Pos2::new(x - 12.0, footer.bottom()),
                    ],
                    Stroke::new(1.0, Color32::from_white_alpha(12)),
                );
            }
            p.text(
                Pos2::new(x, footer.top() + 18.0),
                Align2::LEFT_TOP,
                *k,
                theme::font_semi(9.5),
                theme::FAINT,
            );
            let galley = p.layout(
                v.clone(),
                theme::font_semi(12.5),
                theme::TEXT_DIM,
                col_w - 24.0,
            );
            p.galley(Pos2::new(x, footer.top() + 34.0), galley, theme::TEXT_DIM);
        }
        let _ = CornerRadius::ZERO;
        let _ = Sense::hover();
    }
}

fn notice(ui: &mut egui::Ui, icon: &str, text: &str, color: Color32, width: f32) {
    egui::Frame::new()
        .fill(Color32::from_rgba_unmultiplied(9, 12, 18, 170))
        .stroke(Stroke::new(1.0, alpha(color, 0.22)))
        .corner_radius(CornerRadius::same(10))
        .inner_margin(egui::Margin::symmetric(12, 9))
        .show(ui, |ui| {
            ui.set_max_width(width.min(620.0));
            ui.horizontal(|ui| {
                ui.label(RichText::new(icon).color(color));
                ui.add(
                    egui::Label::new(RichText::new(text).font(theme::font(12.0)).color(mix(
                        color,
                        theme::TEXT,
                        0.4,
                    )))
                    .wrap(),
                );
            });
        });
}

pub fn phase_label(phase: &str) -> &'static str {
    match phase {
        "resolving" => tr("Auflösen", "Resolving"),
        "queued" => tr("In Warteschlange", "Queued"),
        "downloading" => tr("Lädt herunter", "Downloading"),
        "paused" => tr("Pausiert", "Paused"),
        "verifying" => tr("Prüfe", "Verifying"),
        "extracting" => tr("Entpacke", "Extracting"),
        "downloaded" => tr("Heruntergeladen", "Downloaded"),
        "ready" => tr("Bereit", "Ready"),
        "cancelled" => tr("Abgebrochen", "Cancelled"),
        "error" => tr("Fehler", "Error"),
        _ => "—",
    }
}

impl GachaHubApp {
    /// Compact self-update control for the top bar (only shown when relevant).
    pub fn app_update_button(&mut self, ui: &mut egui::Ui) {
        if self.app_update_ready {
            if theme::ghost_button_sized(
                ui,
                "↻",
                tr("Neu starten & aktualisieren", "Restart to update"),
                36.0,
                true,
            )
            .clicked()
            {
                self.restart_app();
            }
        } else if let Some((done, total)) = self.app_update_progress {
            let pct = if total > 0 {
                done as f64 * 100.0 / total as f64
            } else {
                0.0
            };
            theme::pill(
                ui,
                &format!("{} {pct:.0}%", tr("GachaHub-Update", "GachaHub update")),
                theme::BRAND,
            );
        } else if let Some(info) = &self.app_update {
            let label = format!("GachaHub {}", info.version);
            if theme::ghost_button_sized(ui, "⬆", &label, 36.0, true)
                .on_hover_text(tr(
                    "Neue GachaHub-Version herunterladen und installieren",
                    "Download and install the new GachaHub version",
                ))
                .clicked()
            {
                self.install_app_update();
            }
        }
    }

    /// "⋯" menu with less frequent game actions.
    fn game_more_menu(&mut self, ui: &mut egui::Ui, game: &gachahub_core::Game, installed: bool) {
        let resp = theme::ghost_button(ui, "", "•••")
            .on_hover_text(tr("Weitere Aktionen", "More actions"));
        egui::Popup::menu(&resp).show(|ui| {
            ui.set_min_width(220.0);
            if ui
                .button(tr("📂  Spielordner öffnen", "📂  Open game folder"))
                .clicked()
            {
                self.open_game_folder(&game.id);
            }
            if !game.official_launchers.is_empty()
                && ui
                    .button(tr(
                        "🔀  Version / Download-Quelle wählen…",
                        "🔀  Choose edition / download source…",
                    ))
                    .clicked()
            {
                self.edition_picker = Some(game.id.clone());
            }
            if !game.steam_names.is_empty() {
                if !installed
                    && gachahub_core::download_manager::supports_direct_download(&game.id)
                    && ui
                        .button(tr(
                            "🎮  Stattdessen über Steam installieren",
                            "🎮  Install via Steam instead",
                        ))
                        .clicked()
                {
                    let now = self.now;
                    self.install_via_steam(game, now);
                }
                if ui
                    .button(tr(
                        "⟳  Steam-Bibliothek durchsuchen",
                        "⟳  Scan Steam library",
                    ))
                    .clicked()
                {
                    self.refresh_steam();
                }
            }
            if ui
                .button(tr("⚙  Spiel-Einstellungen", "⚙  Game settings"))
                .clicked()
            {
                self.settings_game = game.id.clone();
                self.settings_tab = SettingsTab::Game;
                self.section = Section::Settings;
            }
            if installed {
                ui.separator();
                if ui
                    .button(tr("🗑  Deinstallieren…", "🗑  Uninstall…"))
                    .clicked()
                {
                    self.uninstall_confirm = Some(game.id.clone());
                }
            }
        });

        if self.uninstall_confirm.as_deref() == Some(game.id.as_str()) {
            let can_delete = self.can_delete_files(&game.id);
            let dir = self.cfg.resolved(&game.id).install_path;
            let modal = egui::Modal::new(egui::Id::unique(("uninstall", game.id.as_str()))).show(ui.ctx(), |ui| {
                ui.set_width(440.0);
                ui.label(RichText::new(format!("{} {}", game.name, tr("deinstallieren?", "– uninstall?"))).font(theme::font_heavy(20.0)));
                ui.add_space(6.0);
                let mut choice = None;
                if can_delete {
                    ui.label(format!("{}\n{dir}", tr("Alle Spieldateien werden gelöscht:", "All game files will be deleted:")));
                } else {
                    ui.label(tr(
                        "Diese Installation wurde nicht von GachaHub angelegt. Die Dateien bleiben erhalten – GachaHub vergisst nur den Pfad.",
                        "This installation was not created by GachaHub. Files are kept – GachaHub only forgets the path.",
                    ));
                }
                ui.add_space(10.0);
                ui.horizontal(|ui| {
                    if ui.button(tr("Abbrechen", "Cancel")).clicked() {
                        choice = Some(None);
                    }
                    let label = if can_delete { tr("🗑  Dateien löschen", "🗑  Delete files") } else { tr("Aus GachaHub entfernen", "Remove from GachaHub") };
                    if ui.add(egui::Button::new(RichText::new(label).color(theme::ERR))).clicked() {
                        choice = Some(Some(can_delete));
                    }
                });
                choice
            });
            match modal.inner {
                Some(Some(delete)) => {
                    self.uninstall_confirm = None;
                    self.uninstall_game(&game.id, delete);
                }
                Some(None) => self.uninstall_confirm = None,
                None if modal.should_close() => self.uninstall_confirm = None,
                None => {}
            }
        }
    }
}
