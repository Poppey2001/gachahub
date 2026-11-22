//! Application state, message handling and the top-level layout
//! (game rail on the left, the active page on the right).

use crate::{
    backend::{Backend, Msg, PickTarget},
    i18n::{set_language, tr},
    theme::{self, alpha, hex, mix},
};
use eframe::egui::{
    self, Align2, Color32, CornerRadius, Margin, Pos2, Rect, Sense, Stroke, StrokeKind, Vec2,
};
use gachahub_core::{
    install::InstallJob,
    mod_manager::ModSnapshot,
    runners::{RunnerInfo, RunnerVersion},
    settings::Config,
    xxmi::{XxmiManagerStatus, XxmiProgress},
    CoreEvent, EventSink, Game, GameRegistry,
};
use std::{
    collections::{HashMap, HashSet},
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Section {
    Game,
    Downloads,
    Mods,
    Runners,
    Settings,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingsTab {
    General,
    Game,
    Xxmi,
}

#[derive(Clone, Copy, PartialEq)]
pub enum ToastKind {
    Info,
    Success,
    Error,
}

pub struct Toast {
    pub text: String,
    pub kind: ToastKind,
    pub born: f64,
}

#[derive(Default, Clone)]
pub struct Artwork {
    pub background: Option<String>,
    pub icon: Option<String>,
    pub logo: Option<String>,
}

pub struct GachaHubApp {
    pub backend: Backend,
    pub sink: EventSink,
    pub cfg: Config,
    dirty_since: Option<f64>,

    pub games: Vec<Game>,
    pub artwork: HashMap<String, Artwork>,
    pub section: Section,
    pub catalog_open: bool,

    pub jobs: Vec<InstallJob>,
    pub busy: HashSet<String>,
    pub game_errors: HashMap<String, String>,
    pub running: HashMap<String, f64>,
    toasts: Vec<Toast>,

    pub runners: Vec<RunnerInfo>,
    pub runners_loaded: bool,
    pub runner_family: String,
    pub runner_versions: HashMap<String, Result<Vec<RunnerVersion>, String>>,

    pub xxmi: Option<XxmiManagerStatus>,
    pub xxmi_progress: Option<XxmiProgress>,

    pub mods: HashMap<String, Result<ModSnapshot, String>>,
    pub mods_game: String,
    pub mod_filter: String,

    pub settings_tab: SettingsTab,
    pub settings_game: String,
}

pub fn now_unix() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

/// Directories searched for optional artwork (`<dir>/<game-id>/background.webp`, ...).
fn artwork_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    if let Some(d) = directories::ProjectDirs::from("", "", "gachahub") {
        dirs.push(d.data_dir().join("game-assets"));
    }
    if let Ok(exe) = std::env::current_exe() {
        if let Some(parent) = exe.parent() {
            dirs.push(parent.join("game-assets"));
            dirs.push(parent.join("assets").join("game-assets"));
        }
    }
    dirs.push(PathBuf::from(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../assets/game-assets"
    )));
    dirs
}

fn find_asset(dirs: &[PathBuf], game_id: &str, stem: &str) -> Option<String> {
    for dir in dirs {
        for ext in ["webp", "png", "jpg", "jpeg"] {
            let path = dir.join(game_id).join(format!("{stem}.{ext}"));
            if path.is_file() {
                let path = path.canonicalize().unwrap_or(path);
                return Some(format!("file://{}", path.display()));
            }
        }
    }
    None
}

