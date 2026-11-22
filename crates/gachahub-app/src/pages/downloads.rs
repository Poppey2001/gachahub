use crate::{
    app::GachaHubApp,
    i18n::tr,
    pages::game::phase_label,
    theme::{self, alpha},
};
use eframe::egui::{self, Align2, Color32, RichText, Sense, Vec2};

impl GachaHubApp {
    pub fn page_downloads(&mut self, ui: &mut egui::Ui) {
        super::page_frame(ui, |ui| {
            theme::page_header(
                ui,
                tr("Warteschlange", "Queue"),
                "Downloads",
                tr(
                    "Fortsetzbare Downloads mit Pause, Prüfsummen und HTTP-Range-Resume.",
                    "Resumable downloads with pause, checksums and HTTP range resume.",
                ),
            );

            let active: Vec<_> = self
                .jobs
                .iter()
                .filter(|j| {
                    !matches!(
                        j.phase.as_str(),
                        "ready" | "cancelled" | "downloaded" | "error"
                    )
                })
                .cloned()
                .collect();
            let finished: Vec<_> = self
                .jobs
                .iter()
                .filter(|j| {
                    matches!(
                        j.phase.as_str(),
                        "ready" | "cancelled" | "downloaded" | "error"
                    )
                })
                .cloned()
                .collect();

            theme::section_label(ui, &format!("{} · {}", tr("Aktiv", "Active"), active.len()));
            if active.is_empty() {
                super::empty_state(
                    ui,
                    "⬇",
                    tr("Keine aktiven Downloads", "No active downloads"),
                    tr(
                        "Starte eine Installation auf der Spielseite.",
                        "Start an install from a game page.",
                    ),
                );
            }
            for job in &active {
                let Some(game) = self.game(&job.game_id).cloned() else {
                    continue;
                };
                let (a, b) = Self::accents(&game);
                theme::card().show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    ui.horizontal(|ui| {
                        theme::game_badge(ui, 52.0, &game.short_name, a, b, true, Sense::hover());
                        ui.add_space(6.0);
                        ui.vertical(|ui| {
                            ui.horizontal(|ui| {
                                ui.label(
                                    RichText::new(&game.name)
                                        .font(theme::font_semi(15.0))
                                        .color(theme::TEXT),
                                );
                                theme::pill(ui, phase_label(&job.phase), a);
                                if let Some(v) = &job.version {
                                    ui.label(
                                        RichText::new(format!("v{v}"))
                                            .font(theme::font(11.0))
                                            .color(theme::MUTED),
                                    );
                                }
                                ui.with_layout(
                                    egui::Layout::right_to_left(egui::Align::Center),
                                    |ui| {
                                        if ui
                                            .button("✖")
                                            .on_hover_text(tr("Abbrechen", "Cancel"))
                                            .clicked()
                                        {
                                            let _ = gachahub_core::download_manager::cancel(
                                                &self.sink, &job.id,
                                            );
                                        }
                                        let paused = job.phase == "paused";
                                        if ui.button(if paused { "▶" } else { "⏸" }).clicked() {
                                            let _ = gachahub_core::download_manager::pause(
                                                &self.sink, &job.id, !paused,
                                            );
                                        }
                                    },
                                );
                            });
                            theme::progress_bar(ui, job.progress, a, b, 6.0);
                            ui.horizontal(|ui| {
                                let meta = format!(
                                    "{:.1}%   ·   {} / {}   ·   {}/s",
                                    job.progress * 100.0,
                                    theme::fmt_bytes(job.downloaded_bytes),
                                    theme::fmt_bytes(job.total_bytes),
                                    theme::fmt_bytes(job.speed_bytes)
                                );
                                ui.label(
                                    RichText::new(meta)
                                        .font(theme::font(11.5))
                                        .color(theme::MUTED),
                                );
                                if job.speed_bytes > 0 && job.total_bytes > job.downloaded_bytes {
                                    let eta = (job.total_bytes - job.downloaded_bytes)
                                        / job.speed_bytes.max(1);
                                    ui.label(
                                        RichText::new(format!(
                                            "·   ETA {}",
                                            theme::fmt_duration(eta)
                                        ))
                                        .font(theme::font(11.5))
                                        .color(theme::MUTED),
                                    );
                                }
                            });
                            if let Some(m) = &job.message {
                                ui.label(
                                    RichText::new(m).font(theme::font(11.0)).color(theme::FAINT),
                                );
                            }
                        });
                    });
                });
                ui.add_space(6.0);
            }

            if !finished.is_empty() {
                ui.add_space(14.0);
                ui.horizontal(|ui| {
                    theme::section_label(
                        ui,
                        &format!("{} · {}", tr("Verlauf", "History"), finished.len()),
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.small_button(tr("Leeren", "Clear")).clicked() {
                            self.jobs.retain(|j| {
                                !matches!(
                                    j.phase.as_str(),
                                    "ready" | "cancelled" | "downloaded" | "error"
                                )
                            });
                        }
                    });
                });
                for job in finished.iter().rev() {
                    let name = self
                        .game(&job.game_id)
                        .map(|g| g.name.clone())
                        .unwrap_or_else(|| job.game_id.clone());
                    let color = match job.phase.as_str() {
                        "error" => theme::ERR,
                        "cancelled" => theme::WARN,
                        _ => theme::OK,
                    };
                    egui::Frame::new()
                        .fill(alpha(theme::PANEL, 0.6))
                        .stroke(egui::Stroke::new(1.0, theme::BORDER_SOFT))
                        .corner_radius(12)
                        .inner_margin(egui::Margin::symmetric(14, 10))
                        .show(ui, |ui| {
                            ui.set_width(ui.available_width());
                            ui.horizontal(|ui| {
                                let (r, _) =
                                    ui.allocate_exact_size(Vec2::splat(10.0), Sense::hover());
                                ui.painter().circle_filled(r.center(), 4.0, color);
                                ui.label(
                                    RichText::new(&name)
                                        .font(theme::font_semi(13.0))
                                        .color(theme::TEXT),
                                );
                                ui.label(
                                    RichText::new(phase_label(&job.phase))
                                        .color(color)
                                        .font(theme::font(12.0)),
                                );
                                if let Some(m) = &job.message {
                                    ui.add(
                                        egui::Label::new(
                                            RichText::new(m)
                                                .font(theme::font(11.5))
                                                .color(theme::MUTED),
                                        )
                                        .truncate(),
                                    );
                                }
                            });
                        });
                    ui.add_space(4.0);
                }
            }
            let _ = (Align2::LEFT_TOP, Color32::WHITE);
        });
    }
}
