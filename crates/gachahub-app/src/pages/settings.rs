use crate::{
    app::{GachaHubApp, SettingsTab},
    backend::{Msg, PickTarget},
    i18n::tr,
    theme::{self, alpha},
};
use eframe::egui::{self, CornerRadius, RichText, Sense, Stroke, StrokeKind, Vec2};
use gachahub_core::settings::{AppChannel, Language, ModDeploymentMode, RuntimeKind};

/// A labelled settings row: title + hint on the left, control on the right.
fn row(
    ui: &mut egui::Ui,
    title: &str,
    hint: &str,
    control: impl FnOnce(&mut egui::Ui) -> bool,
) -> bool {
    let mut changed = false;
    ui.horizontal(|ui| {
        ui.set_min_height(40.0);
        ui.vertical(|ui| {
            ui.set_width((ui.available_width() * 0.45).max(220.0));
            ui.label(
                RichText::new(title)
                    .font(theme::font_semi(13.0))
                    .color(theme::TEXT),
            );
            if !hint.is_empty() {
                ui.add(
                    egui::Label::new(
                        RichText::new(hint)
                            .font(theme::font(11.5))
                            .color(theme::MUTED),
                    )
                    .wrap(),
                );
            }
        });
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            changed = control(ui);
        });
    });
    ui.add_space(4.0);
    let r = ui.available_rect_before_wrap();
    ui.painter().line_segment(
        [r.left_top(), egui::pos2(r.right(), r.top())],
        Stroke::new(1.0, theme::BORDER_SOFT),
    );
    ui.add_space(6.0);
    changed
}

fn path_edit(ui: &mut egui::Ui, value: &mut String, hint: &str) -> (bool, bool) {
    let browse = ui.button("📁").clicked();
    let changed = ui
        .add(
            egui::TextEdit::singleline(value)
                .hint_text(hint)
                .desired_width(360.0),
        )
        .changed();
    (changed, browse)
}

/// Text field for an `Option<String>` override; empty means "inherit".
fn opt_edit(ui: &mut egui::Ui, value: &mut Option<String>, inherited: &str, width: f32) -> bool {
    let mut text = value.clone().unwrap_or_default();
    let changed = ui
        .add(
            egui::TextEdit::singleline(&mut text)
                .hint_text(inherited)
                .desired_width(width),
        )
        .changed();
    if changed {
        *value = if text.trim().is_empty() {
            None
        } else {
            Some(text)
        };
    }
    changed
}

/// Three-state toggle for `Option<bool>` (inherit / on / off).
fn opt_bool(ui: &mut egui::Ui, value: &mut Option<bool>, inherited: bool) -> bool {
    let mut changed = false;
    let mut on = value.unwrap_or(inherited);
    if value.is_some()
        && ui
            .small_button("↺")
            .on_hover_text(tr(
                "Auf globalen Wert zurücksetzen",
                "Reset to global value",
            ))
            .clicked()
    {
        *value = None;
        return true;
    }
    if theme::toggle(ui, &mut on).changed() {
        *value = Some(on);
        changed = true;
    }
    if value.is_none() {
        ui.label(
            RichText::new(tr("global", "global"))
                .font(theme::font(10.5))
                .color(theme::FAINT),
        );
    }
    changed
}

fn runtime_combo(ui: &mut egui::Ui, id: &str, value: &mut RuntimeKind) -> bool {
    let mut changed = false;
    egui::ComboBox::from_id_salt(id)
        .selected_text(value.label())
        .width(180.0)
        .show_ui(ui, |ui| {
            for r in RuntimeKind::ALL {
                changed |= ui.selectable_value(value, r, r.label()).changed();
            }
        });
    changed
}

