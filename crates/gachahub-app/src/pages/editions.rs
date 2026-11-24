//! Edition picker for games with several official PC builds (e.g. GFL2:
//! Darkwinter / Haoplay). GachaHub downloads the chosen official installer,
//! runs it and then watches for the finished installation.

use crate::{
    app::{GachaHubApp, ToastKind},
    backend::Msg,
    i18n::{is_german, tr},
    theme::{self, alpha, mix},
};
use eframe::egui::{self, Color32, CornerRadius, RichText, Sense, Stroke, StrokeKind, Vec2};
use gachahub_core::games::OfficialLauncher;
use std::path::PathBuf;

/// How long GachaHub keeps looking for the installation after running an installer.
const WATCH_SECS: f64 = 3.0 * 3600.0;
const SCAN_EVERY_SECS: f64 = 15.0;

fn installer_dir() -> PathBuf {
    directories::ProjectDirs::from("", "", "gachahub")
        .map(|d| d.cache_dir().to_path_buf())
        .unwrap_or_else(std::env::temp_dir)
        .join("installers")
}

impl GachaHubApp {
    fn edition(&self, game_id: &str, edition: &str) -> Option<OfficialLauncher> {
        self.game(game_id)?
            .official_launchers
            .iter()
            .find(|l| l.id == edition)
            .cloned()
    }

    /// Download `edition` directly with GachaHub. Each edition gets its own
    /// folder; switching edition starts a fresh install there.
    pub fn download_edition(&mut self, game_id: &str, edition: &str) {
        let previous = self.cfg.games.get(game_id).and_then(|g| g.edition.clone());
        let default_dir = format!(
            "{}/{game_id}-{edition}",
            self.cfg.global.default_install_path
        );
        let gs = self.cfg.game_mut(game_id);
        let switched = previous.as_deref().is_some_and(|p| p != edition);
        if switched || gs.install_path.is_none() {
            gs.install_path = Some(default_dir);
            gs.executable_path = None;
            gs.steam_app_id = None;
        }
        gs.edition = Some(edition.to_string());
        self.dirty_now();
        self.start_job(game_id, gachahub_core::download_manager::JobKind::Install);
    }

    /// Download and run the official installer of `edition`.
    pub fn install_official(&mut self, game_id: &str, edition: &str) {
        let Some(launcher) = self.edition(game_id, edition) else {
            return;
        };
        self.cfg.game_mut(game_id).edition = Some(edition.to_string());
        self.dirty_now();
        if launcher.installer_urls.is_empty() {
            self.open_official_website(game_id, &launcher);
            return;
        }
        let gid = game_id.to_string();
        let ed = edition.to_string();
        let sender = self.backend.sender();
        self.installer_progress.insert(gid.clone(), (0, 0));
        self.backend.spawn(async move {
            let dir = installer_dir().join(&ed);
            let progress_gid = gid.clone();
            let last = std::sync::Mutex::new(std::time::Instant::now());
            let res =
                gachahub_core::official::download_installer(&launcher, &dir, move |done, total| {
                    let mut last = last.lock().unwrap_or_else(|e| e.into_inner());
                    if last.elapsed().as_millis() > 150 || done == total {
                        *last = std::time::Instant::now();
                        sender.send(Msg::InstallerProgress(progress_gid.clone(), done, total));
                    }
                })
                .await;
            Msg::InstallerReady(gid, ed, res)
        });
    }

    fn open_official_website(&mut self, game_id: &str, launcher: &OfficialLauncher) {
        match open::that_detached(&launcher.website) {
            Ok(()) => {
                self.toast(
                    ToastKind::Info,
                    format!(
                        "{} – {}",
                        launcher.label_for(is_german()),
                        tr(
                            "offizielle Seite geöffnet. Lade dort den PC-Launcher herunter und installiere das Spiel – GachaHub erkennt es danach automatisch.",
                            "official site opened. Download the PC launcher there and install the game – GachaHub detects it automatically afterwards.",
                        )
                    ),
                );
                self.watch_official(game_id);
            }
            Err(e) => self.toast(ToastKind::Error, format!("{}: {e}", launcher.website)),
        }
    }

    fn watch_official(&mut self, game_id: &str) {
        self.official_watch
            .insert(game_id.to_string(), self.now + WATCH_SECS);
        // scan soon
        self.last_official_scan = self.now - SCAN_EVERY_SECS + 3.0;
    }