impl GachaHubApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        theme::install(&cc.egui_ctx);
        egui_extras::install_image_loaders(&cc.egui_ctx);

        let backend = Backend::new(cc.egui_ctx.clone());
        let sink = backend.event_sink();
        let mut cfg = Config::load();
        set_language(cfg.global.language);

        let games = GameRegistry::builtin().games;
        cfg.library.retain(|id| games.iter().any(|g| &g.id == id));
        if cfg.library.is_empty() {
            cfg.library
                .push(games.first().map(|g| g.id.clone()).unwrap_or_default());
        }
        if !cfg.library.contains(&cfg.selected_game) {
            cfg.selected_game = cfg.library[0].clone();
        }

        let dirs = artwork_dirs();
        let artwork = games
            .iter()
            .map(|g| {
                (
                    g.id.clone(),
                    Artwork {
                        background: find_asset(&dirs, &g.id, "background"),
                        icon: find_asset(&dirs, &g.id, "icon"),
                        logo: find_asset(&dirs, &g.id, "logo"),
                    },
                )
            })
            .collect();

        let selected = cfg.selected_game.clone();
        let mut app = Self {
            backend,
            sink,
            cfg,
            dirty_since: None,
            games,
            artwork,
            section: Section::Game,
            catalog_open: false,
            jobs: Vec::new(),
            busy: HashSet::new(),
            game_errors: HashMap::new(),
            running: HashMap::new(),
            toasts: Vec::new(),
            runners: Vec::new(),
            runners_loaded: false,
            runner_family: "ge-proton".into(),
            runner_versions: HashMap::new(),
            xxmi: None,
            xxmi_progress: None,
            mods: HashMap::new(),
            mods_game: selected.clone(),
            mod_filter: String::new(),
            settings_tab: SettingsTab::General,
            settings_game: selected,
        };
        // Developer aid: open a specific page on start (used for screenshots/CI).
        if let Ok(page) = std::env::var("GACHAHUB_START_PAGE") {
            app.section = match page.as_str() {
                "downloads" => Section::Downloads,
                "mods" => Section::Mods,
                "runners" => Section::Runners,
                "settings" => Section::Settings,
                "catalog" => {
                    app.catalog_open = true;
                    Section::Game
                }
                _ => Section::Game,
            };
            if app.section == Section::Mods {
                let id = app.mods_game.clone();
                app.refresh_mods(&id);
            }
        }
        app.refresh_runners();
        app.refresh_xxmi();
        app
    }

    // ------------------------------------------------------------ helpers

    pub fn game(&self, id: &str) -> Option<&Game> {
        self.games.iter().find(|g| g.id == id)
    }

    pub fn selected_game(&self) -> Game {
        self.game(&self.cfg.selected_game)
            .cloned()
            .unwrap_or_else(|| self.games[0].clone())
    }

    pub fn accents(game: &Game) -> (Color32, Color32) {
        (hex(&game.accent), hex(&game.accent2))
    }

    pub fn mark_dirty(&mut self, ctx: &egui::Context) {
        if self.dirty_since.is_none() {
            self.dirty_since = Some(ctx.input(|i| i.time));
        }
    }

    pub fn toast(&mut self, kind: ToastKind, text: impl Into<String>) {
        self.toasts.push(Toast {
            text: text.into(),
            kind,
            born: -1.0,
        });
    }

    pub fn select_game(&mut self, id: &str) {
        if !self.cfg.library.iter().any(|g| g == id) {
            self.cfg.library.push(id.to_string());
        }
        self.cfg.selected_game = id.to_string();
        self.section = Section::Game;
        self.dirty_since.get_or_insert(0.0);
    }

    pub fn job_for(&self, game_id: &str) -> Option<&InstallJob> {
        self.jobs.iter().rev().find(|j| j.game_id == game_id)
    }

    pub fn active_jobs(&self) -> usize {
        self.jobs
            .iter()
            .filter(|j| {
                !matches!(
                    j.phase.as_str(),
                    "ready" | "error" | "cancelled" | "downloaded"
                )
            })
            .count()
    }

    fn upsert_job(&mut self, job: InstallJob) {
        if let Some(existing) = self.jobs.iter_mut().find(|j| j.id == job.id) {
            *existing = job;
        } else {
            self.jobs.push(job);
        }
    }

    pub fn refresh_runners(&mut self) {
        self.busy.insert("runners".into());
        self.backend
            .blocking(|| Msg::Runners(gachahub_core::runners::detect()));
    }

    pub fn refresh_xxmi(&mut self) {
        let g = &self.cfg.global;
        let (install, custom, wine, prefix) = (
            g.xxmi_install_path.clone(),
            g.xxmi_launcher_path.clone(),
            g.xxmi_wine_executable.clone(),
            g.xxmi_wine_prefix.clone(),
        );
        self.busy.insert("xxmi-status".into());
        self.backend.spawn(async move {
            Msg::Xxmi(gachahub_core::xxmi::manager_status(&install, &custom, &wine, &prefix).await)
        });
    }

    pub fn refresh_mods(&mut self, game_id: &str) {
        let r = self.cfg.resolved(game_id);
        let gid = game_id.to_string();
        self.busy.insert(format!("mods:{game_id}"));
        self.backend.blocking(move || {
            let res =
                gachahub_core::mod_manager::scan(&gid, &r.mod_library_path, &r.active_mods_path);
            Msg::Mods(gid, res)
        });
    }

    // ------------------------------------------------------------ messages

    fn process_messages(&mut self, ctx: &egui::Context) {
        for msg in self.backend.drain() {
            match msg {
                Msg::Core(CoreEvent::Download(job)) => {
                    if job.phase == "downloaded" {
                        let name = self
                            .game(&job.game_id)
                            .map(|g| g.name.clone())
                            .unwrap_or_default();
                        self.toast(
                            ToastKind::Success,
                            format!(
                                "{name}: {}",
                                tr("Download abgeschlossen", "download finished")
                            ),
                        );
                    }
                    self.upsert_job(job);
                }
                Msg::Core(CoreEvent::Xxmi(progress)) => {
                    let done = matches!(progress.phase.as_str(), "ready" | "error");
                    self.xxmi_progress = Some(progress);
                    if done {
                        self.busy.remove("xxmi-install");
                    }
                }
                Msg::DownloadStarted(game_id, res) => {
                    self.busy.remove(&format!("install:{game_id}"));
                    match res {
                        Ok(job) => {
                            self.game_errors.remove(&game_id);
                            self.upsert_job(job);
                        }
                        Err(e) => {
                            self.game_errors.insert(game_id, e);
                        }
                    }
                }
                Msg::Launched(game_id, res) => {
                    self.busy.remove(&format!("launch:{game_id}"));
                    match res {
                        Ok(result) => {
                            self.game_errors.remove(&game_id);
                            self.running.insert(game_id.clone(), ctx.input(|i| i.time));
                            self.cfg.game_mut(&game_id).last_played_unix = Some(now_unix());
                            self.mark_dirty(ctx);
                            self.toast(
                                ToastKind::Success,
                                format!(
                                    "{} (PID {})",
                                    tr("Spiel gestartet", "Game started"),
                                    result.pid
                                ),
                            );
                        }
                        Err(e) => {
                            self.game_errors.insert(game_id, e);
                        }
                    }
                }
                Msg::SessionEnded(game_id, secs) => {
                    self.running.remove(&game_id);
                    self.cfg.game_mut(&game_id).playtime_seconds += secs;
                    self.mark_dirty(ctx);
                }
                Msg::Runners(list) => {
                    self.busy.remove("runners");
                    self.runners = list;
                    self.runners_loaded = true;
                }
                Msg::RunnerVersions(family, res) => {
                    self.busy.remove(&format!("versions:{family}"));
                    self.runner_versions.insert(family, res);
                }
                Msg::RunnerInstalled(res) => {
                    self.busy.retain(|k| !k.starts_with("runner-install:"));
                    match res {
                        Ok(info) => {
                            self.toast(
                                ToastKind::Success,
                                format!("{} {}", info.name, tr("installiert", "installed")),
                            );
                            self.runner_versions.clear();
                            self.refresh_runners();
                        }
                        Err(e) => self.toast(ToastKind::Error, e),
                    }
                }
                Msg::RunnerRemoved(res) => {
                    match res {
                        Ok(()) => {
                            self.toast(ToastKind::Info, tr("Runner entfernt", "Runner removed"))
                        }
                        Err(e) => self.toast(ToastKind::Error, e),
                    }
                    self.runner_versions.clear();
                    self.refresh_runners();
                }
                Msg::Xxmi(res) => {
                    self.busy.remove("xxmi-status");
                    self.busy.remove("xxmi-install");
                    match res {
                        Ok(status) => {
                            let auto = self.cfg.global.xxmi_auto_update
                                && status.installed
                                && status.managed
                                && status.update_available
                                && self.xxmi.is_none();
                            self.xxmi = Some(status);
                            if auto {
                                self.install_xxmi();
                            }
                        }
                        Err(e) => {
                            if self
                                .xxmi_progress
                                .as_ref()
                                .is_some_and(|p| p.phase != "ready")
                            {
                                self.toast(ToastKind::Error, e);
                            }
                        }
                    }
                }
                Msg::XxmiRemoved(res) => {
                    match res {
                        Ok(()) => self.toast(ToastKind::Info, tr("XXMI entfernt", "XXMI removed")),
                        Err(e) => self.toast(ToastKind::Error, e),
                    }
                    self.xxmi_progress = None;
                    self.refresh_xxmi();
                }
                Msg::Mods(game_id, res) => {
                    self.busy.remove(&format!("mods:{game_id}"));
                    if let Err(e) = &res {
                        if self.section == Section::Mods
                            && !self.cfg.resolved(&game_id).active_mods_path.is_empty()
                        {
                            self.toast(ToastKind::Error, e.clone());
                        }
                    }
                    self.mods.insert(game_id, res);
                }
                Msg::PostInstall(game_id, res) => {
                    self.busy.remove(&format!("postinstall:{game_id}"));
                    match res {
                        Ok(result) => {
                            let gs = self.cfg.game_mut(&game_id);
                            gs.apply_detected(&result.settings);
                            if !result.executable_path.is_empty() {
                                gs.executable_path = Some(result.executable_path.clone());
                                gs.installation_detected_at = Some(now_unix().to_string());
                            }
                            for w in &result.warnings {
                                self.toast(ToastKind::Info, w.clone());
                            }
                            if result.executable_path.is_empty() {
                                self.game_errors.insert(
                                    game_id,
                                    tr(
                                        "Keine Spiel-EXE im gewählten Ordner gefunden.",
                                        "No game executable found in the selected folder.",
                                    )
                                    .into(),
                                );
                            } else {
                                self.game_errors.remove(&game_id);
                                self.toast(
                                    ToastKind::Success,
                                    tr("Installation erkannt", "Installation detected"),
                                );
                            }
                            self.mark_dirty(ctx);
                        }
                        Err(e) => {
                            self.game_errors.insert(game_id, e);
                        }
                    }
                }
                Msg::Picked(target, path) => {
                    if let Some(path) = path {
                        self.apply_pick(target, path);
                        self.mark_dirty(ctx);
                    }
                }
                Msg::Notice(res) => match res {
                    Ok(t) => self.toast(ToastKind::Success, t),
                    Err(e) => self.toast(ToastKind::Error, e),
                },
            }
        }
    }

    fn apply_pick(&mut self, target: PickTarget, path: PathBuf) {
        let s = path.to_string_lossy().into_owned();
        match target {
            PickTarget::GameInstallPath(id) => self.cfg.game_mut(&id).install_path = Some(s),
            PickTarget::ImportExisting(id) => self.import_existing(&id, &path),
            PickTarget::GameExecutable(id) => {
                let gs = self.cfg.game_mut(&id);
                gs.executable_path = Some(s);
                gs.installation_detected_at = Some(now_unix().to_string());
                self.game_errors.remove(&id);
            }
            PickTarget::GamePrefix(id) => self.cfg.game_mut(&id).prefix_path = Some(s),
            PickTarget::GameRunnerPath(id) => self.cfg.game_mut(&id).runner_path = Some(s),
            PickTarget::GameModLibrary(id) => {
                self.cfg.game_mut(&id).mod_library_path = Some(s);
                self.refresh_mods(&id);
            }
            PickTarget::GameActiveMods(id) => {
                self.cfg.game_mut(&id).active_mods_path = Some(s);
                self.refresh_mods(&id);
            }
            PickTarget::ImportModFolder(id) | PickTarget::ImportModArchive(id) => {
                let r = self.cfg.resolved(&id);
                self.busy.insert(format!("mods:{id}"));
                self.backend.blocking(move || {
                    let res = gachahub_core::mod_manager::import_source(
                        &id,
                        &s,
                        &r.mod_library_path,
                        &r.active_mods_path,
                    );
                    Msg::Mods(id, res)
                });
            }
            PickTarget::GlobalInstallPath => self.cfg.global.default_install_path = s,
            PickTarget::GlobalPrefixBase => self.cfg.global.prefix_base_path = s,
            PickTarget::GlobalModRoot => self.cfg.global.mod_root_path = s,
            PickTarget::GlobalRunnerPath => self.cfg.global.default_runner_path = s,
            PickTarget::XxmiInstallPath => {
                self.cfg.global.xxmi_install_path = s;
                self.refresh_xxmi();
            }
            PickTarget::XxmiLauncherPath => {
                self.cfg.global.xxmi_launcher_path = s;
                self.refresh_xxmi();
            }
        }
    }

    /// Point a game at an existing installation. Tries to find the executable
    /// (via the post-install profile for Genshin, by scanning otherwise).
    fn import_existing(&mut self, game_id: &str, folder: &Path) {
        let folder_s = folder.to_string_lossy().into_owned();
        self.cfg.game_mut(game_id).install_path = Some(folder_s.clone());
        if game_id == "genshin" {
            let prefix = self.cfg.resolved(game_id).prefix_path;
            let gid = game_id.to_string();
            self.busy.insert(format!("postinstall:{game_id}"));
            self.backend.blocking(move || {
                let res = gachahub_core::postinstall::defaults(&gid, &folder_s, &prefix);
                Msg::PostInstall(gid, res)
            });
            return;
        }
        match find_executable(folder, game_id) {
            Some(exe) => {
                let gs = self.cfg.game_mut(game_id);
                gs.executable_path = Some(exe.to_string_lossy().into_owned());
                gs.installation_detected_at = Some(now_unix().to_string());
                self.game_errors.remove(game_id);
                self.toast(
                    ToastKind::Success,
                    tr("Installation erkannt", "Installation detected"),
                );
            }
            None => {
                self.game_errors.insert(
                    game_id.to_string(),
                    tr(
                        "Keine passende .exe gefunden – bitte in den Spiel-Einstellungen manuell auswählen.",
                        "No matching .exe found – please select it manually in the game settings.",
                    )
                    .into(),
                );
            }
        }
    }

    // ------------------------------------------------------------ actions

    pub fn start_install(&mut self, game_id: &str) {
        let install = self.cfg.resolved(game_id).install_path;
        let sink = self.sink.clone();
        let gid = game_id.to_string();
        self.busy.insert(format!("install:{game_id}"));
        self.game_errors.remove(game_id);
        self.backend.spawn(async move {
            let res = gachahub_core::download_manager::start(sink, gid.clone(), install).await;
            Msg::DownloadStarted(gid, res)
        });
    }

    pub fn launch(&mut self, game_id: &str) {
        let game = match self.game(game_id) {
            Some(g) => g.clone(),
            None => return,
        };
        let r = self.cfg.resolved(game_id);
        let g = self.cfg.global.clone();
        let gid = game_id.to_string();
        let sender = self.backend.sender();
        let use_xxmi = r.xxmi_enabled && game.xxmi_importer.is_some();
        self.busy.insert(format!("launch:{game_id}"));
        self.backend.blocking(move || {
            if use_xxmi {
                let importer = game.xxmi_importer.unwrap_or_default();
                let res = gachahub_core::xxmi::managed_launch(
                    &g.xxmi_install_path,
                    &g.xxmi_launcher_path,
                    &importer,
                    &g.xxmi_wine_executable,
                    &g.xxmi_wine_prefix,
                    true,
                )
                .map(|s| gachahub_core::launcher::LaunchResult {
                    pid: 0,
                    game_id: gid.clone(),
                    runtime: "xxmi".into(),
                    executable_path: s.launcher_path,
                    prefix_path: g.xxmi_wine_prefix.clone(),
                    runner_version: String::new(),
                    runner_path: String::new(),
                    command_preview: s.launch_command.join(" "),
                });
                return Msg::Launched(gid, res);
            }
            let res = gachahub_core::launcher::launch(
                &gid,
                &r.executable_path,
                r.runtime.as_str(),
                &r.runner_version,
                &r.runner_path,
                &r.prefix_path,
                &r.launch_arguments,
                r.environment_variables.clone(),
                r.gamescope_enabled,
                r.game_mode_enabled,
                r.mango_hud_enabled,
                r.prevent_sleep,
            );
            match res {
                Ok((result, child)) => {
                    crate::backend::track_session(sender, gid.clone(), child);
                    Msg::Launched(gid, Ok(result))
                }
                Err(e) => Msg::Launched(gid, Err(e)),
            }
        });
    }

    pub fn install_xxmi(&mut self) {
        let path = self.cfg.global.xxmi_install_path.clone();
        let sink = self.sink.clone();
        self.busy.insert("xxmi-install".into());
        self.xxmi_progress = None;
        self.backend.spawn(async move {
            Msg::Xxmi(gachahub_core::xxmi::install_managed(sink, &path).await)
        });
    }

    pub fn remove_xxmi(&mut self) {
        let path = self.cfg.global.xxmi_install_path.clone();
        self.backend.spawn(async move {
            Msg::XxmiRemoved(gachahub_core::xxmi::remove_managed(&path).await)
        });
    }

    /// Launch the XXMI GUI for first-time importer setup.
    pub fn open_xxmi_gui(&mut self, importer: &str) {
        let g = self.cfg.global.clone();
        let importer = importer.to_string();
        self.backend.blocking(move || {
            Msg::Notice(
                gachahub_core::xxmi::managed_launch(
                    &g.xxmi_install_path,
                    &g.xxmi_launcher_path,
                    &importer,
                    &g.xxmi_wine_executable,
                    &g.xxmi_wine_prefix,
                    false,
                )
                .map(|_| tr("XXMI wurde geöffnet", "XXMI opened").to_string()),
            )
        });
    }

    // ------------------------------------------------------------ layout

    fn rail(&mut self, ui: &mut egui::Ui) {
        let rect = ui.max_rect();
        let p = ui.painter();
        theme::gradient_rect(
            p,
            rect,
            Color32::from_rgb(9, 11, 17),
            Color32::from_rgb(7, 9, 14),
            false,
        );
        p.line_segment(
            [rect.right_top(), rect.right_bottom()],
            Stroke::new(1.0, Color32::from_white_alpha(10)),
        );

        ui.add_space(14.0);
        ui.vertical_centered(|ui| {
            // brand
            let (r, resp) = ui.allocate_exact_size(Vec2::splat(48.0), Sense::click());
            egui::Image::new(egui::include_image!("../assets/gachahub-icon.png"))
                .corner_radius(CornerRadius::same(14))
                .paint_at(ui, r);
            if resp.on_hover_text("GachaHub").clicked() {
                self.catalog_open = true;
            }
            ui.add_space(10.0);
            let (r, _) = ui.allocate_exact_size(Vec2::new(36.0, 1.0), Sense::hover());
            ui.painter()
                .rect_filled(r, 0, Color32::from_white_alpha(14));
            ui.add_space(6.0);

            let tools_height = 4.0 * 52.0 + 30.0;
            let avail = (ui.available_height() - tools_height).max(60.0);
            egui::ScrollArea::vertical()
                .max_height(avail)
                .show(ui, |ui| {
                    ui.vertical_centered(|ui| {
                        let library = self.cfg.library.clone();
                        let mut select = None;
                        let mut remove = None;
                        let mut move_by: Option<(usize, isize)> = None;
                        for (idx, id) in library.iter().enumerate() {
                            let Some(game) = self.game(id).cloned() else {
                                continue;
                            };
                            let (a, b) = Self::accents(&game);
                            let active =
                                self.section == Section::Game && self.cfg.selected_game == *id;
                            ui.add_space(4.0);
                            let (r, resp) =
                                ui.allocate_exact_size(Vec2::new(76.0, 52.0), Sense::click());
                            let hover = ui.ctx().animate_bool(resp.id, resp.hovered());
                            let sel = ui.ctx().animate_bool(resp.id.with("sel"), active);
                            let badge = Rect::from_center_size(
                                r.center() + Vec2::new(hover * 2.0, 0.0),
                                Vec2::splat(48.0),
                            );
                            if let Some(icon) = self.artwork.get(id).and_then(|a| a.icon.clone()) {
                                ui.painter().rect_filled(
                                    badge.expand(3.0 * sel),
                                    CornerRadius::same(17),
                                    alpha(a, 0.25 * sel),
                                );
                                egui::Image::new(icon)
                                    .corner_radius(CornerRadius::same(14))
                                    .paint_at(ui, badge);
                            } else {
                                theme::paint_game_badge(
                                    ui,
                                    badge,
                                    &game.short_name,
                                    a,
                                    b,
                                    active,
                                    hover,
                                );
                            }
                            // active indicator
                            if sel > 0.01 {
                                let h = 30.0 * sel;
                                let ind = Rect::from_center_size(
                                    Pos2::new(r.left() + 2.0, r.center().y),
                                    Vec2::new(4.0, h),
                                );
                                ui.painter().rect_filled(
                                    ind.expand(3.0),
                                    CornerRadius::same(4),
                                    alpha(a, 0.2 * sel),
                                );
                                ui.painter().rect_filled(ind, CornerRadius::same(2), a);
                            }
                            if self.running.contains_key(id) {
                                let c = badge.right_bottom() - Vec2::splat(4.0);
                                ui.painter().circle_filled(c, 6.0, theme::BG);
                                ui.painter().circle_filled(c, 4.0, theme::OK);
                            }
                            let resp = resp
                                .on_hover_text(&game.name)
                                .on_hover_cursor(egui::CursorIcon::PointingHand);
                            if resp.clicked() {
                                select = Some(id.clone());
                            }
                            resp.context_menu(|ui| {
                                if ui.button(tr("⬆  Nach oben", "⬆  Move up")).clicked() {
                                    move_by = Some((idx, -1));
                                    ui.close();
                                }
                                if ui.button(tr("⬇  Nach unten", "⬇  Move down")).clicked() {
                                    move_by = Some((idx, 1));
                                    ui.close();
                                }
                                if ui.button(tr("⚙  Einstellungen", "⚙  Settings")).clicked() {
                                    self.settings_game = id.clone();
                                    self.settings_tab = SettingsTab::Game;
                                    self.section = Section::Settings;
                                    ui.close();
                                }
                                ui.separator();
                                if ui
                                    .add_enabled(
                                        library.len() > 1,
                                        egui::Button::new(tr(
                                            "✖  Aus Bibliothek entfernen",
                                            "✖  Remove from library",
                                        )),
                                    )
                                    .clicked()
                                {
                                    remove = Some(id.clone());
                                    ui.close();
                                }
                            });
                        }
                        ui.add_space(6.0);
                        // add-game button
                        let (r, resp) =
                            ui.allocate_exact_size(Vec2::new(76.0, 52.0), Sense::click());
                        let hover = ui.ctx().animate_bool(resp.id, resp.hovered());
                        let b = Rect::from_center_size(r.center(), Vec2::splat(44.0));
                        ui.painter().rect(
                            b,
                            CornerRadius::same(14),
                            alpha(theme::PANEL, 0.5 + hover * 0.5),
                            Stroke::new(1.0, mix(theme::BORDER, theme::BRAND, hover * 0.6)),
                            StrokeKind::Inside,
                        );
                        ui.painter().text(
                            b.center(),
                            Align2::CENTER_CENTER,
                            "+",
                            theme::font(24.0),
                            mix(theme::MUTED, theme::BRAND, hover),
                        );
                        if resp
                            .on_hover_text(tr("Spiel hinzufügen", "Add game"))
                            .on_hover_cursor(egui::CursorIcon::PointingHand)
                            .clicked()
                        {
                            self.catalog_open = true;
                        }

                        if let Some(id) = select {
                            self.select_game(&id);
                        }
                        if let Some(id) = remove {
                            self.cfg.library.retain(|g| *g != id);
                            if self.cfg.selected_game == id {
                                self.cfg.selected_game = self.cfg.library[0].clone();
                            }
                            self.dirty_since.get_or_insert(0.0);
                        }
                        if let Some((idx, d)) = move_by {
                            let target = idx as isize + d;
                            if target >= 0 && (target as usize) < self.cfg.library.len() {
                                self.cfg.library.swap(idx, target as usize);
                                self.dirty_since.get_or_insert(0.0);
                            }
                        }
                    });
                });
        });

        // bottom tools
        let tools = [
            (Section::Downloads, "⬇", tr("Downloads", "Downloads")),
            (Section::Mods, "🔌", "Mods"),
            (Section::Runners, "🍷", tr("Runner", "Runners")),
            (Section::Settings, "⚙", tr("Einstellungen", "Settings")),
        ];
        let mut y = rect.bottom() - 18.0 - tools.len() as f32 * 52.0;
        let active_jobs = self.active_jobs();
        for (section, icon, label) in tools {
            let r = Rect::from_min_size(Pos2::new(rect.left() + 16.0, y), Vec2::new(44.0, 44.0));
            let resp = ui.interact(r, ui.scope_id().with(label), Sense::click());
            let hover = ui.ctx().animate_bool(resp.id, resp.hovered());
            let active = self.section == section;
            let p = ui.painter();
            if active || hover > 0.0 {
                p.rect(
                    r,
                    CornerRadius::same(13),
                    if active {
                        alpha(theme::BRAND, 0.12)
                    } else {
                        alpha(theme::PANEL_HOVER, hover)
                    },
                    Stroke::new(
                        1.0,
                        if active {
                            alpha(theme::BRAND, 0.3)
                        } else {
                            Color32::TRANSPARENT
                        },
                    ),
                    StrokeKind::Inside,
                );
            }
            let color = if active {
                theme::BRAND
            } else {
                mix(theme::MUTED, theme::TEXT, hover)
            };
            p.text(
                r.center(),
                Align2::CENTER_CENTER,
                icon,
                theme::font(19.0),
                color,
            );
            if section == Section::Downloads && active_jobs > 0 {
                let c = r.right_top() + Vec2::new(-4.0, 5.0);
                p.circle_filled(c, 8.0, theme::BRAND);
                p.text(
                    c,
                    Align2::CENTER_CENTER,
                    active_jobs.to_string(),
                    theme::font_heavy(10.0),
                    theme::BG,
                );
            }
            if resp
                .on_hover_text(label)
                .on_hover_cursor(egui::CursorIcon::PointingHand)
                .clicked()
            {
                self.section = section;
                if section == Section::Mods {
                    if !self.cfg.library.contains(&self.mods_game) {
                        self.mods_game = self.cfg.selected_game.clone();
                    }
                    let id = self.mods_game.clone();
                    self.refresh_mods(&id);
                }
            }
            y += 52.0;
        }
    }

    fn toasts_ui(&mut self, ctx: &egui::Context) {
        let now = ctx.input(|i| i.time);
        for t in &mut self.toasts {
            if t.born < 0.0 {
                t.born = now;
            }
        }
        self.toasts
            .retain(|t| now - t.born < if t.kind == ToastKind::Error { 9.0 } else { 4.5 });
        if self.toasts.is_empty() {
            return;
        }
        ctx.request_repaint_after(std::time::Duration::from_millis(250));
        egui::Area::new(egui::Id::unique("gachahub-toasts"))
            .anchor(Align2::RIGHT_BOTTOM, Vec2::new(-22.0, -22.0))
            .order(egui::Order::Foreground)
            .show(ctx, |ui| {
                ui.set_max_width(380.0);
                let mut dismiss = None;
                for (i, t) in self.toasts.iter().enumerate() {
                    let color = match t.kind {
                        ToastKind::Info => theme::BRAND,
                        ToastKind::Success => theme::OK,
                        ToastKind::Error => theme::ERR,
                    };
                    let age = (now - t.born) as f32;
                    let a = (age * 6.0).min(1.0);
                    let resp = egui::Frame::new()
                        .fill(Color32::from_rgba_unmultiplied(
                            14,
                            18,
                            27,
                            (240.0 * a) as u8,
                        ))
                        .stroke(Stroke::new(1.0, alpha(color, 0.35 * a)))
                        .corner_radius(CornerRadius::same(12))
                        .inner_margin(Margin::symmetric(14, 11))
                        .shadow(egui::Shadow {
                            offset: [0, 8],
                            blur: 24,
                            spread: 0,
                            color: Color32::from_black_alpha(120),
                        })
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                let (r, _) =
                                    ui.allocate_exact_size(Vec2::new(4.0, 18.0), Sense::hover());
                                ui.painter().rect_filled(r, CornerRadius::same(2), color);
                                ui.add(
                                    egui::Label::new(
                                        egui::RichText::new(&t.text)
                                            .color(theme::TEXT)
                                            .font(theme::font(13.0)),
                                    )
                                    .wrap(),
                                );
                            });
                        })
                        .response
                        .interact(Sense::click());
                    if resp.clicked() {
                        dismiss = Some(i);
                    }
                    ui.add_space(6.0);
                }
                if let Some(i) = dismiss {
                    self.toasts.remove(i);
                }
            });
    }
}