impl GachaHubApp {
    pub fn page_settings(&mut self, ui: &mut egui::Ui) {
        super::page_frame(ui, |ui| {
            theme::page_header(
                ui,
                tr("Konfiguration", "Configuration"),
                tr("Einstellungen", "Settings"),
                &format!(
                    "{} {}",
                    tr("Gespeichert in", "Stored in"),
                    gachahub_core::settings::Config::path().display()
                ),
            );

            // tab bar
            ui.horizontal(|ui| {
                for (tab, label) in [
                    (SettingsTab::General, tr("Allgemein", "General")),
                    (SettingsTab::Game, tr("Spiel-spezifisch", "Per game")),
                    (SettingsTab::Xxmi, "XXMI"),
                ] {
                    let active = self.settings_tab == tab;
                    let galley = ui.painter().layout_no_wrap(
                        label.to_string(),
                        theme::font_semi(13.0),
                        theme::TEXT,
                    );
                    let (rect, resp) = ui.allocate_exact_size(
                        Vec2::new(galley.size().x + 32.0, 36.0),
                        Sense::click(),
                    );
                    let hover = ui.ctx().animate_bool(resp.id, resp.hovered());
                    ui.painter().rect(
                        rect,
                        CornerRadius::same(11),
                        if active {
                            alpha(theme::BRAND, 0.14)
                        } else {
                            alpha(theme::PANEL, 0.6 + hover * 0.4)
                        },
                        Stroke::new(
                            1.0,
                            if active {
                                alpha(theme::BRAND, 0.45)
                            } else {
                                theme::BORDER_SOFT
                            },
                        ),
                        StrokeKind::Inside,
                    );
                    ui.painter().text(
                        rect.center(),
                        egui::Align2::CENTER_CENTER,
                        label,
                        theme::font_semi(13.0),
                        if active {
                            theme::BRAND
                        } else {
                            theme::TEXT_DIM
                        },
                    );
                    if resp
                        .on_hover_cursor(egui::CursorIcon::PointingHand)
                        .clicked()
                    {
                        self.settings_tab = tab;
                    }
                }
            });
            ui.add_space(14.0);

            let mut changed = false;
            match self.settings_tab {
                SettingsTab::General => changed |= self.settings_general(ui),
                SettingsTab::Game => changed |= self.settings_game_tab(ui),
                SettingsTab::Xxmi => changed |= self.settings_xxmi(ui),
            }
            if changed {
                self.mark_dirty(ui.ctx());
            }
        });
    }

