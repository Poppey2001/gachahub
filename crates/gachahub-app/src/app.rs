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
    download_manager::{DownloadOptions, JobKind},
    install::InstallJob,
    mod_manager::ModSnapshot,
    runners::{RunnerInfo, RunnerProgress, RunnerVersion},
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
    /// User-supplied icon (game-assets folder) – highest priority.
    pub icon: Option<String>,
    pub logo: Option<String>,
    /// Icon extracted from the installed game executable.
    pub exe_icon: Option<String>,
}

/// GachaHub's own emblem for each supported game (original artwork, see assets/game-icons).
fn bundled_icon(game_id: &str) -> Option<&'static [u8]> {
    Some(match game_id {
        "arknights" => include_bytes!("../../../assets/game-icons/arknights.svg"),
        "endfield" => include_bytes!("../../../assets/game-icons/endfield.svg"),
        "genshin" => include_bytes!("../../../assets/game-icons/genshin.svg"),
        "gfl2" => include_bytes!("../../../assets/game-icons/gfl2.svg"),
        "hi3" => include_bytes!("../../../assets/game-icons/hi3.svg"),
        "bluearchive" => include_bytes!("../../../assets/game-icons/bluearchive.svg"),
        "limbus" => include_bytes!("../../../assets/game-icons/limbus.svg"),
        "pgr" => include_bytes!("../../../assets/game-icons/pgr.svg"),
        "snowbreak" => include_bytes!("../../../assets/game-icons/snowbreak.svg"),
        "reverse1999" => include_bytes!("../../../assets/game-icons/reverse1999.svg"),
        "tof" => include_bytes!("../../../assets/game-icons/tof.svg"),
        "hsr" => include_bytes!("../../../assets/game-icons/hsr.svg"),
        "nte" => include_bytes!("../../../assets/game-icons/nte.svg"),
        "wuwa" => include_bytes!("../../../assets/game-icons/wuwa.svg"),
        "zzz" => include_bytes!("../../../assets/game-icons/zzz.svg"),
        _ => return None,
    })
}

fn exe_icon_cache(game_id: &str, exe: &str) -> PathBuf {
    use std::hash::{Hash, Hasher};
    let mut h = std::collections::hash_map::DefaultHasher::new();
    exe.hash(&mut h);
    if let Ok(meta) = std::fs::metadata(exe) {
        meta.len().hash(&mut h);
    }
    directories::ProjectDirs::from("", "", "gachahub")
        .map(|d| d.cache_dir().to_path_buf())
        .unwrap_or_else(std::env::temp_dir)
        .join("exe-icons")
        .join(format!("{game_id}-{:016x}.png", h.finish()))
}

