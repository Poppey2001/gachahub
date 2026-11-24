//! Bridge between the immediate-mode UI thread and async core work.
//!
//! The UI never blocks: every slow operation (network, disk, process spawn,
//! file dialogs) runs on a Tokio runtime and reports back through a channel
//! that the UI drains once per frame.

use eframe::egui;
use gachahub_core::{
    install::InstallJob,
    launcher::LaunchResult,
    mod_manager::ModSnapshot,
    postinstall::PostInstallResult,
    runners::{RunnerInfo, RunnerVersion},
    xxmi::XxmiManagerStatus,
    CoreEvent, EventSink,
};
use std::{
    future::Future,
    path::PathBuf,
    sync::{mpsc, Arc},
};

/// What a file/folder dialog result should be applied to.
#[derive(Debug, Clone, PartialEq)]
pub enum PickTarget {
    GameInstallPath(String),
    ImportExisting(String),
    GameExecutable(String),
    GamePrefix(String),
    GameRunnerPath(String),
    GameModLibrary(String),
    GameActiveMods(String),
    ImportModFolder(String),
    ImportModArchive(String),
    LegacyGmmConfig(String),
    GlobalInstallPath,
    GlobalPrefixBase,
    GlobalModRoot,
    GlobalRunnerPath,
    XxmiInstallPath,
    XxmiLauncherPath,
}

#[derive(Debug)]
pub enum Msg {
    Core(CoreEvent),
    DownloadStarted(String, Result<InstallJob, String>),
    Launched(String, Result<LaunchResult, String>),
    SessionStarted(String, Arc<gachahub_core::session::GameSession>),
    SessionEnded(String, u64),
    /// Official installer download progress: (game, downloaded, total).
    InstallerProgress(String, u64, u64),
    /// (game, edition id, downloaded installer)
    InstallerReady(String, String, Result<PathBuf, String>),
    /// (game, Some((install folder, exe)) when found)
    OfficialDetected(String, Option<(PathBuf, PathBuf)>),
    SteamApps(Vec<gachahub_core::steam::SteamApp>),
    Runners(Vec<RunnerInfo>),
    RunnerVersions(String, Result<Vec<RunnerVersion>, String>),
    RunnerInstalled(Result<RunnerInfo, String>),
    RunnerRemoved(Result<(), String>),
    Xxmi(Result<XxmiManagerStatus, String>),
    XxmiRemoved(Result<(), String>),
    Mods(String, Result<ModSnapshot, String>),
    PostInstall(String, Result<PostInstallResult, String>),
    Picked(PickTarget, Option<PathBuf>),
    ExeIcon(String, Result<PathBuf, String>),
    LatestVersion(String, Result<String, String>),
    /// (result, started manually by the user)
    AppUpdateChecked(
        Result<Option<gachahub_core::self_update::UpdateInfo>, String>,
        bool,
    ),
    AppUpdateInstalled(Result<(), String>),
    Notice(Result<String, String>),
}

pub struct Backend {
    rt: tokio::runtime::Runtime,
    tx: mpsc::Sender<Msg>,
    rx: mpsc::Receiver<Msg>,
    ctx: egui::Context,
}

impl Backend {
    pub fn new(ctx: egui::Context) -> Self {
        let rt = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(4)
            .enable_all()
            .thread_name("gachahub-worker")
            .build()
            .expect("failed to start async runtime");
        let (tx, rx) = mpsc::channel();
        Self { rt, tx, rx, ctx }
    }

    pub fn drain(&self) -> Vec<Msg> {
        self.rx.try_iter().collect()
    }

    pub fn sender(&self) -> Sender {
        Sender {
            tx: self.tx.clone(),
            ctx: self.ctx.clone(),
        }
    }

    pub fn event_sink(&self) -> EventSink {
        let s = self.sender();
        Arc::new(move |ev| s.send(Msg::Core(ev)))
    }

    /// Run an async task and deliver its message to the UI.
    pub fn spawn<F>(&self, fut: F)
    where
        F: Future<Output = Msg> + Send + 'static,
    {
        let s = self.sender();
        self.rt.spawn(async move { s.send(fut.await) });
    }

    /// Run blocking work (filesystem scans, process spawns) off the UI thread.
    pub fn blocking<F>(&self, work: F)
    where
        F: FnOnce() -> Msg + Send + 'static,
    {
        let s = self.sender();
        self.rt.spawn_blocking(move || s.send(work()));
    }

    pub fn pick_folder(&self, target: PickTarget, title: &str) {
        let title = title.to_string();
        self.spawn(async move {
            let picked = rfd::AsyncFileDialog::new()
                .set_title(title)
                .pick_folder()
                .await;
            Msg::Picked(target, picked.map(|h| h.path().to_path_buf()))
        });
    }

    pub fn pick_file(&self, target: PickTarget, title: &str, filters: &[(&str, &[&str])]) {
        let title = title.to_string();
        let filters: Vec<(String, Vec<String>)> = filters
            .iter()
            .map(|(n, e)| (n.to_string(), e.iter().map(|x| x.to_string()).collect()))
            .collect();
        self.spawn(async move {
            let mut dialog = rfd::AsyncFileDialog::new().set_title(title);
            for (name, exts) in &filters {
                dialog = dialog.add_filter(name, exts);
            }
            let picked = dialog.pick_file().await;
            Msg::Picked(target, picked.map(|h| h.path().to_path_buf()))
        });
    }
}

#[derive(Clone)]
pub struct Sender {
    tx: mpsc::Sender<Msg>,
    ctx: egui::Context,
}

impl Sender {
    pub fn send(&self, msg: Msg) {
        let _ = self.tx.send(msg);
        self.ctx.request_repaint();
    }
}

/// Follow a launched game in the background (see `GameSession`) and report its play time.
pub fn track_session(sender: Sender, game_id: String, child: std::process::Child, game_exe: &str) {
    let session = Arc::new(gachahub_core::session::GameSession::new(child, game_exe));
    sender.send(Msg::SessionStarted(game_id.clone(), session.clone()));
    std::thread::Builder::new()
        .name(format!("session-{game_id}"))
        .spawn(move || {
            let secs = session.wait();
            sender.send(Msg::SessionEnded(game_id, secs));
        })
        .ok();
}

/// Follow a game that another program (the Steam client) starts.
pub fn track_detached(sender: Sender, game_id: String, game_exe: &str) {
    let session = Arc::new(gachahub_core::session::GameSession::detached(game_exe));
    sender.send(Msg::SessionStarted(game_id.clone(), session.clone()));
    std::thread::Builder::new()
        .name(format!("session-{game_id}"))
        .spawn(move || {
            let secs = session.wait();
            sender.send(Msg::SessionEnded(game_id, secs));
        })
        .ok();
}