    fn settings_general(&mut self, ui: &mut egui::Ui) -> bool {
        let mut changed = false;
        theme::section_label(ui, "GachaHub");
        theme::card().show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.label(
                        RichText::new(format!("GachaHub {}", gachahub_core::VERSION))
                            .font(theme::font_semi(14.0))
                            .color(theme::TEXT),
                    );
                    let status = if self.app_update_ready {
                        tr("Update installiert – Neustart nötig.", "Update installed – restart required.").to_string()
                    } else if let Some(info) = &self.app_update {
                        format!(
                            "{} {}{}",
                            tr("Neue Version verfügbar:", "New version available:"),
                            info.version,
                            if info.prerelease { tr(" (Vorabversion)", " (pre-release)") } else { "" }
                        )
                    } else if self.busy.contains("app-update-check") {
                        tr("Suche nach Updates…", "Checking for updates…").to_string()
                    } else {
                        self.app_update_status.clone().unwrap_or_default()
                    };
                    ui.label(RichText::new(status).font(theme::font(11.5)).color(theme::MUTED));
                });
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if self.app_update_ready {
                        if ui.button(tr("↻  Neu starten", "↻  Restart")).clicked() {
                            self.restart_app();
                        }
                    } else if let Some((done, total)) = self.app_update_progress {
                        ui.spinner();
                        ui.label(format!("{} / {}", theme::fmt_bytes(done), theme::fmt_bytes(total)));
                    } else if self.app_update.is_some() {
                        if ui.button(tr("⬆  Jetzt aktualisieren", "⬆  Update now")).clicked() {
                            self.install_app_update();
                        }
                    }
                    if !self.busy.contains("app-update-check")
                        && self.app_update_progress.is_none()
                        && ui.button(tr("⟳  Nach Updates suchen", "⟳  Check for updates")).clicked()
                    {
                        self.check_app_update(true);
                    }
                });
            });
            ui.add_space(6.0);
            let g = &mut self.cfg.global;
            changed |= row(
                ui,
                tr("Beim Start nach Updates suchen", "Check for updates on start"),
                "",
                |ui| theme::toggle(ui, &mut g.app_update_check).changed(),
            );
            changed |= row(
                ui,
                tr("Update-Kanal", "Update channel"),
                tr(
                    "Vorabversionen werden bei jeder Änderung automatisch gebaut – neuer, aber weniger getestet.",
                    "Pre-releases are built automatically for every change – newer, but less tested.",
                ),
                |ui| {
                    let mut c = false;
                    c |= ui
                        .selectable_value(&mut g.app_update_channel, AppChannel::Prerelease, tr("Vorabversionen", "Pre-releases"))
                        .changed();
                    c |= ui.selectable_value(&mut g.app_update_channel, AppChannel::Stable, tr("Stabil", "Stable")).changed();
                    c
                },
            );
        });

        theme::section_label(ui, tr("Oberfläche", "Interface"));
        theme::card().show(ui, |ui| {
            ui.set_width(ui.available_width());
            let g = &mut self.cfg.global;
            changed |= row(ui, tr("Sprache", "Language"), "", |ui| {
                let mut c = false;
                c |= ui
                    .selectable_value(&mut g.language, Language::En, "English")
                    .changed();
                c |= ui
                    .selectable_value(&mut g.language, Language::De, "Deutsch")
                    .changed();
                c
            });
            changed |= row(
                ui,
                tr("Animierte Hintergründe", "Animated backgrounds"),
                tr(
                    "Sanft schwebende Farbverläufe und Partikel.",
                    "Softly drifting gradients and particles.",
                ),
                |ui| theme::toggle(ui, &mut g.animated_backgrounds).changed(),
            );
            changed |= row(
                ui,
                tr("Hintergrund abdunkeln", "Background dim"),
                tr("Wirkt auf eigene Artworks.", "Applies to custom artwork."),
                |ui| {
                    ui.add(egui::Slider::new(&mut g.background_dim, 0..=100).suffix(" %"))
                        .changed()
                },
            );
        });

        theme::section_label(ui, tr("Pfade", "Paths"));
        theme::card().show(ui, |ui| {
            ui.set_width(ui.available_width());
            let mut pick = None;
            {
                let g = &mut self.cfg.global;
                changed |= row(
                    ui,
                    tr("Standard-Installationsordner", "Default install folder"),
                    tr(
                        "Jedes Spiel bekommt einen Unterordner.",
                        "Each game gets a sub folder.",
                    ),
                    |ui| {
                        let (c, b) =
                            path_edit(ui, &mut g.default_install_path, "~/Games/GachaHub/Games");
                        if b {
                            pick = Some(PickTarget::GlobalInstallPath);
                        }
                        c
                    },
                );
                changed |= row(
                    ui,
                    tr("Wine-/Proton-Prefixe", "Wine/Proton prefixes"),
                    "",
                    |ui| {
                        let (c, b) = path_edit(
                            ui,
                            &mut g.prefix_base_path,
                            "~/.local/share/gachahub/prefixes",
                        );
                        if b {
                            pick = Some(PickTarget::GlobalPrefixBase);
                        }
                        c
                    },
                );
                changed |= row(ui, tr("Mod-Stammordner", "Mod root folder"), "", |ui| {
                    let (c, b) =
                        path_edit(ui, &mut g.mod_root_path, "~/.local/share/gachahub/mods");
                    if b {
                        pick = Some(PickTarget::GlobalModRoot);
                    }
                    c
                });
            }
            if let Some(t) = pick {
                self.backend
                    .pick_folder(t, tr("Ordner wählen", "Choose folder"));
            }
        });

        if !cfg!(target_os = "windows") {
            theme::section_label(
                ui,
                tr("Linux-Laufzeit (Standard)", "Linux runtime (defaults)"),
            );
            theme::card().show(ui, |ui| {
                ui.set_width(ui.available_width());
                let runners = self.runners.clone();
                let mut pick = false;
                let g = &mut self.cfg.global;
                changed |= row(
                    ui,
                    tr("Laufzeit", "Runtime"),
                    tr(
                        "„Automatisch“ nutzt umu-run mit verwaltetem Proton.",
                        "“Automatic” uses umu-run with managed Proton.",
                    ),
                    |ui| runtime_combo(ui, "global-runtime", &mut g.default_runtime),
                );
                changed |= row(
                    ui,
                    tr("Runner", "Runner"),
                    tr(
                        "Konkreter Proton-/Wine-Build (leer = automatisch).",
                        "Specific Proton/Wine build (empty = automatic).",
                    ),
                    |ui| {
                        let mut c = false;
                        if ui.small_button("📁").clicked() {
                            pick = true;
                        }
                        let current = runners
                            .iter()
                            .find(|r| r.path == g.default_runner_path)
                            .map(|r| r.name.clone())
                            .unwrap_or_else(|| {
                                if g.default_runner_path.is_empty() {
                                    tr("Automatisch", "Automatic").to_string()
                                } else {
                                    g.default_runner_path.clone()
                                }
                            });
                        egui::ComboBox::from_id_salt("global-runner")
                            .selected_text(current)
                            .width(300.0)
                            .show_ui(ui, |ui| {
                                if ui
                                    .selectable_label(
                                        g.default_runner_path.is_empty(),
                                        tr("Automatisch", "Automatic"),
                                    )
                                    .clicked()
                                {
                                    g.default_runner_path.clear();
                                    c = true;
                                }
                                for r in &runners {
                                    if ui
                                        .selectable_label(g.default_runner_path == r.path, &r.name)
                                        .clicked()
                                    {
                                        g.default_runner_path = r.path.clone();
                                        g.default_runner_version = r.version.clone();
                                        c = true;
                                    }
                                }
                            });
                        c
                    },
                );
                changed |= row(
                    ui,
                    "GameMode",
                    tr(
                        "Feral GameMode für CPU-/GPU-Optimierung.",
                        "Feral GameMode CPU/GPU tuning.",
                    ),
                    |ui| theme::toggle(ui, &mut g.default_game_mode_enabled).changed(),
                );
                changed |= row(
                    ui,
                    "Gamescope",
                    tr("Micro-Compositor von Valve.", "Valve's micro compositor."),
                    |ui| theme::toggle(ui, &mut g.default_gamescope_enabled).changed(),
                );
                changed |= row(
                    ui,
                    "MangoHud",
                    tr("FPS-/Performance-Overlay.", "FPS/performance overlay."),
                    |ui| theme::toggle(ui, &mut g.default_mango_hud_enabled).changed(),
                );
                changed |= row(
                    ui,
                    tr("Standby verhindern", "Prevent sleep"),
                    tr(
                        "Via systemd-inhibit, solange das Spiel läuft.",
                        "Via systemd-inhibit while the game runs.",
                    ),
                    |ui| theme::toggle(ui, &mut g.default_prevent_sleep).changed(),
                );
                if pick {
                    self.backend.pick_folder(
                        PickTarget::GlobalRunnerPath,
                        tr("Runner-Ordner wählen", "Choose runner folder"),
                    );
                }
            });
        }

        theme::section_label(ui, tr("Downloads & Updates", "Downloads & updates"));
        theme::card().show(ui, |ui| {
            ui.set_width(ui.available_width());
            let g = &mut self.cfg.global;
            changed |= row(
                ui,
                tr("Automatische Updates", "Automatic updates"),
                tr(
                    "Beim Start nach neuen Spielversionen suchen und sie direkt installieren.",
                    "Check for new game versions on start and install them right away.",
                ),
                |ui| theme::toggle(ui, &mut g.auto_update).changed(),
            );
            changed |= row(
                ui,
                tr("Gleichzeitige Downloads", "Parallel downloads"),
                tr(
                    "Anzahl Dateien, die parallel geladen werden.",
                    "Number of files fetched at the same time.",
                ),
                |ui| {
                    ui.add(egui::Slider::new(&mut g.parallel_downloads, 1..=32))
                        .changed()
                },
            );
            changed |= row(
                ui,
                tr("Bandbreite begrenzen", "Limit bandwidth"),
                tr(
                    "Maximale Download-Geschwindigkeit.",
                    "Maximum download speed.",
                ),
                |ui| {
                    let mut c = false;
                    let mut limited = g.bandwidth_limit_mbps.is_some();
                    if let Some(mbps) = g.bandwidth_limit_mbps.as_mut() {
                        c |= ui
                            .add(
                                egui::Slider::new(mbps, 1..=1000)
                                    .logarithmic(true)
                                    .suffix(" Mbit/s"),
                            )
                            .changed();
                    }
                    if theme::toggle(ui, &mut limited).changed() {
                        g.bandwidth_limit_mbps = limited.then_some(50);
                        c = true;
                    }
                    c
                },
            );
            changed |= row(
                ui,
                tr("Sprachausgabe", "Voice language"),
                tr(
                    "Sprachpaket für Genshin, Star Rail und ZZZ.",
                    "Voice pack for Genshin, Star Rail and ZZZ.",
                ),
                |ui| {
                    let mut c = false;
                    let langs = [
                        ("en-us", "English"),
                        ("ja-jp", "日本語"),
                        ("zh-cn", "中文"),
                        ("ko-kr", "한국어"),
                    ];
                    let current = langs
                        .iter()
                        .find(|(code, _)| *code == g.voice_language)
                        .map(|(_, n)| *n)
                        .unwrap_or("English");
                    egui::ComboBox::from_id_salt("voice-language")
                        .selected_text(current)
                        .width(160.0)
                        .show_ui(ui, |ui| {
                            for (code, name) in langs {
                                if ui
                                    .selectable_label(g.voice_language == code, name)
                                    .clicked()
                                {
                                    g.voice_language = code.to_string();
                                    c = true;
                                }
                            }
                        });
                    c
                },
            );
        });

        theme::section_label(ui, "Mods");
        theme::card().show(ui, |ui| {
            ui.set_width(ui.available_width());
            let g = &mut self.cfg.global;
            changed |= row(
                ui,
                tr("Bereitstellung", "Deployment"),
                tr(
                    "Kopie ist am sichersten, Symlinks sparen Platz.",
                    "Copy is safest, symlinks save space.",
                ),
                |ui| {
                    let mut c = false;
                    c |= ui
                        .selectable_value(
                            &mut g.default_mod_deployment_mode,
                            ModDeploymentMode::Symlink,
                            "Symlink",
                        )
                        .changed();
                    c |= ui
                        .selectable_value(
                            &mut g.default_mod_deployment_mode,
                            ModDeploymentMode::Copy,
                            tr("Kopie", "Copy"),
                        )
                        .changed();
                    c
                },
            );
        });

        ui.add_space(10.0);
        if ui
            .button(tr(
                "↺  Alle globalen Einstellungen zurücksetzen",
                "↺  Reset all global settings",
            ))
            .clicked()
        {
            self.cfg.global = Default::default();
            changed = true;
        }
        changed
    }

    fn settings_game_tab(&mut self, ui: &mut egui::Ui) -> bool {
        let mut changed = false;
        // game picker chips
        ui.horizontal_wrapped(|ui| {
            for id in self.cfg.library.clone() {
                let Some(game) = self.game(&id).cloned() else {
                    continue;
                };
                let active = self.settings_game == id;
                let resp = ui
                    .scope(|ui| {
                        ui.horizontal(|ui| {
                            let (icon_rect, r) =
                                ui.allocate_exact_size(Vec2::splat(30.0), Sense::click());
                            let hover = ui.ctx().animate_bool(r.id, r.hovered());
                            self.paint_game_icon(ui, icon_rect, &game, active, hover);
                            let l = ui.add(
                                egui::Label::new(RichText::new(&game.name).color(if active {
                                    theme::TEXT
                                } else {
                                    theme::MUTED
                                }))
                                .sense(Sense::click()),
                            );
                            r.union(l)
                        })
                        .inner
                    })
                    .inner;
                if resp
                    .on_hover_cursor(egui::CursorIcon::PointingHand)
                    .clicked()
                {
                    self.settings_game = id.clone();
                }
                ui.add_space(10.0);
            }
        });
        ui.add_space(10.0);

        let gid = self.settings_game.clone();
        let Some(game) = self.game(&gid).cloned() else {
            return false;
        };
        let global = self.cfg.global.clone();
        let runners = self.runners.clone();
        let mut pick: Option<(PickTarget, bool)> = None;
        let mut detect = false;

        let gs = self.cfg.game_mut(&gid);
        theme::card().show(ui, |ui| {
            ui.set_width(ui.available_width());
            changed |= row(
                ui,
                tr("Spiel-spezifische Werte verwenden", "Use per-game values"),
                tr(
                    "Aus = alle Werte kommen aus den globalen Einstellungen.",
                    "Off = everything comes from the global settings.",
                ),
                |ui| theme::toggle(ui, &mut gs.override_enabled).changed(),
            );
        });

        theme::section_label(ui, tr("Installation", "Installation"));
        theme::card().show(ui, |ui| {
            ui.set_width(ui.available_width());
            changed |= row(ui, tr("Installationsordner", "Install folder"), "", |ui| {
                if ui.small_button("📁").clicked() {
                    pick = Some((PickTarget::GameInstallPath(gid.clone()), true));
                }
                opt_edit(
                    ui,
                    &mut gs.install_path,
                    &format!("{}/{gid}", global.default_install_path),
                    360.0,
                )
            });
            changed |= row(
                ui,
                tr("Spiel-EXE", "Game executable"),
                tr(
                    "Wird beim Import automatisch erkannt.",
                    "Detected automatically on import.",
                ),
                |ui| {
                    if ui.small_button("📄").clicked() {
                        pick = Some((PickTarget::GameExecutable(gid.clone()), false));
                    }
                    if gid == "genshin"
                        && ui
                            .small_button("🔍")
                            .on_hover_text(tr("Automatisch erkennen", "Detect automatically"))
                            .clicked()
                    {
                        detect = true;
                    }
                    opt_edit(
                        ui,
                        &mut gs.executable_path,
                        tr("nicht gesetzt", "not set"),
                        360.0,
                    )
                },
            );
        });

        if !cfg!(target_os = "windows") {
            theme::section_label(ui, tr("Laufzeit", "Runtime"));
            theme::card().show(ui, |ui| {
                ui.set_width(ui.available_width());
                changed |= row(ui, tr("Laufzeit", "Runtime"), "", |ui| {
                    let mut c = false;
                    if gs.runtime.is_some() && ui.small_button("↺").clicked() {
                        gs.runtime = None;
                        c = true;
                    }
                    let mut v = gs.runtime.unwrap_or(global.default_runtime);
                    if runtime_combo(ui, "game-runtime", &mut v) {
                        gs.runtime = Some(v);
                        c = true;
                    }
                    c
                });
                changed |= row(ui, tr("Runner", "Runner"), "", |ui| {
                    let mut c = false;
                    if ui.small_button("📁").clicked() {
                        pick = Some((PickTarget::GameRunnerPath(gid.clone()), true));
                    }
                    let path = gs
                        .runner_path
                        .clone()
                        .unwrap_or_else(|| global.default_runner_path.clone());
                    let current = runners
                        .iter()
                        .find(|r| r.path == path)
                        .map(|r| r.name.clone())
                        .unwrap_or_else(|| {
                            if path.is_empty() {
                                tr("Automatisch", "Automatic").to_string()
                            } else {
                                path.clone()
                            }
                        });
                    egui::ComboBox::from_id_salt("game-runner")
                        .selected_text(current)
                        .width(300.0)
                        .show_ui(ui, |ui| {
                            if ui
                                .selectable_label(
                                    gs.runner_path.is_none(),
                                    tr("Global übernehmen", "Use global"),
                                )
                                .clicked()
                            {
                                gs.runner_path = None;
                                gs.runner_version = None;
                                c = true;
                            }
                            if ui
                                .selectable_label(
                                    gs.runner_path.as_deref() == Some(""),
                                    tr("Automatisch", "Automatic"),
                                )
                                .clicked()
                            {
                                gs.runner_path = Some(String::new());
                                c = true;
                            }
                            for r in &runners {
                                if ui.selectable_label(path == r.path, &r.name).clicked() {
                                    gs.runner_path = Some(r.path.clone());
                                    gs.runner_version = Some(r.version.clone());
                                    c = true;
                                }
                            }
                        });
                    c
                });
                changed |= row(ui, "Prefix", "", |ui| {
                    if ui.small_button("📁").clicked() {
                        pick = Some((PickTarget::GamePrefix(gid.clone()), true));
                    }
                    opt_edit(
                        ui,
                        &mut gs.prefix_path,
                        &format!("{}/{gid}", global.prefix_base_path),
                        360.0,
                    )
                });
                changed |= row(ui, "GameMode", "", |ui| {
                    opt_bool(
                        ui,
                        &mut gs.game_mode_enabled,
                        global.default_game_mode_enabled,
                    )
                });
                changed |= row(ui, "Gamescope", "", |ui| {
                    opt_bool(
                        ui,
                        &mut gs.gamescope_enabled,
                        global.default_gamescope_enabled,
                    )
                });
                changed |= row(ui, "MangoHud", "", |ui| {
                    opt_bool(
                        ui,
                        &mut gs.mango_hud_enabled,
                        global.default_mango_hud_enabled,
                    )
                });
                changed |= row(ui, tr("Standby verhindern", "Prevent sleep"), "", |ui| {
                    opt_bool(ui, &mut gs.prevent_sleep, global.default_prevent_sleep)
                });
            });
        }

        theme::section_label(ui, tr("Start", "Launch"));
        theme::card().show(ui, |ui| {
            ui.set_width(ui.available_width());
            changed |= row(
                ui,
                tr("Startargumente", "Launch arguments"),
                tr(
                    "Shell-artig, Anführungszeichen erlaubt.",
                    "Shell-like, quotes allowed.",
                ),
                |ui| opt_edit(ui, &mut gs.launch_arguments, "-popupwindow …", 360.0),
            );
            ui.label(
                RichText::new(tr(
                    "Umgebungsvariablen (KEY=VALUE, eine pro Zeile)",
                    "Environment variables (KEY=VALUE, one per line)",
                ))
                .font(theme::font_semi(13.0))
                .color(theme::TEXT),
            );
            let mut env = gs.environment_variables.clone().unwrap_or_default();
            if ui
                .add(
                    egui::TextEdit::multiline(&mut env)
                        .code_editor()
                        .desired_rows(4)
                        .desired_width(f32::INFINITY)
                        .hint_text("DXVK_ASYNC=1"),
                )
                .changed()
            {
                gs.environment_variables = if env.trim().is_empty() {
                    None
                } else {
                    Some(env)
                };
                changed = true;
            }
        });

        if let Some(c) = gachahub_core::compat::preset_for(&gid) {
            if !cfg!(target_os = "windows") {
                theme::section_label(ui, tr("Kompatibilität (Linux)", "Compatibility (Linux)"));
                theme::card().show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    ui.label(
                        RichText::new(&c.linux.label)
                            .font(theme::font_semi(13.5))
                            .color(theme::TEXT),
                    );
                    ui.label(
                        RichText::new(format!(
                            "{}: {}",
                            tr("Empfohlen", "Recommended"),
                            c.linux.recommended_runner
                        ))
                        .color(theme::TEXT_DIM),
                    );
                    for n in &c.linux.notes {
                        ui.label(
                            RichText::new(format!("•  {n}"))
                                .font(theme::font(12.0))
                                .color(theme::MUTED),
                        );
                    }
                    if !c.linux.environment_variables.is_empty()
                        || !c.linux.launch_arguments.is_empty()
                    {
                        ui.add_space(6.0);
                        if ui
                            .button(tr(
                                "✨  Empfohlene Werte übernehmen",
                                "✨  Apply recommended values",
                            ))
                            .clicked()
                        {
                            if !c.linux.environment_variables.is_empty() {
                                gs.environment_variables =
                                    Some(c.linux.environment_variables.clone());
                            }
                            if !c.linux.launch_arguments.is_empty() {
                                gs.launch_arguments = Some(c.linux.launch_arguments.clone());
                            }
                            if let Some(r) = RuntimeKind::parse(&c.linux.recommended_runtime) {
                                gs.runtime = Some(r);
                            }
                            changed = true;
                        }
                    }
                    ui.label(
                        RichText::new(format!("{} {}", tr("Stand:", "Checked:"), c.checked_at))
                            .font(theme::font(10.5))
                            .color(theme::FAINT),
                    );
                });
            }
        }

        if gachahub_core::download_manager::supports_direct_download(&gid) {
            theme::section_label(ui, "Updates");
            theme::card().show(ui, |ui| {
                ui.set_width(ui.available_width());
                changed |= row(
                    ui,
                    tr("Automatische Updates", "Automatic updates"),
                    "",
                    |ui| opt_bool(ui, &mut gs.auto_update, global.auto_update),
                );
            });
        }

        theme::section_label(ui, "Mods");
        theme::card().show(ui, |ui| {
            ui.set_width(ui.available_width());
            changed |= row(ui, tr("Mod-Bibliothek", "Mod library"), "", |ui| {
                if ui.small_button("📁").clicked() {
                    pick = Some((PickTarget::GameModLibrary(gid.clone()), true));
                }
                opt_edit(
                    ui,
                    &mut gs.mod_library_path,
                    &format!("{}/{gid}/library", global.mod_root_path),
                    360.0,
                )
            });
            changed |= row(
                ui,
                tr("Aktiver Mods-Ordner", "Active mods folder"),
                tr("z. B. …/XXMI/GIMI/Mods", "e.g. …/XXMI/GIMI/Mods"),
                |ui| {
                    if ui.small_button("📁").clicked() {
                        pick = Some((PickTarget::GameActiveMods(gid.clone()), true));
                    }
                    opt_edit(
                        ui,
                        &mut gs.active_mods_path,
                        tr("nicht gesetzt", "not set"),
                        360.0,
                    )
                },
            );
            if game.xxmi_importer.is_some() {
                changed |= row(
                    ui,
                    tr("Über XXMI starten", "Launch through XXMI"),
                    tr(
                        "Startet das Spiel mit dem Model-Importer.",
                        "Launches the game with the model importer.",
                    ),
                    |ui| theme::toggle(ui, &mut gs.xxmi_enabled).changed(),
                );
            }
        });

        ui.add_space(10.0);
        if ui
            .button(tr(
                "↺  Spiel-Einstellungen zurücksetzen",
                "↺  Reset game settings",
            ))
            .clicked()
        {
            let playtime = gs.playtime_seconds;
            *gs = Default::default();
            gs.playtime_seconds = playtime;
            changed = true;
        }

        if let Some((target, folder)) = pick {
            if folder {
                self.backend
                    .pick_folder(target, tr("Ordner wählen", "Choose folder"));
            } else {
                self.backend.pick_file(
                    target,
                    tr("Spiel-EXE wählen", "Choose game executable"),
                    &[("Windows executable", &["exe"])],
                );
            }
        }
        if detect {
            let r = self.cfg.resolved(&gid);
            self.busy.insert(format!("postinstall:{gid}"));
            let g2 = gid.clone();
            self.backend.blocking(move || {
                Msg::PostInstall(
                    g2.clone(),
                    gachahub_core::postinstall::defaults(&g2, &r.install_path, &r.prefix_path),
                )
            });
        }
        changed
    }

    fn settings_xxmi(&mut self, ui: &mut egui::Ui) -> bool {
        let mut changed = false;
        theme::card().show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.label(RichText::new("XXMI Launcher").font(theme::font_heavy(18.0)).color(theme::TEXT));
                    ui.label(
                        RichText::new(tr(
                            "Model-Importer für GIMI, SRMI, ZZMI, WWMI, EFMI und HIMI – von GachaHub installiert und aktualisiert.",
                            "Model importers for GIMI, SRMI, ZZMI, WWMI, EFMI and HIMI – installed and updated by GachaHub.",
                        ))
                        .color(theme::MUTED),
                    );
                });
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("⟳").on_hover_text(tr("Status aktualisieren", "Refresh status")).clicked() {
                        self.refresh_xxmi();
                    }
                });
            });
            ui.add_space(8.0);
            match self.xxmi.clone() {
                None => {
                    ui.spinner();
                }
                Some(s) => {
                    ui.horizontal_wrapped(|ui| {
                        if s.installed {
                            theme::pill(ui, &format!("{} {}", tr("Installiert", "Installed"), s.version.clone().unwrap_or_default()), theme::OK);
                        } else {
                            theme::pill(ui, tr("Nicht installiert", "Not installed"), theme::MUTED);
                        }
                        if let Some(l) = &s.latest_version {
                            theme::pill(ui, &format!("{} {l}", tr("Neueste:", "Latest:")), theme::TEXT_DIM);
                        }
                        if s.update_available {
                            theme::pill(ui, tr("Update verfügbar", "Update available"), theme::BRAND);
                        }
                        for imp in &s.importers {
                            theme::pill(ui, &imp.id, if imp.installed { theme::OK } else { theme::FAINT });
                        }
                    });
                    for w in &s.warnings {
                        ui.label(RichText::new(format!("⚠ {w}")).color(theme::WARN).font(theme::font(12.0)));
                    }
                    ui.add_space(8.0);
                    let busy = self.busy.contains("xxmi-install");
                    if let Some(p) = self.xxmi_progress.clone().filter(|_| busy) {
                        ui.label(RichText::new(&p.message).color(theme::TEXT_DIM));
                        theme::progress_bar(ui, p.progress as f32, theme::BRAND, theme::BRAND_2, 6.0);
                    }
                    ui.horizontal(|ui| {
                        let label = if !s.installed {
                            tr("⬇  XXMI installieren", "⬇  Install XXMI")
                        } else if s.update_available {
                            tr("⬆  Aktualisieren", "⬆  Update")
                        } else {
                            tr("🛠  Reparieren", "🛠  Repair")
                        };
                        if ui.add_enabled(!busy && !s.using_custom_path, egui::Button::new(label)).clicked() {
                            self.install_xxmi();
                        }
                        if s.installed && s.managed && ui.add_enabled(!busy, egui::Button::new(tr("🗑  Entfernen", "🗑  Remove"))).clicked() {
                            self.remove_xxmi();
                        }
                        // Open XXMI for the selected game (or the first library game with an importer),
                        // so it runs in that game's runner and prefix.
                        let target = std::iter::once(self.cfg.selected_game.clone())
                            .chain(self.cfg.library.clone())
                            .find(|id| self.game(id).is_some_and(|g| g.xxmi_importer.is_some()));
                        if let Some(target) = target {
                            let name = self.game(&target).map(|g| g.short_name.clone()).unwrap_or_default();
                            if s.installed && ui.button(format!("{} ({name})", tr("Öffnen (GUI)", "Open (GUI)"))).clicked() {
                                self.open_xxmi_gui(&target);
                            }
                        }
                    });
                }
            }
        });

        theme::section_label(ui, tr("Konfiguration", "Configuration"));
        theme::card().show(ui, |ui| {
            ui.set_width(ui.available_width());
            let mut pick = None;
            {
                let g = &mut self.cfg.global;
                changed |= row(
                    ui,
                    tr("Installationsordner", "Install folder"),
                    tr(
                        "Leer = Standard im Datenordner.",
                        "Empty = default in the data folder.",
                    ),
                    |ui| {
                        let (c, b) =
                            path_edit(ui, &mut g.xxmi_install_path, tr("Standard", "default"));
                        if b {
                            pick = Some((PickTarget::XxmiInstallPath, true));
                        }
                        c
                    },
                );
                changed |= row(
                    ui,
                    tr("Eigener Launcher", "Custom launcher"),
                    tr(
                        "Optional: bestehende XXMI Launcher.exe verwenden.",
                        "Optional: use an existing XXMI Launcher.exe.",
                    ),
                    |ui| {
                        let (c, b) = path_edit(ui, &mut g.xxmi_launcher_path, "XXMI Launcher.exe");
                        if b {
                            pick = Some((PickTarget::XxmiLauncherPath, false));
                        }
                        c
                    },
                );
                changed |= row(
                    ui,
                    tr("Ohne GUI starten", "Launch without GUI"),
                    "--nogui",
                    |ui| theme::toggle(ui, &mut g.xxmi_no_gui).changed(),
                );
                changed |= row(
                    ui,
                    tr("Automatisch aktualisieren", "Auto update"),
                    "",
                    |ui| theme::toggle(ui, &mut g.xxmi_auto_update).changed(),
                );
            }
            if let Some((t, folder)) = pick {
                if folder {
                    self.backend.pick_folder(t, "XXMI");
                } else {
                    self.backend
                        .pick_file(t, "XXMI Launcher", &[("Executable", &["exe"])]);
                }
            }
        });
        changed
    }
}