/// Look for a plausible game executable inside an install folder.
pub fn find_executable(root: &Path, game_id: &str) -> Option<PathBuf> {
    let preferred: &[&str] = match game_id {
        "genshin" => &["GenshinImpact.exe", "YuanShen.exe"],
        "hsr" => &["StarRail.exe"],
        "zzz" => &["ZenlessZoneZero.exe"],
        "wuwa" => &["Wuthering Waves.exe", "Client-Win64-Shipping.exe"],
        "endfield" => &["Endfield.exe"],
        "arknights" => &["Arknights.exe"],
        "gfl2" => &["GF2_Exilium.exe"],
        _ => &[],
    };
    let mut fallback = None;
    let mut stack = vec![(root.to_path_buf(), 0usize)];
    while let Some((dir, depth)) = stack.pop() {
        let Ok(entries) = std::fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                if depth < 4 {
                    stack.push((path, depth + 1));
                }
                continue;
            }
            let name = entry.file_name().to_string_lossy().into_owned();
            if preferred.iter().any(|p| p.eq_ignore_ascii_case(&name)) {
                return Some(path);
            }
            let lower = name.to_ascii_lowercase();
            if fallback.is_none()
                && lower.ends_with(".exe")
                && ![
                    "unins",
                    "crash",
                    "launcher",
                    "setup",
                    "update",
                    "vc_redist",
                    "dxsetup",
                ]
                .iter()
                .any(|b| lower.contains(b))
            {
                fallback = Some(path);
            }
        }
    }
    fallback
}