fn file_uri(path: &Path) -> String {
    format!("file://{}", path.display())
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
    /// Executable path whose icon was last requested, per game.
    exe_icon_source: HashMap<String, String>,
    /// Installed game version per game (read from disk).
    pub installed_versions: HashMap<String, String>,
    /// Latest version reported by the provider.
    pub latest_versions: HashMap<String, String>,
    /// Games with an interrupted installation that can be resumed.
    pub partial_installs: HashSet<String>,
    /// Running runner installations by tag.
    pub runner_progress: HashMap<String, RunnerProgress>,

    /// Newer GachaHub release, if one was found.
    pub app_update: Option<gachahub_core::self_update::UpdateInfo>,
    /// Self-update download progress (downloaded, total).
    pub app_update_progress: Option<(u64, u64)>,
    /// Update installed – restart to activate it.
    pub app_update_ready: bool,
    /// Last self-update error / "up to date" note for the settings page.
    pub app_update_status: Option<String>,
    /// Running game sessions (for "stop game").
    pub sessions: HashMap<String, std::sync::Arc<gachahub_core::session::GameSession>>,
    /// Game whose uninstall confirmation dialog is open.
    pub uninstall_confirm: Option<String>,
    /// Text field for new mod profile names.
    pub mod_profile_input: String,
    /// Installed Steam games (local scan).
    pub steam_apps: Vec<gachahub_core::steam::SteamApp>,
    /// Re-scan Steam periodically after "Install in Steam" until the game shows up.
    steam_watch_until: f64,
    last_steam_scan: f64,
    /// Game whose edition picker (official launchers) is open.
    pub edition_picker: Option<String>,
    /// Official installer downloads in flight: game → (downloaded, total).
    pub installer_progress: HashMap<String, (u64, u64)>,
    /// After running an official installer: game → watch-until time.
    pub official_watch: HashMap<String, f64>,
    pub last_official_scan: f64,
    /// Search text in the game catalog.
    pub catalog_filter: String,
    /// egui time of the current frame.
    pub now: f64,
    /// Path of the running executable, captured before any self-update.
    startup_exe: Option<PathBuf>,
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
                // (canonicalize would add a `\\?\` prefix on Windows)
                let path = if cfg!(windows) {
                    path
                } else {
                    path.canonicalize().unwrap_or(path)
                };
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
                        exe_icon: None,
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
            exe_icon_source: HashMap::new(),
            installed_versions: HashMap::new(),
            latest_versions: HashMap::new(),
            partial_installs: HashSet::new(),
            runner_progress: HashMap::new(),
            app_update: None,
            app_update_progress: None,
            app_update_ready: false,
            app_update_status: None,
            startup_exe: std::env::current_exe().ok(),
            sessions: HashMap::new(),
            uninstall_confirm: None,
            mod_profile_input: String::new(),
            steam_apps: Vec::new(),
            steam_watch_until: 0.0,
            last_steam_scan: 0.0,
            edition_picker: None,
            installer_progress: HashMap::new(),
            official_watch: HashMap::new(),
            last_official_scan: 0.0,
            catalog_filter: String::new(),
            now: 0.0,
        };
        // Developer aid: open a specific page on start (used for screenshots/CI).
        if let Ok(page) = std::env::var("GACHAHUB_START_PAGE") {
            app.section = match page.as_str() {
                "downloads" => Section::Downloads,
                "mods" => Section::Mods,
                "runners" => Section::Runners,
                "settings" => Section::Settings,
                "editions" => {
                    app.edition_picker = Some("gfl2".into());
                    app.cfg.selected_game = "gfl2".into();
                    Section::Game
                }
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
        app.refresh_installed_versions();
        app.check_updates();
        if app.cfg.global.app_update_check {
            app.check_app_update(false);
        }
        app.refresh_steam();
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

    /// Schedule a config save on the next frame.
    pub fn dirty_now(&mut self) {
        self.dirty_since.get_or_insert(0.0);
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
        let (install, custom) = (g.xxmi_install_path.clone(), g.xxmi_launcher_path.clone());
        self.busy.insert("xxmi-status".into());
        self.backend.spawn(async move {
            Msg::Xxmi(gachahub_core::xxmi::manager_status(&install, &custom).await)
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

    /// Paint the best available icon for a game: custom artwork, the icon of the
    /// installed executable, GachaHub's bundled emblem, or a generated badge.
    pub fn paint_game_icon(
        &self,
        ui: &egui::Ui,
        rect: Rect,
        game: &Game,
        active: bool,
        hover: f32,
    ) {
        let (a, b) = Self::accents(game);
        let radius = CornerRadius::same((rect.width() * 0.26) as u8);
        if active {
            ui.painter().rect_filled(
                rect.expand(3.5),
                CornerRadius::same((rect.width() * 0.26 + 3.5) as u8),
                alpha(a, 0.28),
            );
        }
        let art = self.artwork.get(&game.id);
        let image =
            if let Some(uri) = art.and_then(|x| x.icon.clone().or_else(|| x.exe_icon.clone())) {
                Some(egui::Image::new(uri))
            } else {
                bundled_icon(&game.id).map(|bytes| {
                    egui::Image::from_bytes(format!("bytes://game-icons/{}.svg", game.id), bytes)
                })
            };
        match image {
            Some(image) => {
                let level = if active { 255.0 } else { 175.0 + hover * 80.0 };
                image
                    .corner_radius(radius)
                    .tint(Color32::from_gray(level as u8))
                    .paint_at(ui, rect);
                if active {
                    ui.painter().rect_stroke(
                        rect,
                        radius,
                        Stroke::new(1.5, alpha(a, 0.8)),
                        StrokeKind::Inside,
                    );
                }
            }
            None => theme::paint_game_badge(ui, rect, &game.short_name, a, b, active, hover),
        }
    }

    /// Extract icons from installed game executables (once per executable).
    fn refresh_exe_icons(&mut self) {
        let wanted: Vec<(String, String)> = self
            .cfg
            .games
            .iter()
            .filter_map(|(id, gs)| {
                gs.executable_path
                    .clone()
                    .filter(|e| !e.is_empty())
                    .map(|e| (id.clone(), e))
            })
            .filter(|(id, exe)| self.exe_icon_source.get(id) != Some(exe))
            .collect();
        for (id, exe) in wanted {
            self.exe_icon_source.insert(id.clone(), exe.clone());
            let cache = exe_icon_cache(&id, &exe);
            if cache.is_file() {
                if let Some(art) = self.artwork.get_mut(&id) {
                    art.exe_icon = Some(file_uri(&cache));
                }
                continue;
            }
            self.backend.blocking(move || {
                let exe_path = gachahub_core::expand_user(&exe);
                let res = gachahub_core::exe_icon::extract_png(&exe_path, &cache).map(|_| cache);
                Msg::ExeIcon(id, res)
            });
        }
    }

    // ------------------------------------------------------------ Steam

    pub fn refresh_steam(&mut self) {
        self.backend
            .blocking(|| Msg::SteamApps(gachahub_core::steam::detect()));
    }

    /// The Steam installation that belongs to a catalog game, if any.
    pub fn steam_app_for(&self, game: &Game) -> Option<gachahub_core::steam::SteamApp> {
        let names: Vec<&str> = game.steam_names.iter().map(String::as_str).collect();
        gachahub_core::steam::find_app(&self.steam_apps, &names).cloned()
    }

    /// Link Steam installs to catalog games (paths, executable, launch through Steam).
    fn link_steam_games(&mut self) {
        let mut found = Vec::new();
        for game in self.games.clone() {
            let Some(app) = self.steam_app_for(&game) else {
                continue;
            };
            let gs = self.cfg.games.get(&game.id);
            let has_other_install =
                gs.is_some_and(|g| g.executable_path.is_some() && g.steam_app_id.is_none());
            let already = gs.and_then(|g| g.steam_app_id.as_deref()) == Some(app.app_id.as_str());
            if has_other_install || already {
                continue;
            }
            let exe = find_executable(&app.install_dir, &game.id);
            let gs = self.cfg.game_mut(&game.id);
            gs.steam_app_id = Some(app.app_id.clone());
            gs.install_path = Some(app.install_dir.to_string_lossy().into_owned());
            gs.executable_path = exe.map(|e| e.to_string_lossy().into_owned());
            gs.installation_detected_at = Some(now_unix().to_string());
            if !self.cfg.library.contains(&game.id) {
                self.cfg.library.push(game.id.clone());
            }
            self.game_errors.remove(&game.id);
            found.push(game.name.clone());
        }
        if !found.is_empty() {
            self.toast(
                ToastKind::Success,
                format!("Steam: {} {}", found.join(", "), tr("gefunden", "found")),
            );
            self.dirty_since.get_or_insert(0.0);
            self.refresh_installed_versions();
        }
    }

    pub fn is_steam_game(&self, game_id: &str) -> bool {
        self.cfg
            .games
            .get(game_id)
            .is_some_and(|g| g.steam_app_id.is_some())
    }

    /// Open the game in the Steam client to install it there.
    pub fn install_via_steam(&mut self, game: &Game, now: f64) {
        let name = game
            .steam_names
            .first()
            .cloned()
            .unwrap_or_else(|| game.name.clone());
        let term: String = name
            .chars()
            .map(|c| {
                if c.is_ascii_alphanumeric() {
                    c.to_string()
                } else if c == ' ' {
                    "+".into()
                } else {
                    format!("%{:02X}", c as u32 & 0xff)
                }
            })
            .collect();
        let url = format!("steam://openurl/https://store.steampowered.com/search/?term={term}");
        match open::that_detached(&url) {
            Ok(()) => {
                self.toast(
                    ToastKind::Info,
                    tr(
                        "Steam wurde geöffnet – nach der Installation erkennt GachaHub das Spiel automatisch.",
                        "Steam opened – GachaHub detects the game automatically once it is installed.",
                    ),
                );
                // keep looking for the new installation for 2 hours
                self.steam_watch_until = now + 2.0 * 3600.0;
            }
            Err(e) => self.toast(
                ToastKind::Error,
                format!(
                    "{}: {e}",
                    tr(
                        "Steam konnte nicht geöffnet werden – ist Steam installiert?",
                        "Could not open Steam – is it installed?"
                    )
                ),
            ),
        }
    }

    pub fn stop_game(&mut self, game_id: &str) {
        if let Some(session) = self.sessions.get(game_id).cloned() {
            self.backend.blocking(move || {
                session.kill();
                Msg::Notice(Ok(tr("Spiel beendet", "Game stopped").into()))
            });
        }
    }

    pub fn open_game_folder(&mut self, game_id: &str) {
        let r = self.cfg.resolved(game_id);
        let dir = if !r.executable_path.is_empty() {
            gachahub_core::expand_user(&r.executable_path)
                .parent()
                .map(Path::to_path_buf)
        } else {
            Some(gachahub_core::expand_user(&r.install_path))
        };
        match dir.filter(|d| d.is_dir()) {
            Some(d) => {
                if let Err(e) = open::that_detached(&d) {
                    self.toast(ToastKind::Error, e.to_string());
                }
            }
            None => self.toast(
                ToastKind::Error,
                tr(
                    "Spielordner existiert nicht.",
                    "Game folder does not exist.",
                ),
            ),
        }
    }

    /// Can GachaHub safely delete this game's files? Only folders it installed
    /// itself (version marker present) – never arbitrary user folders.
    pub fn can_delete_files(&self, game_id: &str) -> bool {
        let dir = gachahub_core::expand_user(&self.cfg.resolved(game_id).install_path);
        dir.join(gachahub_core::download_manager::VERSION_FILE)
            .is_file()
    }

    /// Remove a game: forget its paths and, if `delete_files`, delete the install folder.
    pub fn uninstall_game(&mut self, game_id: &str, delete_files: bool) {
        let r = self.cfg.resolved(game_id);
        let gs = self.cfg.game_mut(game_id);
        let playtime = gs.playtime_seconds;
        *gs = Default::default();
        gs.playtime_seconds = playtime;
        self.dirty_since.get_or_insert(0.0);
        self.installed_versions.remove(game_id);
        self.latest_versions.remove(game_id);
        self.partial_installs.remove(game_id);
        let name = self
            .game(game_id)
            .map(|g| g.name.clone())
            .unwrap_or_default();
        if delete_files && self.can_delete_files(game_id) {
            let dir = gachahub_core::expand_user(&r.install_path);
            self.backend.blocking(move || {
                Msg::Notice(
                    std::fs::remove_dir_all(&dir)
                        .map(|_| format!("{name}: {}", tr("deinstalliert", "uninstalled")))
                        .map_err(|e| format!("{}: {e}", dir.display())),
                )
            });
        } else {
            self.toast(
                ToastKind::Info,
                format!(
                    "{name}: {}",
                    tr("aus GachaHub entfernt", "removed from GachaHub")
                ),
            );
        }
    }

    // ------------------------------------------------------------ GachaHub self-update

    pub fn check_app_update(&mut self, manual: bool) {
        let prerelease =
            self.cfg.global.app_update_channel == gachahub_core::settings::AppChannel::Prerelease;
        self.busy.insert("app-update-check".into());
        self.backend.spawn(async move {
            Msg::AppUpdateChecked(gachahub_core::self_update::check(prerelease).await, manual)
        });
    }

    pub fn install_app_update(&mut self) {
        let Some(info) = self.app_update.clone() else {
            return;
        };
        let sink = self.sink.clone();
        self.busy.insert("app-update-install".into());
        self.app_update_progress = Some((0, info.asset_size));
        self.app_update_status = None;
        self.backend.spawn(async move {
            Msg::AppUpdateInstalled(gachahub_core::self_update::install(&info, sink).await)
        });
    }

    pub fn restart_app(&mut self) {
        let _ = self.cfg.save();
        let exe = self
            .startup_exe
            .clone()
            .or_else(|| std::env::current_exe().ok());
        match exe {
            Some(exe) => {
                if let Err(e) = gachahub_core::self_update::restart(&exe) {
                    self.toast(ToastKind::Error, e);
                }
            }
            None => self.toast(
                ToastKind::Error,
                tr(
                    "Pfad der App unbekannt – bitte manuell neu starten.",
                    "Unknown app path – please restart manually.",
                ),
            ),
        }
    }

    /// A download/update/repair job completed: pick up the new installation.
    fn on_job_finished(&mut self, job: &InstallJob) {
        let game_id = job.game_id.clone();
        let name = self
            .game(&game_id)
            .map(|g| g.name.clone())
            .unwrap_or_default();
        let text = match job.kind.as_str() {
            "update" => tr("Update installiert", "update installed"),
            "verify" => tr(
                "Dateien geprüft und repariert",
                "files verified and repaired",
            ),
            _ => tr("Installation abgeschlossen", "installation finished"),
        };
        self.toast(ToastKind::Success, format!("{name}: {text}"));
        if let Some(v) = &job.version {
            self.latest_versions.insert(game_id.clone(), v.clone());
        }
        if self.cfg.resolved(&game_id).executable_path.is_empty() {
            if let Some(dest) = job.destination.as_deref() {
                if game_id == "genshin" {
                    self.import_existing(&game_id, Path::new(dest));
                } else if let Some(exe) = find_executable(Path::new(dest), &game_id) {
                    let gs = self.cfg.game_mut(&game_id);
                    gs.executable_path = Some(exe.to_string_lossy().into_owned());
                    gs.installation_detected_at = Some(now_unix().to_string());
                }
            }
        }
        self.dirty_since.get_or_insert(0.0);
        self.refresh_installed_versions();
    }

    // ------------------------------------------------------------ messages

    fn process_messages(&mut self, ctx: &egui::Context) {
        for msg in self.backend.drain() {
            match msg {
                Msg::Core(CoreEvent::Download(job)) => {
                    let finished = job.phase == "ready"
                        && self
                            .jobs
                            .iter()
                            .any(|j| j.id == job.id && j.phase != "ready");
                    if finished {
                        self.on_job_finished(&job);
                    }
                    self.upsert_job(job);
                }
                Msg::Core(CoreEvent::Runner(progress)) => {
                    if matches!(progress.phase.as_str(), "ready" | "error") {
                        self.runner_progress.remove(&progress.tag_name);
                    } else {
                        self.runner_progress
                            .insert(progress.tag_name.clone(), progress);
                    }
                }
                Msg::Core(CoreEvent::SelfUpdate(done, total)) => {
                    self.app_update_progress = Some((done, total));
                }
                Msg::AppUpdateChecked(res, manual) => {
                    self.busy.remove("app-update-check");
                    match res {
                        Ok(Some(info)) => {
                            if self.app_update.as_ref().map(|u| &u.version) != Some(&info.version) {
                                self.toast(
                                    ToastKind::Info,
                                    format!(
                                        "GachaHub {} {}",
                                        info.version,
                                        tr("ist verfügbar", "is available")
                                    ),
                                );
                            }
                            self.app_update_status = None;
                            self.app_update = Some(info);
                        }
                        Ok(None) => {
                            self.app_update = None;
                            self.app_update_status =
                                Some(tr("GachaHub ist aktuell.", "GachaHub is up to date.").into());
                            if manual {
                                self.toast(
                                    ToastKind::Success,
                                    tr("GachaHub ist aktuell.", "GachaHub is up to date."),
                                );
                            }
                        }
                        Err(e) => {
                            self.app_update_status = Some(e.clone());
                            if manual {
                                self.toast(ToastKind::Error, e);
                            }
                        }
                    }
                }
                Msg::AppUpdateInstalled(res) => {
                    self.busy.remove("app-update-install");
                    self.app_update_progress = None;
                    match res {
                        Ok(()) => {
                            self.app_update_ready = true;
                            self.toast(
                                ToastKind::Success,
                                tr(
                                    "Update installiert – Neustart aktiviert die neue Version.",
                                    "Update installed – restart to activate it.",
                                ),
                            );
                        }
                        Err(e) => {
                            self.app_update_status = Some(e.clone());
                            self.toast(ToastKind::Error, e);
                        }
                    }
                }
                Msg::LatestVersion(game_id, res) => {
                    if let Ok(latest) = res {
                        self.latest_versions.insert(game_id.clone(), latest);
                        let busy = self.job_for(&game_id).is_some_and(|j| {
                            !matches!(j.phase.as_str(), "ready" | "error" | "cancelled")
                        });
                        if let Some(version) = self.update_available(&game_id) {
                            if self.cfg.resolved(&game_id).auto_update && !busy {
                                let name = self
                                    .game(&game_id)
                                    .map(|g| g.name.clone())
                                    .unwrap_or_default();
                                self.toast(
                                    ToastKind::Info,
                                    format!(
                                        "{name}: {} {version}",
                                        tr("Update wird installiert:", "installing update")
                                    ),
                                );
                                self.start_job(&game_id, JobKind::Update);
                            }
                        }
                    }
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
                Msg::InstallerProgress(game_id, done, total) => {
                    self.installer_progress.insert(game_id, (done, total));
                }
                Msg::InstallerReady(game_id, edition, res) => {
                    self.installer_progress.remove(&game_id);
                    self.on_installer_ready(&game_id, &edition, res);
                }
                Msg::OfficialDetected(game_id, found) => {
                    if let Some((dir, exe)) = found {
                        self.on_official_detected(&game_id, dir, exe);
                    }
                }
                Msg::SteamApps(apps) => {
                    self.steam_apps = apps;
                    self.link_steam_games();
                }
                Msg::SessionStarted(game_id, session) => {
                    self.sessions.insert(game_id, session);
                }
                Msg::SessionEnded(game_id, secs) => {
                    self.running.remove(&game_id);
                    self.sessions.remove(&game_id);
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
                Msg::ExeIcon(game_id, res) => {
                    // Not every executable carries an icon – the bundled emblem stays then.
                    if let (Ok(path), Some(art)) = (res, self.artwork.get_mut(&game_id)) {
                        let uri = file_uri(&path);
                        ctx.forget_image(&uri);
                        art.exe_icon = Some(uri);
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
            PickTarget::ImportExisting(id) => {
                self.import_existing(&id, &path);
                self.refresh_installed_versions();
                self.check_updates();
            }
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
            PickTarget::LegacyGmmConfig(id) => {
                match gachahub_core::mod_manager::read_legacy_config(&s) {
                    Ok(cfg) => {
                        let gs = self.cfg.game_mut(&id);
                        let mut found = false;
                        if let Some(lib) = cfg.library_path.filter(|p| !p.is_empty()) {
                            gs.mod_library_path = Some(lib);
                            found = true;
                        }
                        if let Some(active) = cfg.active_mods_path.filter(|p| !p.is_empty()) {
                            gs.active_mods_path = Some(active);
                            found = true;
                        }
                        if found {
                            self.toast(
                                ToastKind::Success,
                                tr(
                                    "Pfade aus GIMI Mod Manager übernommen",
                                    "Paths imported from GIMI Mod Manager",
                                ),
                            );
                            self.refresh_mods(&id);
                        } else {
                            self.toast(
                                ToastKind::Error,
                                tr(
                                    "In der Datei wurden keine Mod-Pfade gefunden.",
                                    "No mod paths found in that file.",
                                ),
                            );
                        }
                    }
                    Err(e) => self.toast(ToastKind::Error, e),
                }
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
    pub(crate) fn import_existing(&mut self, game_id: &str, folder: &Path) {
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
        let needs_edition = self
            .game(game_id)
            .is_some_and(|g| !g.official_launchers.is_empty())
            && self
                .cfg
                .games
                .get(game_id)
                .and_then(|g| g.edition.as_ref())
                .is_none();
        if needs_edition {
            self.edition_picker = Some(game_id.to_string());
            return;
        }
        self.start_job(game_id, JobKind::Install);
    }

    /// Install, update or verify a game (see `download_manager`).
    pub fn start_job(&mut self, game_id: &str, kind: JobKind) {
        let install = self.cfg.resolved(game_id).install_path;
        let g = &self.cfg.global;
        let opts = DownloadOptions {
            kind,
            parallel: g.parallel_downloads.max(1) as usize,
            bandwidth_limit_mbps: g.bandwidth_limit_mbps,
            voice_language: g.voice_language.clone(),
            edition: self.cfg.games.get(game_id).and_then(|g| g.edition.clone()),
        };
        let sink = self.sink.clone();
        let gid = game_id.to_string();
        self.busy.insert(format!("install:{game_id}"));
        self.game_errors.remove(game_id);
        self.backend.spawn(async move {
            let res =
                gachahub_core::download_manager::start(sink, gid.clone(), install, opts).await;
            Msg::DownloadStarted(gid, res)
        });
    }

    /// Re-read installed versions (marker file / HoYoPlay config.ini) of all library games.
    pub fn refresh_installed_versions(&mut self) {
        for id in self.cfg.library.clone() {
            let r = self.cfg.resolved(&id);
            let dir = gachahub_core::expand_user(&r.install_path);
            match gachahub_core::download_manager::installed_version(&r.install_path) {
                Some(v) => {
                    self.installed_versions.insert(id.clone(), v);
                }
                None => {
                    self.installed_versions.remove(&id);
                }
            }
            let partial = dir.join(".gachahub-downloads").exists()
                || dir.join(".gachahub-files.json").exists();
            if partial && !self.installed_versions.contains_key(&id) {
                self.partial_installs.insert(id);
            } else {
                self.partial_installs.remove(&id);
            }
        }
    }

    /// Ask the provider for the newest version of every installed, updatable game.
    pub fn check_updates(&mut self) {
        for id in self.cfg.library.clone() {
            if !gachahub_core::download_manager::supports_direct_download(&id)
                || !self.installed_versions.contains_key(&id)
            {
                continue;
            }
            let gid = id.clone();
            let edition = self.cfg.games.get(&id).and_then(|g| g.edition.clone());
            self.backend.spawn(async move {
                let res =
                    gachahub_core::download_manager::latest_version_for(&gid, edition.as_deref())
                        .await;
                Msg::LatestVersion(gid, res)
            });
        }
    }

    /// Installed and latest version differ → update available.
    pub fn update_available(&self, game_id: &str) -> Option<String> {
        let installed = self.installed_versions.get(game_id)?;
        let latest = self.latest_versions.get(game_id)?;
        gachahub_core::download_manager::is_newer(latest, installed).then(|| latest.clone())
    }

    /// Start a Windows program for a game with the game's runtime settings
    /// (native on Windows, Proton/Wine with the game's prefix on Linux).
    pub(crate) fn spawn_for_game(
        game_id: &str,
        r: &gachahub_core::settings::ResolvedGameSettings,
        exe: &str,
        args: Vec<String>,
    ) -> Result<(gachahub_core::launcher::LaunchResult, std::process::Child), String> {
        gachahub_core::launcher::launch_exe(
            game_id,
            exe,
            r.runtime.as_str(),
            &r.runner_version,
            &r.runner_path,
            &r.prefix_path,
            args,
            r.environment_variables.clone(),
            r.gamescope_enabled,
            r.game_mode_enabled,
            r.mango_hud_enabled,
            r.prevent_sleep,
        )
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
        if let Some(app_id) = self
            .cfg
            .games
            .get(game_id)
            .and_then(|g| g.steam_app_id.clone())
        {
            match open::that_detached(gachahub_core::steam::launch_url(&app_id)) {
                Ok(()) => {
                    crate::backend::track_detached(
                        self.backend.sender(),
                        game_id.to_string(),
                        &r.executable_path,
                    );
                    self.running.insert(game_id.to_string(), self.now);
                    self.cfg.game_mut(game_id).last_played_unix = Some(now_unix());
                    self.dirty_since.get_or_insert(0.0);
                    self.toast(
                        ToastKind::Success,
                        tr("Start über Steam…", "Launching through Steam…"),
                    );
                }
                Err(e) => {
                    self.game_errors
                        .insert(game_id.to_string(), format!("Steam: {e}"));
                }
            }
            return;
        }
        let xxmi_importer = game.xxmi_importer.clone().filter(|_| r.xxmi_enabled);
        self.busy.insert(format!("launch:{game_id}"));
        self.backend.blocking(move || {
            let res = match xxmi_importer {
                // XXMI starts the game itself; it runs in the game's runner + prefix so
                // the injected importer and the game share one Wine environment.
                Some(importer) => gachahub_core::xxmi::resolve_launcher(
                    &g.xxmi_install_path,
                    &g.xxmi_launcher_path,
                )
                .and_then(|launcher| {
                    let args = gachahub_core::xxmi::launch_args(&importer, g.xxmi_no_gui)?;
                    Self::spawn_for_game(&gid, &r, &launcher.to_string_lossy(), args)
                }),
                None => {
                    let mut args = gachahub_core::launcher::split_args(&r.launch_arguments);
                    args.retain(|a| !a.is_empty());
                    // GachaHub-installed Kuro games need the resource quality the official launcher passes
                    if gachahub_core::kuro::supports(&gid)
                        && !args.iter().any(|a| a.starts_with("-krqlv"))
                        && gachahub_core::download_manager::installed_version(&r.install_path)
                            .is_some()
                    {
                        args.extend(gachahub_core::kuro::launch_args(&gid));
                    }
                    Self::spawn_for_game(&gid, &r, &r.executable_path, args)
                }
            };
            match res {
                Ok((result, child)) => {
                    crate::backend::track_session(sender, gid.clone(), child, &r.executable_path);
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

    /// Launch the XXMI GUI (first-time importer setup) in the game's environment.
    pub fn open_xxmi_gui(&mut self, game_id: &str) {
        let Some(importer) = self.game(game_id).and_then(|g| g.xxmi_importer.clone()) else {
            return;
        };
        let g = self.cfg.global.clone();
        let r = self.cfg.resolved(game_id);
        let gid = game_id.to_string();
        self.backend.blocking(move || {
            let res =
                gachahub_core::xxmi::resolve_launcher(&g.xxmi_install_path, &g.xxmi_launcher_path)
                    .and_then(|launcher| {
                        let args = gachahub_core::xxmi::launch_args(&importer, false)?;
                        Self::spawn_for_game(&gid, &r, &launcher.to_string_lossy(), args)
                    });
            Msg::Notice(res.map(|_| tr("XXMI wurde geöffnet", "XXMI opened").to_string()))
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
                            let (a, _) = Self::accents(&game);
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
                            self.paint_game_icon(ui, badge, &game, active, hover);
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

pub use gachahub_core::install::find_executable;

impl eframe::App for GachaHubApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        self.now = ctx.input(|i| i.time);
        self.process_messages(&ctx);
        // After "install in Steam": re-scan the Steam libraries every 20 s.
        if self.now < self.steam_watch_until && self.now - self.last_steam_scan > 20.0 {
            self.last_steam_scan = self.now;
            self.refresh_steam();
        }
        self.official_watch_tick();
        if self.steam_watch_until > self.now || !self.official_watch.is_empty() {
            ctx.request_repaint_after(std::time::Duration::from_secs(5));
        }
        self.refresh_exe_icons();

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
        if self.edition_picker.is_some() {
            self.edition_modal(&ctx);
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