    pub fn on_installer_ready(
        &mut self,
        game_id: &str,
        edition: &str,
        res: Result<PathBuf, String>,
    ) {
        let Some(launcher) = self.edition(game_id, edition) else {
            return;
        };
        let installer = match res {
            Ok(p) => p,
            Err(e) => {
                // Download link outdated or blocked → official website instead.
                self.toast(
                    ToastKind::Error,
                    format!(
                        "{}: {e}",
                        tr(
                            "Installer-Download fehlgeschlagen",
                            "Installer download failed"
                        )
                    ),
                );
                self.open_official_website(game_id, &launcher);
                return;
            }
        };
        let r = self.cfg.resolved(game_id);
        let gid = game_id.to_string();
        let exe = installer.to_string_lossy().into_owned();
        self.backend.blocking(move || {
            match GachaHubApp::spawn_for_game(&gid, &r, &exe, Vec::new()) {
                Ok(_) => Msg::Notice(Ok(tr(
                    "Offizieller Installer gestartet – installiere das Spiel dort. GachaHub erkennt es danach automatisch.",
                    "Official installer started – install the game there. GachaHub detects it automatically afterwards.",
                )
                .into())),
                Err(e) => Msg::Notice(Err(e)),
            }
        });
        self.watch_official(game_id);
    }

    /// Periodically look for installations started through an official launcher.
    pub fn official_watch_tick(&mut self) {
        let now = self.now;
        self.official_watch.retain(|_, until| *until > now);
        if self.official_watch.is_empty() || now - self.last_official_scan < SCAN_EVERY_SECS {
            return;
        }
        self.last_official_scan = now;
        for game_id in self.official_watch.keys().cloned().collect::<Vec<_>>() {
            let Some(game) = self.game(&game_id).cloned() else {
                continue;
            };
            let mut names: Vec<String> = Vec::new();
            for l in &game.official_launchers {
                for n in &l.detect_names {
                    if !names.contains(n) {
                        names.push(n.clone());
                    }
                }
            }
            let prefix = gachahub_core::expand_user(&self.cfg.resolved(&game_id).prefix_path);
            self.backend.blocking(move || {
                let found = gachahub_core::official::detect(&game_id, &names, Some(&prefix));
                Msg::OfficialDetected(game_id, found)
            });
        }
    }

    pub fn on_official_detected(&mut self, game_id: &str, dir: PathBuf, exe: PathBuf) {
        if self.official_watch.remove(game_id).is_none() {
            return;
        }
        if !self.cfg.resolved(game_id).executable_path.is_empty() {
            return;
        }
        let gs = self.cfg.game_mut(game_id);
        gs.install_path = Some(dir.to_string_lossy().into_owned());
        gs.executable_path = Some(exe.to_string_lossy().into_owned());
        gs.installation_detected_at = Some(crate::app::now_unix().to_string());
        if !self.cfg.library.iter().any(|g| g == game_id) {
            self.cfg.library.push(game_id.to_string());
        }
        self.game_errors.remove(game_id);
        self.dirty_now();
        let name = self
            .game(game_id)
            .map(|g| g.name.clone())
            .unwrap_or_default();
        self.toast(
            ToastKind::Success,
            format!(
                "{name}: {}",
                tr(
                    "Installation erkannt – bereit zum Spielen",
                    "installation detected – ready to play"
                )
            ),
        );
    }