impl eframe::App for GachaHubApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        self.process_messages(&ctx);

        egui::Panel::left("rail")
            .exact_size(76.0)
            .resizable(false)
            .show_separator_line(false)
            .frame(egui::Frame::new())
            .show(ui, |ui| self.rail(ui));

        egui::CentralPanel::no_frame().show(ui, |ui| match self.section {
            Section::Game => self.page_game(ui),
            Section::Downloads => self.page_downloads(ui),
            Section::Mods => self.page_mods(ui),
            Section::Runners => self.page_runners(ui),
            Section::Settings => self.page_settings(ui),
        });

        if self.catalog_open {
            self.catalog_modal(&ctx);
        }
        self.toasts_ui(&ctx);

        if !self.running.is_empty()
            || !self.jobs.is_empty()
            || self.busy.iter().any(|b| b.starts_with("xxmi-install"))
        {
            ctx.request_repaint_after(std::time::Duration::from_millis(500));
        }

        // Debounced config persistence.
        let now = ctx.input(|i| i.time);
        if let Some(since) = self.dirty_since {
            if now - since > 0.8 {
                if let Err(e) = self.cfg.save() {
                    self.toast(ToastKind::Error, e);
                }
                self.dirty_since = None;
            } else {
                ctx.request_repaint_after(std::time::Duration::from_millis(900));
            }
        }
        set_language(self.cfg.global.language);
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        let _ = self.cfg.save();
    }
}
