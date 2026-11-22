use crate::{
    app::GachaHubApp,
    backend::Msg,
    i18n::tr,
    theme::{self, alpha},
};
use eframe::egui::{self, CornerRadius, RichText, Sense, Stroke, StrokeKind, Vec2};

impl GachaHubApp {
    pub fn page_runners(&mut self, ui: &mut egui::Ui) {
        super::page_frame(ui, |ui| {
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    theme::page_header(
                        ui,
                        tr("Kompatibilität", "Compatibility"),
                        tr("Runner", "Runners"),
                        tr(
                            "Wine- und Proton-Builds erkennen, installieren und verwalten.",
                            "Detect, install and manage Wine and Proton builds.",
                        ),
                    );
                });
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
                    if ui.button(tr("⟳  Neu erkennen", "⟳  Re-detect")).clicked() {
                        self.refresh_runners();
                    }
                });
            });

            if cfg!(target_os = "windows") {
                super::empty_state(
                    ui,
                    "🎮",
                    tr("Unter Windows nicht nötig", "Not needed on Windows"),
                    tr(
                        "Spiele laufen nativ – Runner sind nur für Linux relevant.",
                        "Games run natively – runners only matter on Linux.",
                    ),
                );
                return;
            }

            // ---------------------------------------------------- installed
            theme::section_label(
                ui,
                &format!(
                    "{} · {}",
                    tr("Installiert", "Installed"),
                    self.runners.len()
                ),
            );
            if self.busy.contains("runners") && !self.runners_loaded {
                ui.spinner();
            } else if self.runners.is_empty() {
                super::empty_state(
                    ui,
                    "🍷",
                    tr("Keine Runner gefunden", "No runners found"),
                    tr(
                        "Installiere unten einen Proton-Build.",
                        "Install a Proton build below.",
                    ),
                );
            }
            let mut remove = None;
            for runner in &self.runners {
                egui::Frame::new()
                    .fill(alpha(theme::PANEL, 0.85))
                    .stroke(Stroke::new(1.0, theme::BORDER_SOFT))
                    .corner_radius(12)
                    .inner_margin(egui::Margin::symmetric(14, 10))
                    .show(ui, |ui| {
                        ui.set_width(ui.available_width());
                        ui.horizontal(|ui| {
                            let color = if runner.kind == "wine" {
                                theme::WARN
                            } else {
                                theme::BRAND
                            };
                            theme::pill(ui, &runner.kind.to_uppercase(), color);
                            ui.vertical(|ui| {
                                ui.label(
                                    RichText::new(&runner.name)
                                        .font(theme::font_semi(13.5))
                                        .color(theme::TEXT),
                                );
                                ui.add(
                                    egui::Label::new(
                                        RichText::new(&runner.path)
                                            .font(theme::font(11.0))
                                            .color(theme::MUTED),
                                    )
                                    .truncate(),
                                );
                            });
                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    if runner.removable
                                        && ui
                                            .button("🗑")
                                            .on_hover_text(tr("Entfernen", "Remove"))
                                            .clicked()
                                    {
                                        remove = Some(runner.path.clone());
                                    }
                                    theme::pill(ui, &runner.source, theme::TEXT_DIM);
                                    theme::pill(ui, &runner.recommended_backend, theme::MUTED);
                                },
                            );
                        });
                    });
                ui.add_space(4.0);
            }
            if let Some(path) = remove {
                self.backend
                    .blocking(move || Msg::RunnerRemoved(gachahub_core::runners::remove(&path)));
            }

            // ---------------------------------------------------- families
            ui.add_space(18.0);
            theme::section_label(ui, tr("Runner herunterladen", "Download runners"));
            let families = gachahub_core::runners::families();
            ui.horizontal_wrapped(|ui| {
                for fam in families.iter().filter(|f| f.installable) {
                    let active = self.runner_family == fam.id;
                    let (rect, resp) =
                        ui.allocate_exact_size(Vec2::new(196.0, 64.0), Sense::click());
                    let hover = ui.ctx().animate_bool(resp.id, resp.hovered());
                    let p = ui.painter();
                    p.rect(
                        rect,
                        CornerRadius::same(14),
                        if active {
                            alpha(theme::BRAND, 0.1)
                        } else {
                            alpha(theme::PANEL, 0.8 + hover * 0.2)
                        },
                        Stroke::new(
                            1.0,
                            if active {
                                alpha(theme::BRAND, 0.5)
                            } else {
                                theme::BORDER_SOFT
                            },
                        ),
                        StrokeKind::Inside,
                    );
                    let badge = egui::Rect::from_min_size(
                        rect.min + Vec2::new(12.0, 14.0),
                        Vec2::splat(36.0),
                    );
                    theme::paint_game_badge(
                        ui,
                        badge,
                        &fam.short_name,
                        theme::BRAND,
                        theme::BRAND_2,
                        active,
                        hover,
                    );
                    p.text(
                        badge.right_center() + Vec2::new(10.0, -7.0),
                        egui::Align2::LEFT_CENTER,
                        &fam.name,
                        theme::font_semi(12.5),
                        theme::TEXT,
                    );
                    p.text(
                        badge.right_center() + Vec2::new(10.0, 9.0),
                        egui::Align2::LEFT_CENTER,
                        &fam.source,
                        theme::font(10.5),
                        theme::MUTED,
                    );
                    if resp
                        .on_hover_text(&fam.description)
                        .on_hover_cursor(egui::CursorIcon::PointingHand)
                        .clicked()
                    {
                        self.runner_family = fam.id.clone();
                    }
                }
            });
            ui.add_space(10.0);

            let fam_id = self.runner_family.clone();
            if !self.runner_versions.contains_key(&fam_id)
                && !self.busy.contains(&format!("versions:{fam_id}"))
            {
                self.busy.insert(format!("versions:{fam_id}"));
                let f = fam_id.clone();
                self.backend.spawn(async move {
                    Msg::RunnerVersions(f.clone(), gachahub_core::runners::versions(&f).await)
                });
            }
            match self.runner_versions.get(&fam_id).cloned() {
                None => {
                    ui.horizontal(|ui| {
                        ui.spinner();
                        ui.label(
                            RichText::new(tr("Lade Releases…", "Loading releases…"))
                                .color(theme::MUTED),
                        );
                    });
                }
                Some(Err(e)) => {
                    ui.add(egui::Label::new(RichText::new(e).color(theme::ERR)).wrap());
                    if ui.button(tr("Erneut versuchen", "Retry")).clicked() {
                        self.runner_versions.remove(&fam_id);
                    }
                }
                Some(Ok(list)) => {
                    for v in list.iter().take(12) {
                        egui::Frame::new()
                            .fill(alpha(theme::PANEL, 0.6))
                            .stroke(Stroke::new(1.0, theme::BORDER_SOFT))
                            .corner_radius(12)
                            .inner_margin(egui::Margin::symmetric(14, 9))
                            .show(ui, |ui| {
                                ui.set_width(ui.available_width());
                                ui.horizontal(|ui| {
                                    ui.label(
                                        RichText::new(&v.name)
                                            .font(theme::font_semi(13.0))
                                            .color(theme::TEXT),
                                    );
                                    if v.prerelease {
                                        theme::pill(ui, "pre", theme::WARN);
                                    }
                                    ui.label(
                                        RichText::new(v.published_at.get(..10).unwrap_or(""))
                                            .font(theme::font(11.0))
                                            .color(theme::MUTED),
                                    );
                                    ui.with_layout(
                                        egui::Layout::right_to_left(egui::Align::Center),
                                        |ui| {
                                            let key = format!("runner-install:{}", v.tag_name);
                                            if v.installed {
                                                theme::pill(
                                                    ui,
                                                    tr("✔ installiert", "✔ installed"),
                                                    theme::OK,
                                                );
                                            } else if self.busy.contains(&key) {
                                                ui.spinner();
                                                ui.label(
                                                    RichText::new(tr(
                                                        "Installiere…",
                                                        "Installing…",
                                                    ))
                                                    .color(theme::MUTED),
                                                );
                                            } else if ui
                                                .button(tr("⬇  Installieren", "⬇  Install"))
                                                .clicked()
                                            {
                                                self.busy.insert(key);
                                                let (f, t) =
                                                    (v.family_id.clone(), v.tag_name.clone());
                                                self.backend.spawn(async move {
                                                    Msg::RunnerInstalled(
                                                        gachahub_core::runners::install(&f, &t)
                                                            .await,
                                                    )
                                                });
                                            }
                                            ui.label(
                                                RichText::new(theme::fmt_bytes(v.size))
                                                    .font(theme::font(11.0))
                                                    .color(theme::MUTED),
                                            );
                                        },
                                    );
                                });
                            });
                        ui.add_space(3.0);
                    }
                }
            }
        });
    }
}