    /// Modal: choose which official edition to install.
    pub fn edition_modal(&mut self, ctx: &egui::Context) {
        let Some(game_id) = self.edition_picker.clone() else {
            return;
        };
        let Some(game) = self.game(&game_id).cloned() else {
            self.edition_picker = None;
            return;
        };
        let (a, b) = Self::accents(&game);
        let german = is_german();
        let current = self.cfg.games.get(&game_id).and_then(|g| g.edition.clone());
        let direct = gachahub_core::download_manager::supports_direct_download(&game_id);
        let mut chosen: Option<String> = None;
        let mut official: Option<String> = None;
        let mut steam = false;
        let modal = egui::Modal::new(egui::Id::unique(("editions", game_id.as_str())))
            .backdrop_color(Color32::from_black_alpha(170))
            .frame(
                egui::Frame::new()
                    .fill(theme::BG_RAISED)
                    .stroke(Stroke::new(1.0, theme::BORDER))
                    .corner_radius(22)
                    .inner_margin(26),
            )
            .show(ctx, |ui| {
                ui.set_width(520.0);
                ui.label(
                    RichText::new(tr("VERSION WÄHLEN", "CHOOSE EDITION"))
                        .font(theme::font_heavy(10.5))
                        .color(a)
                        .extra_letter_spacing(2.0),
                );
                ui.label(RichText::new(&game.name).font(theme::font_heavy(22.0)).color(theme::TEXT));
                ui.label(
                    RichText::new(if direct {
                        tr(
                            "Dein Konto gehört zu einem Server – wähle die passende Version. GachaHub lädt das Spiel direkt von den offiziellen Servern herunter (fortsetzbar, mit Updates und Dateiprüfung).",
                            "Your account belongs to one server – pick the matching edition. GachaHub downloads the game straight from the official servers (resumable, with updates and file verification).",
                        )
                    } else {
                        tr(
                            "Dein Konto gehört zu einem Server – wähle die passende Version. GachaHub lädt den offiziellen Installer, startet ihn und übernimmt das Spiel danach automatisch.",
                            "Your account belongs to one server – pick the matching edition. GachaHub downloads the official installer, runs it and picks the game up automatically afterwards.",
                        )
                    })
                    .color(theme::MUTED),
                );
                ui.add_space(14.0);
                for l in &game.official_launchers {
                    let selected = current.as_deref() == Some(l.id.as_str());
                    let (rect, resp) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 66.0), Sense::click());
                    let hover = ui.ctx().animate_bool(resp.id, resp.hovered());
                    let p = ui.painter();
                    p.rect_filled(rect, CornerRadius::same(14), mix(theme::PANEL, a, 0.05 + hover * 0.07));
                    p.rect_stroke(
                        rect,
                        CornerRadius::same(14),
                        Stroke::new(1.0, if selected { alpha(a, 0.7) } else { mix(theme::BORDER, b, hover * 0.6) }),
                        StrokeKind::Inside,
                    );
                    p.text(
                        rect.left_top() + Vec2::new(16.0, 14.0),
                        egui::Align2::LEFT_TOP,
                        l.label_for(german),
                        theme::font_heavy(16.0),
                        theme::TEXT,
                    );
                    p.text(
                        rect.left_top() + Vec2::new(16.0, 38.0),
                        egui::Align2::LEFT_TOP,
                        l.region_for(german),
                        theme::font(11.5),
                        theme::MUTED,
                    );
                    let hint = if direct {
                        tr("⬇ Direkt herunterladen", "⬇ Download")
                    } else if l.installer_urls.is_empty() {
                        tr("Offizielle Seite ↗", "Official site ↗")
                    } else {
                        tr("⬇ Installer laden", "⬇ Get installer")
                    };
                    p.text(
                        rect.right_center() - Vec2::new(16.0, 0.0),
                        egui::Align2::RIGHT_CENTER,
                        hint,
                        theme::font_semi(12.0),
                        mix(a, Color32::WHITE, 0.2),
                    );
                    if resp.on_hover_cursor(egui::CursorIcon::PointingHand).clicked() {
                        chosen = Some(l.id.clone());
                    }
                    ui.add_space(8.0);
                }
                if direct {
                    ui.horizontal_wrapped(|ui| {
                        ui.label(
                            RichText::new(tr("Alternativ offizieller Launcher:", "Or the official launcher:"))
                                .font(theme::font(11.5))
                                .color(theme::MUTED),
                        );
                        for l in &game.official_launchers {
                            if ui.small_button(l.label_for(german)).clicked() {
                                official = Some(l.id.clone());
                            }
                        }
                    });
                }
                ui.add_space(6.0);
                ui.horizontal(|ui| {
                    if !game.steam_names.is_empty()
                        && theme::ghost_button(ui, "🎮", tr("Über Steam", "Via Steam")).clicked()
                    {
                        steam = true;
                    }
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button(tr("Abbrechen", "Cancel")).clicked() {
                            self.edition_picker = None;
                        }
                    });
                });
            });
        if modal.should_close() {
            self.edition_picker = None;
        }
        if let Some(ed) = chosen {
            self.edition_picker = None;
            if direct {
                self.download_edition(&game_id, &ed);
            } else {
                self.install_official(&game_id, &ed);
            }
        } else if let Some(ed) = official {
            self.edition_picker = None;
            self.install_official(&game_id, &ed);
        } else if steam {
            self.edition_picker = None;
            let now = self.now;
            self.install_via_steam(&game, now);
        }
    }
}
