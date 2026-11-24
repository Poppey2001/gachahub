//! Game installation, updates and repairs.
//!
//! Two sources are supported:
//! * **Sophon** (HoYoPlay chunk distribution) – preferred for HoYoverse games.
//!   Files are assembled from MD5-verified chunks, many in parallel. Unchanged
//!   files are skipped, so the same routine performs installs, updates and repairs.
//! * **Direct packages** – classic (split) archives. They are downloaded with
//!   HTTP-Range resume, verified and then extracted into the install folder.
//!
//! Progress is reported as [`InstallJob`] snapshots through the [`EventSink`].

use crate::{
    events::{CoreEvent, EventSink},
    install::InstallJob,
    providers::{self, ProviderDownloadPlan},
    sophon,
};
use futures::{stream, TryStreamExt};
use reqwest::header::{CONTENT_LENGTH, RANGE};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    io::{Read, Seek, SeekFrom},
    path::{Component, Path, PathBuf},
    sync::{
        atomic::{AtomicBool, AtomicU64, Ordering},
        Arc, Mutex, OnceLock,
    },
    time::{Duration, Instant},
};
use tokio::{
    fs::{self, OpenOptions},
    io::{AsyncReadExt, AsyncSeekExt, AsyncWriteExt},
    time::sleep,
};
use uuid::Uuid;

/// Marker written into the game folder after a successful install/update.
pub const VERSION_FILE: &str = ".gachahub-version";
/// Per-file MD5 index so updates do not need to re-hash unchanged files.
const INDEX_FILE: &str = ".gachahub-files.json";
const CACHE_DIR: &str = ".gachahub-downloads";
const MAX_RETRIES: u32 = 5;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JobKind {
    Install,
    Update,
    Verify,
}

impl JobKind {
    fn as_str(self) -> &'static str {
        match self {
            JobKind::Install => "install",
            JobKind::Update => "update",
            JobKind::Verify => "verify",
        }
    }
}

#[derive(Debug, Clone)]
pub struct DownloadOptions {
    pub kind: JobKind,
    /// Number of files downloaded at the same time (Sophon).
    pub parallel: usize,
    /// Optional global bandwidth limit in Mbit/s.
    pub bandwidth_limit_mbps: Option<u32>,
    /// Voice pack language, HoYoPlay code like `en-us`, `ja-jp`, `zh-cn`, `ko-kr`.
    pub voice_language: String,
}

impl Default for DownloadOptions {
    fn default() -> Self {
        Self {
            kind: JobKind::Install,
            parallel: 8,
            bandwidth_limit_mbps: None,
            voice_language: "en-us".into(),
        }
    }
}

// ============================================================ job control

#[derive(Debug)]
struct DownloadControl {
    paused: AtomicBool,
    cancelled: AtomicBool,
    snapshot: Mutex<InstallJob>,
    resume_phase: Mutex<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DownloadControlState {
    pub job_id: String,
    pub paused: bool,
    pub cancelled: bool,
}

fn controls() -> &'static Mutex<HashMap<String, Arc<DownloadControl>>> {
    static CONTROLS: OnceLock<Mutex<HashMap<String, Arc<DownloadControl>>>> = OnceLock::new();
    CONTROLS.get_or_init(|| Mutex::new(HashMap::new()))
}

impl DownloadControl {
    async fn wait_if_paused(&self) -> Result<bool, String> {
        let mut waited = false;
        loop {
            if self.cancelled.load(Ordering::Relaxed) {
                return Err(CANCELLED.into());
            }
            if !self.paused.load(Ordering::Relaxed) {
                return Ok(waited);
            }
            waited = true;
            sleep(Duration::from_millis(150)).await;
        }
    }
    fn check_cancel(&self) -> Result<(), String> {
        if self.cancelled.load(Ordering::Relaxed) {
            Err(CANCELLED.into())
        } else {
            Ok(())
        }
    }
}

const CANCELLED: &str = "download cancelled";

/// Shared progress counters + periodic event emission.
struct Tracker {
    sink: EventSink,
    control: Arc<DownloadControl>,
    done: AtomicU64,
    total: AtomicU64,
    limiter: Option<RateLimiter>,
}

impl Tracker {
    fn update(&self, f: impl FnOnce(&mut InstallJob)) {
        let job = {
            let Ok(mut job) = self.control.snapshot.lock() else {
                return;
            };
            f(&mut job);
            job.clone()
        };
        (self.sink)(CoreEvent::Download(job));
    }

    fn set_phase(&self, phase: &str, message: impl Into<String>) {
        let message = message.into();
        self.update(|job| {
            if job.phase != "paused" {
                job.phase = phase.into();
            }
            job.message = Some(message);
        });
    }

    fn add(&self, bytes: u64) {
        self.done.fetch_add(bytes, Ordering::Relaxed);
    }

    async fn throttle(&self, bytes: usize) {
        if let Some(l) = &self.limiter {
            l.acquire(bytes as u64).await;
        }
    }

    /// Emit progress every 250 ms until `stop` is set.
    fn spawn_ticker(self: &Arc<Self>, stop: Arc<AtomicBool>) {
        let tracker = self.clone();
        tokio::spawn(async move {
            let mut last_done = tracker.done.load(Ordering::Relaxed);
            let mut last_at = Instant::now();
            let mut speed = 0f64;
            while !stop.load(Ordering::Relaxed) {
                sleep(Duration::from_millis(250)).await;
                let done = tracker.done.load(Ordering::Relaxed);
                let total = tracker.total.load(Ordering::Relaxed);
                let dt = last_at.elapsed().as_secs_f64().max(0.001);
                let inst = done.saturating_sub(last_done) as f64 / dt;
                speed = if speed == 0.0 {
                    inst
                } else {
                    speed * 0.8 + inst * 0.2
                };
                last_done = done;
                last_at = Instant::now();
                let paused = tracker.control.paused.load(Ordering::Relaxed);
                tracker.update(|job| {
                    job.downloaded_bytes = done.min(total.max(done));
                    job.total_bytes = total;
                    job.progress = if total > 0 {
                        (done as f64 / total as f64).min(1.0) as f32
                    } else {
                        0.0
                    };
                    job.speed_bytes = if paused { 0 } else { speed as u64 };
                });
            }
        });
    }
}

/// Token bucket shared by all download streams of a job.
struct RateLimiter {
    bytes_per_sec: f64,
    state: tokio::sync::Mutex<(f64, Instant)>,
}

impl RateLimiter {
    fn new(mbps: u32) -> Self {
        let bytes_per_sec = mbps.max(1) as f64 * 1_000_000.0 / 8.0;
        Self {
            bytes_per_sec,
            state: tokio::sync::Mutex::new((bytes_per_sec, Instant::now())),
        }
    }
    async fn acquire(&self, bytes: u64) {
        let wait = {
            let mut st = self.state.lock().await;
            let now = Instant::now();
            st.0 = (st.0 + now.duration_since(st.1).as_secs_f64() * self.bytes_per_sec)
                .min(self.bytes_per_sec);
            st.1 = now;
            st.0 -= bytes as f64;
            if st.0 < 0.0 {
                Duration::from_secs_f64(-st.0 / self.bytes_per_sec)
            } else {
                Duration::ZERO
            }
        };
        if !wait.is_zero() {
            sleep(wait).await;
        }
    }
}

fn http_client(timeout: Option<Duration>) -> Result<reqwest::Client, String> {
    let mut b = reqwest::Client::builder()
        .user_agent(crate::USER_AGENT)
        .connect_timeout(Duration::from_secs(20));
    if let Some(t) = timeout {
        b = b.timeout(t);
    }
    b.build().map_err(|e| e.to_string())
}

/// Retry transient failures with exponential backoff.
async fn with_retries<T, F, Fut>(
    control: &DownloadControl,
    what: &str,
    mut f: F,
) -> Result<T, String>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = Result<T, String>>,
{
    let mut attempt = 0;
    loop {
        control.wait_if_paused().await?;
        match f().await {
            Ok(v) => return Ok(v),
            Err(e) if e == CANCELLED => return Err(e),
            Err(e) if attempt + 1 >= MAX_RETRIES => return Err(format!("{what}: {e}")),
            Err(_) => {
                attempt += 1;
                sleep(Duration::from_millis(500 * 2u64.pow(attempt))).await;
            }
        }
    }
}

// ============================================================ public API

/// Whether GachaHub can download this game itself (otherwise: import an
/// installation made with the publisher's official launcher).
pub fn supports_direct_download(game_id: &str) -> bool {
    matches!(game_id, "genshin" | "hsr" | "zzz" | "hi3") || crate::kuro::supports(game_id)
}

pub async fn resolve_plan(
    game_id: &str,
    voice_language: &str,
) -> Result<ProviderDownloadPlan, String> {
    match game_id {
        "genshin" | "hsr" | "zzz" | "hi3" => {
            providers::hoyoverse::fetch_download_plan(game_id, voice_language)
                .await
                .map_err(|e| e.to_string())
        }
        _ => Err(format!(
            "GachaHub cannot download {game_id} itself yet – import an existing installation"
        )),
    }
}

/// Installed game version: GachaHub's marker file, or HoYoPlay's `config.ini`.
pub fn installed_version(install_path: &str) -> Option<String> {
    let dir = crate::expand_user(install_path);
    if let Ok(v) = std::fs::read_to_string(dir.join(VERSION_FILE)) {
        let v = v.trim().to_string();
        if !v.is_empty() {
            return Some(v);
        }
    }
    let ini = std::fs::read_to_string(dir.join("config.ini")).ok()?;
    ini.lines()
        .filter_map(|l| l.split_once('='))
        .find(|(k, _)| k.trim() == "game_version")
        .map(|(_, v)| v.trim().to_string())
        .filter(|v| !v.is_empty())
}

/// Latest version published by the provider.
pub async fn latest_version(game_id: &str) -> Result<String, String> {
    if !supports_direct_download(game_id) {
        return Err(format!("no update source for {game_id}"));
    }
    let client = http_client(Some(Duration::from_secs(30)))?;
    if crate::kuro::supports(game_id) {
        return crate::kuro::fetch_index(&client, game_id)
            .await
            .map(|i| i.version);
    }
    match sophon::branch(&client, game_id).await {
        Ok(b) if !b.tag.is_empty() => Ok(b.tag),
        _ => resolve_plan(game_id, "en-us").await.map(|p| p.version),
    }
}

/// Compare dotted versions numerically ("5.10.0" > "5.9.1").
pub fn is_newer(latest: &str, installed: &str) -> bool {
    let parse = |v: &str| {
        v.split(['.', '-', '_'])
            .map(|p| p.parse::<u64>().unwrap_or(0))
            .collect::<Vec<_>>()
    };
    let (a, b) = (parse(latest), parse(installed));
    for i in 0..a.len().max(b.len()) {
        let (x, y) = (
            a.get(i).copied().unwrap_or(0),
            b.get(i).copied().unwrap_or(0),
        );
        if x != y {
            return x > y;
        }
    }
    false
}

enum Source {
    Sophon {
        build: sophon::Build,
    },
    /// Plain per-file distribution with MD5 (Kuro launcher).
    Files {
        build: crate::kuro::KuroBuild,
    },
    Direct {
        plan: ProviderDownloadPlan,
    },
}

pub async fn start(
    sink: EventSink,
    game_id: String,
    install_path: String,
    opts: DownloadOptions,
) -> Result<InstallJob, String> {
    let client = http_client(Some(Duration::from_secs(30)))?;

    // Kuro games: file list from the Kuro launcher index.
    // HoYoverse: prefer Sophon; fall back to classic packages for fresh installs.
    let source = if crate::kuro::supports(&game_id) {
        Ok(Source::Files {
            build: crate::kuro::fetch_build(&client, &game_id).await?,
        })
    } else {
        match sophon::branch(&client, &game_id).await {
            Ok(branch) => match sophon::build(&client, &branch).await {
                Ok(build) => Ok(Source::Sophon { build }),
                Err(e) => Err(e),
            },
            Err(e) => Err(e),
        }
    };
    let source = match source {
        Ok(s) => s,
        Err(sophon_err) => {
            if opts.kind != JobKind::Install {
                return Err(format!(
                    "update/repair needs HoYoPlay Sophon data: {sophon_err}"
                ));
            }
            let mut plan = resolve_plan(&game_id, &opts.voice_language)
                .await
                .map_err(|e| format!("{e} (Sophon: {sophon_err})"))?;
            if plan.packages.is_empty() {
                return Err(format!(
                    "no downloadable packages for {game_id} (Sophon: {sophon_err})"
                ));
            }
            fill_missing_sizes(&client, &mut plan).await;
            Source::Direct { plan }
        }
    };

    let destination = if install_path.trim().is_empty() {
        crate::expand_user(&format!("~/Games/GachaHub/Games/{game_id}"))
    } else {
        crate::expand_user(&install_path)
    };
    let id = Uuid::new_v4().to_string();
    let (version, mode, total) = match &source {
        Source::Sophon { build } => (
            build.tag.clone(),
            "sophon",
            build
                .manifests
                .iter()
                .filter(|m| wanted_manifest(m, &opts.voice_language))
                .map(|m| m.uncompressed_size)
                .sum(),
        ),
        Source::Direct { plan } => (plan.version.clone(), "direct", plan.total_bytes),
        Source::Files { build } => (
            build.version.clone(),
            "files",
            build.files.iter().map(|f| f.size).sum(),
        ),
    };
    let job = InstallJob {
        id: id.clone(),
        game_id: game_id.clone(),
        phase: "queued".into(),
        progress: 0.0,
        downloaded_bytes: 0,
        total_bytes: total,
        speed_bytes: 0,
        version: Some(version.clone()),
        provider_mode: Some(mode.into()),
        destination: Some(destination.to_string_lossy().into_owned()),
        message: Some(format!("{} {version}", opts.kind.as_str())),
        kind: opts.kind.as_str().into(),
    };

    let control = Arc::new(DownloadControl {
        paused: AtomicBool::new(false),
        cancelled: AtomicBool::new(false),
        snapshot: Mutex::new(job.clone()),
        resume_phase: Mutex::new("downloading".into()),
    });
    controls()
        .lock()
        .map_err(|_| "download control lock poisoned")?
        .insert(id.clone(), control.clone());

    let tracker = Arc::new(Tracker {
        sink: sink.clone(),
        control: control.clone(),
        done: AtomicU64::new(0),
        total: AtomicU64::new(total),
        limiter: opts
            .bandwidth_limit_mbps
            .filter(|m| *m > 0)
            .map(RateLimiter::new),
    });

    tokio::spawn(async move {
        let stop = Arc::new(AtomicBool::new(false));
        tracker.spawn_ticker(stop.clone());
        let result = async {
            fs::create_dir_all(&destination)
                .await
                .map_err(|e| format!("cannot create {}: {e}", destination.display()))?;
            match source {
                Source::Sophon { build } => {
                    run_sophon(&tracker, &build, &destination, &opts).await?
                }
                Source::Direct { plan } => run_direct(&tracker, &plan, &destination).await?,
                Source::Files { build } => {
                    run_files(&tracker, &build.files, &destination, &opts).await?
                }
            }
            fs::write(destination.join(VERSION_FILE), &version)
                .await
                .map_err(|e| format!("cannot write version marker: {e}"))?;
            Ok::<(), String>(())
        }
        .await;
        stop.store(true, Ordering::Relaxed);
        sleep(Duration::from_millis(300)).await; // let the ticker finish

        match result {
            Ok(()) => tracker.update(|job| {
                job.phase = "ready".into();
                job.progress = 1.0;
                job.downloaded_bytes = job.total_bytes;
                job.speed_bytes = 0;
                job.message = Some(format!(
                    "{} {}",
                    game_ready_message(opts.kind),
                    job.version.clone().unwrap_or_default()
                ));
            }),
            Err(error) => tracker.update(|job| {
                job.phase = if error == CANCELLED {
                    "cancelled".into()
                } else {
                    "error".into()
                };
                job.speed_bytes = 0;
                job.message = Some(if error == CANCELLED {
                    "Cancelled – downloaded data is kept and reused next time.".into()
                } else {
                    error
                });
            }),
        }
        if let Ok(mut map) = controls().lock() {
            map.remove(&id);
        }
    });

    Ok(job)
}

fn game_ready_message(kind: JobKind) -> &'static str {
    match kind {
        JobKind::Install => "Installed",
        JobKind::Update => "Updated to",
        JobKind::Verify => "Verified",
    }
}

pub fn pause(sink: &EventSink, job_id: &str, paused: bool) -> Result<DownloadControlState, String> {
    let control = {
        let map = controls()
            .lock()
            .map_err(|_| "download control lock poisoned")?;
        map.get(job_id)
            .cloned()
            .ok_or_else(|| format!("download job not active: {job_id}"))?
    };
    if control.cancelled.load(Ordering::Relaxed) {
        return Err(format!("download job already cancelled: {job_id}"));
    }
    control.paused.store(paused, Ordering::Relaxed);
    if let Ok(mut job) = control.snapshot.lock() {
        if paused {
            if job.phase != "paused" {
                if let Ok(mut resume) = control.resume_phase.lock() {
                    *resume = job.phase.clone();
                }
            }
            job.phase = "paused".into();
            job.speed_bytes = 0;
            job.message = Some("Download paused".into());
        } else if job.phase == "paused" {
            job.phase = control
                .resume_phase
                .lock()
                .ok()
                .map(|v| v.clone())
                .filter(|v| v != "paused" && v != "queued")
                .unwrap_or_else(|| "downloading".into());
            job.message = Some("Resuming…".into());
        }
        let current = job.clone();
        drop(job);
        sink(CoreEvent::Download(current));
    }
    Ok(DownloadControlState {
        job_id: job_id.into(),
        paused,
        cancelled: false,
    })
}

pub fn cancel(sink: &EventSink, job_id: &str) -> Result<DownloadControlState, String> {
    let control = {
        let map = controls()
            .lock()
            .map_err(|_| "download control lock poisoned")?;
        map.get(job_id)
            .cloned()
            .ok_or_else(|| format!("download job not active: {job_id}"))?
    };
    control.cancelled.store(true, Ordering::Relaxed);
    control.paused.store(false, Ordering::Relaxed);
    if let Ok(mut job) = control.snapshot.lock() {
        job.phase = "cancelled".into();
        job.speed_bytes = 0;
        job.message = Some("Cancelled – downloaded data is kept and reused next time.".into());
        let current = job.clone();
        drop(job);
        sink(CoreEvent::Download(current));
    }
    Ok(DownloadControlState {
        job_id: job_id.into(),
        paused: false,
        cancelled: true,
    })
}

// ============================================================ Sophon

fn wanted_manifest(m: &sophon::ManifestRef, voice: &str) -> bool {
    m.matching_field == "game" || m.matching_field.eq_ignore_ascii_case(voice)
}

/// Turn a manifest path into a path below `root`, rejecting `..` and absolute paths.
fn safe_join(root: &Path, name: &str) -> Result<PathBuf, String> {
    let rel = Path::new(name.trim_start_matches(['/', '\\']));
    let mut out = root.to_path_buf();
    for c in rel.components() {
        match c {
            Component::Normal(p) => out.push(p),
            Component::CurDir => {}
            _ => return Err(format!("unsafe path in manifest: {name}")),
        }
    }
    Ok(out)
}

async fn md5_file(path: &Path, control: &DownloadControl) -> Result<String, String> {
    let mut file = fs::File::open(path).await.map_err(|e| e.to_string())?;
    let mut ctx = md5::Context::new();
    let mut buf = vec![0u8; 1 << 20];
    loop {
        control.check_cancel()?;
        let n = file.read(&mut buf).await.map_err(|e| e.to_string())?;
        if n == 0 {
            break;
        }
        ctx.consume(&buf[..n]);
    }
    Ok(format!("{:x}", ctx.compute()))
}

async fn run_sophon(
    tracker: &Arc<Tracker>,
    build: &sophon::Build,
    root: &Path,
    opts: &DownloadOptions,
) -> Result<(), String> {
    let control = &tracker.control;
    let client = http_client(Some(Duration::from_secs(120)))?;

    tracker.set_phase("resolving", "Loading file manifests…");
    let mut work: Vec<(Arc<sophon::ManifestRef>, sophon::Asset)> = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for manifest in build
        .manifests
        .iter()
        .filter(|m| wanted_manifest(m, &opts.voice_language))
    {
        let manifest = Arc::new(manifest.clone());
        let assets = with_retries(control, "manifest", || {
            sophon::fetch_assets(&client, &manifest)
        })
        .await?;
        for asset in assets {
            if seen.insert(asset.name.clone()) {
                work.push((manifest.clone(), asset));
            }
        }
    }
    let total: u64 = work.iter().map(|(_, a)| a.size).sum();
    tracker.total.store(total, Ordering::Relaxed);

    let index_path = root.join(INDEX_FILE);
    let index: HashMap<String, String> = fs::read(&index_path)
        .await
        .ok()
        .and_then(|b| serde_json::from_slice(&b).ok())
        .unwrap_or_default();
    let index = Arc::new(Mutex::new(index));
    let verify = opts.kind == JobKind::Verify;
    tracker.set_phase(
        if verify { "verifying" } else { "downloading" },
        format!("{} files", work.len()),
    );

    let processed = Arc::new(AtomicU64::new(0));
    let result = stream::iter(work.into_iter().map(Ok::<_, String>))
        .try_for_each_concurrent(opts.parallel.clamp(1, 32), |(manifest, asset)| {
            let (client, tracker, index, processed) = (
                client.clone(),
                tracker.clone(),
                index.clone(),
                processed.clone(),
            );
            let root = root.to_path_buf();
            async move {
                sophon_asset(&client, &tracker, &manifest, &asset, &root, &index, verify).await?;
                // Persist the index now and then so an interrupted job resumes quickly.
                if processed.fetch_add(1, Ordering::Relaxed) % 200 == 199 {
                    save_index(&root, &index).await;
                }
                Ok(())
            }
        })
        .await;
    save_index(root, &index).await;
    result
}

async fn save_index(root: &Path, index: &Mutex<HashMap<String, String>>) {
    let data = index.lock().ok().and_then(|i| serde_json::to_vec(&*i).ok());
    if let Some(data) = data {
        let _ = fs::write(root.join(INDEX_FILE), data).await;
    }
}

async fn sophon_asset(
    client: &reqwest::Client,
    tracker: &Tracker,
    manifest: &sophon::ManifestRef,
    asset: &sophon::Asset,
    root: &Path,
    index: &Mutex<HashMap<String, String>>,
    verify: bool,
) -> Result<(), String> {
    let control = &tracker.control;
    control.wait_if_paused().await?;
    let path = safe_join(root, &asset.name)?;
    if asset.is_directory() {
        fs::create_dir_all(&path).await.map_err(|e| e.to_string())?;
        return Ok(());
    }
    let expected = asset.md5.to_ascii_lowercase();
    let on_disk = fs::metadata(&path).await.ok().map(|m| m.len());

    // Unchanged file? Trust the index for updates, re-hash when verifying.
    if on_disk == Some(asset.size) {
        let known = index.lock().ok().and_then(|i| i.get(&asset.name).cloned());
        let matches = if !verify && known.as_deref() == Some(expected.as_str()) {
            true
        } else {
            md5_file(&path, control).await? == expected
        };
        if matches {
            if let Ok(mut i) = index.lock() {
                i.insert(asset.name.clone(), expected);
            }
            tracker.add(asset.size);
            return Ok(());
        }
    }

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .await
            .map_err(|e| e.to_string())?;
    }
    let tmp = path.with_file_name(format!(
        "{}.gachahub-part",
        path.file_name()
            .map(|n| n.to_string_lossy())
            .unwrap_or_default()
    ));
    let mut file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(&tmp)
        .await
        .map_err(|e| format!("cannot write {}: {e}", tmp.display()))?;
    file.set_len(asset.size).await.map_err(|e| e.to_string())?;

    for chunk in &asset.chunks {
        let data = with_retries(control, &format!("chunk {}", chunk.name), || async {
            control.wait_if_paused().await?;
            let mut response = client
                .get(manifest.chunk_url(chunk))
                .send()
                .await
                .and_then(|r| r.error_for_status())
                .map_err(|e| e.to_string())?;
            let mut raw = Vec::with_capacity(chunk.size as usize);
            while let Some(bytes) = response.chunk().await.map_err(|e| e.to_string())? {
                control.check_cancel()?;
                tracker.throttle(bytes.len()).await;
                raw.extend_from_slice(&bytes);
            }
            let data = if manifest.chunks_compressed {
                zstd::stream::decode_all(&raw[..])
                    .map_err(|e| format!("decompression failed: {e}"))?
            } else {
                raw
            };
            if !chunk.decompressed_md5.is_empty()
                && !format!("{:x}", md5::compute(&data))
                    .eq_ignore_ascii_case(&chunk.decompressed_md5)
            {
                return Err("chunk checksum mismatch".to_string());
            }
            Ok(data)
        })
        .await?;
        file.seek(SeekFrom::Start(chunk.offset))
            .await
            .map_err(|e| e.to_string())?;
        file.write_all(&data)
            .await
            .map_err(|e| format!("disk write failed: {e}"))?;
        tracker.add(data.len() as u64);
    }
    file.flush().await.map_err(|e| e.to_string())?;
    drop(file);
    if fs::metadata(&path).await.is_ok() {
        let _ = fs::remove_file(&path).await;
    }
    fs::rename(&tmp, &path)
        .await
        .map_err(|e| format!("cannot finalize {}: {e}", path.display()))?;
    if let Ok(mut i) = index.lock() {
        i.insert(asset.name.clone(), expected);
    }
    Ok(())
}

// ============================================================ plain file lists (Kuro)

async fn run_files(
    tracker: &Arc<Tracker>,
    files: &[crate::kuro::KuroFile],
    root: &Path,
    opts: &DownloadOptions,
) -> Result<(), String> {
    let client = http_client(Some(Duration::from_secs(300)))?;
    tracker
        .total
        .store(files.iter().map(|f| f.size).sum(), Ordering::Relaxed);
    let index_path = root.join(INDEX_FILE);
    let index: HashMap<String, String> = fs::read(&index_path)
        .await
        .ok()
        .and_then(|b| serde_json::from_slice(&b).ok())
        .unwrap_or_default();
    let index = Arc::new(Mutex::new(index));
    let verify = opts.kind == JobKind::Verify;
    tracker.set_phase(
        if verify { "verifying" } else { "downloading" },
        format!("{} files", files.len()),
    );
    let processed = Arc::new(AtomicU64::new(0));
    let result = stream::iter(files.iter().cloned().map(Ok::<_, String>))
        .try_for_each_concurrent(opts.parallel.clamp(1, 32), |file| {
            let (client, tracker, index, processed) = (
                client.clone(),
                tracker.clone(),
                index.clone(),
                processed.clone(),
            );
            let root = root.to_path_buf();
            async move {
                plain_file(&client, &tracker, &file, &root, &index, verify).await?;
                if processed.fetch_add(1, Ordering::Relaxed) % 200 == 199 {
                    save_index(&root, &index).await;
                }
                Ok(())
            }
        })
        .await;
    save_index(root, &index).await;
    result
}

async fn plain_file(
    client: &reqwest::Client,
    tracker: &Tracker,
    file: &crate::kuro::KuroFile,
    root: &Path,
    index: &Mutex<HashMap<String, String>>,
    verify: bool,
) -> Result<(), String> {
    let control = &tracker.control;
    control.wait_if_paused().await?;
    let path = safe_join(root, &file.dest)?;
    let on_disk = fs::metadata(&path).await.ok().map(|m| m.len());
    if on_disk == Some(file.size) {
        let known = index.lock().ok().and_then(|i| i.get(&file.dest).cloned());
        let ok = if file.md5.is_empty() {
            true
        } else if !verify && known.as_deref() == Some(file.md5.as_str()) {
            true
        } else {
            md5_file(&path, control).await? == file.md5
        };
        if ok {
            if let Ok(mut i) = index.lock() {
                i.insert(file.dest.clone(), file.md5.clone());
            }
            tracker.add(file.size);
            return Ok(());
        }
    }
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .await
            .map_err(|e| e.to_string())?;
    }
    let part = path.with_file_name(format!(
        "{}.gachahub-part",
        path.file_name()
            .map(|n| n.to_string_lossy())
            .unwrap_or_default()
    ));
    let mut failures = 0u32;
    let mut current = fs::metadata(&part).await.map(|m| m.len()).unwrap_or(0);
    if file.size > 0 && current > file.size {
        let _ = fs::remove_file(&part).await;
        current = 0;
    }
    tracker.add(current);
    loop {
        control.wait_if_paused().await?;
        match download_range(client, tracker, &file.url, &part, &mut current).await {
            Ok(true) => break,
            Ok(false) => continue,
            Err(e) if e == CANCELLED => return Err(e),
            Err(e) => {
                failures += 1;
                if failures >= MAX_RETRIES {
                    return Err(format!("download of {} failed: {e}", file.dest));
                }
                sleep(Duration::from_millis(1000 * 2u64.pow(failures))).await;
            }
        }
    }
    if !file.md5.is_empty() && md5_file(&part, control).await? != file.md5 {
        let _ = fs::remove_file(&part).await;
        return Err(format!("MD5 mismatch for {} – please retry", file.dest));
    }
    if fs::metadata(&path).await.is_ok() {
        let _ = fs::remove_file(&path).await;
    }
    fs::rename(&part, &path)
        .await
        .map_err(|e| format!("cannot finalize {}: {e}", path.display()))?;
    if let Ok(mut i) = index.lock() {
        i.insert(file.dest.clone(), file.md5.clone());
    }
    Ok(())
}

// ============================================================ direct packages

fn safe_file_name(raw: &str, index: usize) -> String {
    let raw = Path::new(raw)
        .file_name()
        .and_then(|x| x.to_str())
        .unwrap_or("")
        .trim();
    if raw.is_empty() || raw == "." || raw == ".." {
        format!("package-{index}.bin")
    } else {
        raw.to_string()
    }
}

async fn fill_missing_sizes(client: &reqwest::Client, plan: &mut ProviderDownloadPlan) {
    for package in &mut plan.packages {
        if package.size > 0 {
            continue;
        }
        if let Ok(response) = client.head(&package.url).send().await {
            if let Some(length) = response
                .headers()
                .get(CONTENT_LENGTH)
                .and_then(|x| x.to_str().ok())
                .and_then(|x| x.parse::<u64>().ok())
            {
                package.size = length;
            }
        }
    }
    plan.total_bytes = plan.packages.iter().map(|p| p.size).sum();
}

async fn run_direct(
    tracker: &Arc<Tracker>,
    plan: &ProviderDownloadPlan,
    root: &Path,
) -> Result<(), String> {
    let control = &tracker.control;
    let client = http_client(None)?;
    let cache = root.join(CACHE_DIR);
    fs::create_dir_all(&cache)
        .await
        .map_err(|e| format!("cannot create download directory: {e}"))?;
    tracker.set_phase("downloading", format!("{} package(s)", plan.packages.len()));

    let mut files = Vec::new();
    for (index, package) in plan.packages.iter().enumerate() {
        let name = safe_file_name(&package.destination, index);
        let final_path = cache.join(&name);
        files.push(final_path.clone());

        if let Ok(meta) = fs::metadata(&final_path).await {
            if package.size == 0 || meta.len() == package.size {
                tracker.add(meta.len());
                continue;
            }
            let _ = fs::remove_file(&final_path).await;
        }
        let part = cache.join(format!("{name}.part"));
        let mut current = fs::metadata(&part).await.map(|m| m.len()).unwrap_or(0);
        if package.size > 0 && current > package.size {
            let _ = fs::remove_file(&part).await;
            current = 0;
        }
        tracker.add(current);
        tracker.set_phase("downloading", format!("Downloading {name}"));

        let mut failures = 0u32;
        loop {
            control.wait_if_paused().await?;
            match download_range(&client, tracker, &package.url, &part, &mut current).await {
                Ok(true) => break,
                Ok(false) => continue, // paused: reconnect from the committed offset
                Err(e) if e == CANCELLED => return Err(e),
                Err(e) => {
                    failures += 1;
                    if failures >= MAX_RETRIES {
                        return Err(format!("download of {name} failed: {e}"));
                    }
                    tracker.set_phase(
                        "downloading",
                        format!("Connection lost, retrying {name} ({failures}/{MAX_RETRIES})…"),
                    );
                    sleep(Duration::from_millis(1000 * 2u64.pow(failures))).await;
                }
            }
        }

        if package.size > 0 {
            let actual = fs::metadata(&part).await.map_err(|e| e.to_string())?.len();
            if actual != package.size {
                let _ = fs::remove_file(&part).await;
                return Err(format!(
                    "size mismatch for {name}: expected {}, got {actual}",
                    package.size
                ));
            }
        }
        if let Some(expected) = package.hash.as_deref().filter(|x| !x.trim().is_empty()) {
            tracker.set_phase("verifying", format!("Verifying {name}"));
            if !md5_file(&part, control)
                .await?
                .eq_ignore_ascii_case(expected.trim())
            {
                let _ = fs::remove_file(&part).await;
                return Err(format!(
                    "MD5 verification failed for {name} – the file was removed, please retry"
                ));
            }
            tracker.set_phase("downloading", format!("{name} verified"));
        }
        fs::rename(&part, &final_path)
            .await
            .map_err(|e| format!("cannot finalize {name}: {e}"))?;
    }

    // ---- extraction
    tracker.set_phase("extracting", "Extracting game files…");
    let extract_total: u64 = files
        .iter()
        .filter_map(|f| std::fs::metadata(f).ok())
        .map(|m| m.len())
        .sum();
    tracker.done.store(0, Ordering::Relaxed);
    tracker.total.store(extract_total, Ordering::Relaxed);
    let (root_c, files_c, tracker_c) = (root.to_path_buf(), files.clone(), tracker.clone());
    tokio::task::spawn_blocking(move || extract_packages(&files_c, &root_c, &tracker_c))
        .await
        .map_err(|e| format!("extraction task failed: {e}"))??;

    let _ = fs::remove_dir_all(&cache).await;
    Ok(())
}

/// Stream `url` into `part` starting at `current`. Returns Ok(true) when the
/// file is complete, Ok(false) when interrupted by a pause.
async fn download_range(
    client: &reqwest::Client,
    tracker: &Tracker,
    url: &str,
    part: &Path,
    current: &mut u64,
) -> Result<bool, String> {
    let control = &tracker.control;
    let mut request = client.get(url);
    if *current > 0 {
        request = request.header(RANGE, format!("bytes={current}-"));
    }
    let mut response = request.send().await.map_err(|e| e.to_string())?;
    if *current > 0 && response.status() != reqwest::StatusCode::PARTIAL_CONTENT {
        // Server ignores Range – restart this file cleanly.
        tracker.done.fetch_sub(
            (*current).min(tracker.done.load(Ordering::Relaxed)),
            Ordering::Relaxed,
        );
        *current = 0;
        let _ = fs::remove_file(part).await;
        return Ok(false);
    }
    response = response.error_for_status().map_err(|e| e.to_string())?;
    let mut file = OpenOptions::new()
        .create(true)
        .append(*current > 0)
        .write(true)
        .truncate(*current == 0)
        .open(part)
        .await
        .map_err(|e| e.to_string())?;
    loop {
        if control.paused.load(Ordering::Relaxed) {
            file.flush().await.map_err(|e| e.to_string())?;
            return Ok(false);
        }
        control.check_cancel()?;
        match response.chunk().await {
            Ok(Some(chunk)) => {
                tracker.throttle(chunk.len()).await;
                file.write_all(&chunk)
                    .await
                    .map_err(|e| format!("disk write failed: {e}"))?;
                *current += chunk.len() as u64;
                tracker.add(chunk.len() as u64);
            }
            Ok(None) => {
                file.flush().await.map_err(|e| e.to_string())?;
                return Ok(true);
            }
            Err(e) => {
                let _ = file.flush().await;
                return Err(e.to_string());
            }
        }
    }
}

/// Reads several raw split volumes (`x.zip.001`, `x.zip.002`, …) as one file.
struct MultiFile {
    parts: Vec<(std::fs::File, u64)>,
    total: u64,
    pos: u64,
}

impl MultiFile {
    fn open(paths: &[PathBuf]) -> Result<Self, String> {
        let mut parts = Vec::new();
        let mut total = 0;
        for p in paths {
            let f =
                std::fs::File::open(p).map_err(|e| format!("cannot open {}: {e}", p.display()))?;
            let len = f.metadata().map_err(|e| e.to_string())?.len();
            total += len;
            parts.push((f, len));
        }
        Ok(Self {
            parts,
            total,
            pos: 0,
        })
    }
}

impl Read for MultiFile {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let mut offset = self.pos;
        for (file, len) in &mut self.parts {
            if offset < *len {
                file.seek(SeekFrom::Start(offset))?;
                let want = buf.len().min((*len - offset) as usize);
                let n = file.read(&mut buf[..want])?;
                self.pos += n as u64;
                return Ok(n);
            }
            offset -= *len;
        }
        Ok(0)
    }
}

impl Seek for MultiFile {
    fn seek(&mut self, pos: SeekFrom) -> std::io::Result<u64> {
        let new = match pos {
            SeekFrom::Start(p) => p as i128,
            SeekFrom::End(d) => self.total as i128 + d as i128,
            SeekFrom::Current(d) => self.pos as i128 + d as i128,
        };
        if new < 0 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "seek before start",
            ));
        }
        self.pos = new as u64;
        Ok(self.pos)
    }
}

/// Split `x.zip.001` → ("x.zip", 1).
fn split_volume(path: &Path) -> Option<(String, u32)> {
    let name = path.file_name()?.to_string_lossy().into_owned();
    let (base, ext) = name.rsplit_once('.')?;
    if ext.len() == 3 && ext.chars().all(|c| c.is_ascii_digit()) {
        Some((base.to_string(), ext.parse().ok()?))
    } else {
        None
    }
}

/// Group downloaded files into archives (single files or split volume sets).
fn archive_groups(files: &[PathBuf]) -> Vec<(String, Vec<PathBuf>)> {
    let mut groups: Vec<(String, Vec<(u32, PathBuf)>)> = Vec::new();
    for f in files {
        let (base, n) = split_volume(f).unwrap_or_else(|| {
            (
                f.file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .into_owned(),
                0,
            )
        });
        match groups.iter_mut().find(|(b, _)| *b == base) {
            Some((_, v)) => v.push((n, f.clone())),
            None => groups.push((base, vec![(n, f.clone())])),
        }
    }
    groups
        .into_iter()
        .map(|(b, mut v)| {
            v.sort_by_key(|(n, _)| *n);
            (b, v.into_iter().map(|(_, p)| p).collect())
        })
        .collect()
}

fn extract_packages(files: &[PathBuf], root: &Path, tracker: &Tracker) -> Result<(), String> {
    for (base, parts) in archive_groups(files) {
        tracker.control.check_cancel()?;
        let lower = base.to_ascii_lowercase();
        let size: u64 = parts
            .iter()
            .filter_map(|p| std::fs::metadata(p).ok())
            .map(|m| m.len())
            .sum();
        if lower.ends_with(".zip") {
            extract_zip(MultiFile::open(&parts)?, root, tracker, size)?;
        } else if lower.ends_with(".7z") || lower.ends_with(".rar") {
            let tool = crate::mod_manager::find_7zip().ok_or(
                "7-Zip is required to extract this game package. Please install 7-Zip and retry.",
            )?;
            let status = std::process::Command::new(tool)
                .arg("x")
                .arg("-y")
                .arg(format!("-o{}", root.display()))
                .arg(&parts[0])
                .stdout(std::process::Stdio::null())
                .status()
                .map_err(|e| format!("7-Zip failed to start: {e}"))?;
            if !status.success() {
                return Err(format!("7-Zip could not extract {base}"));
            }
            tracker.add(size);
        } else {
            return Err(format!("unsupported package format: {base}"));
        }
    }
    Ok(())
}

fn extract_zip<R: Read + Seek>(
    reader: R,
    root: &Path,
    tracker: &Tracker,
    size: u64,
) -> Result<(), String> {
    let mut archive =
        zip::ZipArchive::new(reader).map_err(|e| format!("invalid game archive: {e}"))?;
    let total_compressed: u64 = (0..archive.len())
        .filter_map(|i| archive.by_index_raw(i).ok().map(|f| f.compressed_size()))
        .sum::<u64>()
        .max(1);
    for i in 0..archive.len() {
        tracker.control.check_cancel()?;
        while tracker.control.paused.load(Ordering::Relaxed) {
            std::thread::sleep(Duration::from_millis(200));
            tracker.control.check_cancel()?;
        }
        let mut entry = archive.by_index(i).map_err(|e| e.to_string())?;
        let rel = entry
            .enclosed_name()
            .ok_or_else(|| format!("unsafe path in game archive: {}", entry.name()))?;
        let out = root.join(rel);
        if entry.is_dir() {
            std::fs::create_dir_all(&out).map_err(|e| e.to_string())?;
        } else {
            if let Some(p) = out.parent() {
                std::fs::create_dir_all(p).map_err(|e| e.to_string())?;
            }
            let mut f = std::fs::File::create(&out)
                .map_err(|e| format!("cannot write {}: {e}", out.display()))?;
            std::io::copy(&mut entry, &mut f).map_err(|e| format!("extraction failed: {e}"))?;
        }
        tracker.add(entry.compressed_size() * size / total_compressed);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn version_comparison() {
        assert!(is_newer("5.10.0", "5.9.1"));
        assert!(!is_newer("5.2.0", "5.2.0"));
        assert!(!is_newer("4.8", "5.0"));
    }

    #[test]
    fn safe_join_rejects_traversal() {
        let root = Path::new("/games/x");
        assert!(safe_join(root, "../etc/passwd").is_err());
        assert_eq!(
            safe_join(root, "a/b.dat").unwrap(),
            root.join("a").join("b.dat")
        );
    }

    #[test]
    fn groups_split_volumes() {
        let files = vec![
            PathBuf::from("/c/Game.zip.002"),
            PathBuf::from("/c/Game.zip.001"),
            PathBuf::from("/c/Audio_en.zip"),
        ];
        let g = archive_groups(&files);
        assert_eq!(g.len(), 2);
        assert_eq!(g[0].0, "Game.zip");
        assert_eq!(g[0].1[0], PathBuf::from("/c/Game.zip.001"));
        assert_eq!(g[1].0, "Audio_en.zip");
    }

    #[test]
    fn extracts_split_zip() {
        let dir = std::env::temp_dir().join(format!("gachahub-test-{}", Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        // build a zip in memory and split it into 3 raw volumes
        let mut buf = std::io::Cursor::new(Vec::new());
        {
            let mut zip = zip::ZipWriter::new(&mut buf);
            let opts = zip::write::SimpleFileOptions::default();
            zip.start_file("GameData/a.txt", opts).unwrap();
            zip.write_all(&vec![b'x'; 5000]).unwrap();
            zip.start_file("Game.exe", opts).unwrap();
            zip.write_all(b"MZ").unwrap();
            zip.finish().unwrap();
        }
        let bytes = buf.into_inner();
        let third = bytes.len() / 3 + 1;
        let mut parts = Vec::new();
        for (i, c) in bytes.chunks(third).enumerate() {
            let p = dir.join(format!("Game.zip.{:03}", i + 1));
            std::fs::write(&p, c).unwrap();
            parts.push(p);
        }
        let control = Arc::new(DownloadControl {
            paused: AtomicBool::new(false),
            cancelled: AtomicBool::new(false),
            snapshot: Mutex::new(InstallJob {
                id: String::new(),
                game_id: String::new(),
                phase: String::new(),
                progress: 0.0,
                downloaded_bytes: 0,
                total_bytes: 0,
                speed_bytes: 0,
                version: None,
                provider_mode: None,
                destination: None,
                message: None,
                kind: String::new(),
            }),
            resume_phase: Mutex::new(String::new()),
        });
        let tracker = Tracker {
            sink: Arc::new(|_| {}),
            control,
            done: AtomicU64::new(0),
            total: AtomicU64::new(0),
            limiter: None,
        };
        let out = dir.join("out");
        extract_packages(&parts, &out, &tracker).unwrap();
        assert_eq!(
            std::fs::read(out.join("GameData/a.txt")).unwrap().len(),
            5000
        );
        assert_eq!(std::fs::read(out.join("Game.exe")).unwrap(), b"MZ");
        let _ = std::fs::remove_dir_all(&dir);
    }

    // ------------------------------------------------ end-to-end with a local HTTP server

    async fn serve(files: HashMap<String, Vec<u8>>) -> String {
        use tokio::io::{AsyncReadExt, AsyncWriteExt};
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let files = Arc::new(files);
        tokio::spawn(async move {
            loop {
                let Ok((mut sock, _)) = listener.accept().await else {
                    break;
                };
                let files = files.clone();
                tokio::spawn(async move {
                    let mut req = Vec::new();
                    let mut buf = [0u8; 1024];
                    while !req.windows(4).any(|w| w == b"\r\n\r\n") {
                        let n = sock.read(&mut buf).await.unwrap_or(0);
                        if n == 0 {
                            return;
                        }
                        req.extend_from_slice(&buf[..n]);
                    }
                    let text = String::from_utf8_lossy(&req).to_string();
                    let path = text
                        .split_whitespace()
                        .nth(1)
                        .unwrap_or("/")
                        .split('?')
                        .next()
                        .unwrap()
                        .to_string();
                    let range = text.lines().find_map(|l| {
                        l.to_ascii_lowercase()
                            .strip_prefix("range: bytes=")
                            .map(|r| r.trim_end_matches('-').parse::<usize>().unwrap_or(0))
                    });
                    let resp = match files.get(&path) {
                        Some(body) => {
                            let start = range.unwrap_or(0).min(body.len());
                            let status = if range.is_some() { "206 Partial Content" } else { "200 OK" };
                            let mut r = format!("HTTP/1.1 {status}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n", body.len() - start).into_bytes();
                            r.extend_from_slice(&body[start..]);
                            r
                        }
                        None => b"HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\nConnection: close\r\n\r\n".to_vec(),
                    };
                    let _ = sock.write_all(&resp).await;
                });
            }
        });
        format!("http://{addr}")
    }

    fn tracker() -> Arc<Tracker> {
        let control = Arc::new(DownloadControl {
            paused: AtomicBool::new(false),
            cancelled: AtomicBool::new(false),
            snapshot: Mutex::new(InstallJob {
                id: String::new(),
                game_id: String::new(),
                phase: String::new(),
                progress: 0.0,
                downloaded_bytes: 0,
                total_bytes: 0,
                speed_bytes: 0,
                version: None,
                provider_mode: None,
                destination: None,
                message: None,
                kind: String::new(),
            }),
            resume_phase: Mutex::new(String::new()),
        });
        Arc::new(Tracker {
            sink: Arc::new(|_| {}),
            control,
            done: AtomicU64::new(0),
            total: AtomicU64::new(0),
            limiter: None,
        })
    }

    fn pb_varint(mut v: u64, out: &mut Vec<u8>) {
        loop {
            let b = (v & 0x7f) as u8;
            v >>= 7;
            if v == 0 {
                out.push(b);
                break;
            }
            out.push(b | 0x80);
        }
    }
    fn pb_bytes(n: u64, d: &[u8], out: &mut Vec<u8>) {
        pb_varint(n << 3 | 2, out);
        pb_varint(d.len() as u64, out);
        out.extend_from_slice(d);
    }
    fn pb_int(n: u64, v: u64, out: &mut Vec<u8>) {
        pb_varint(n << 3, out);
        pb_varint(v, out);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn sophon_install_update_and_repair() {
        std::env::set_var("NO_PROXY", "127.0.0.1");
        std::env::set_var("no_proxy", "127.0.0.1");
        // Two files, the first one split into two chunks.
        let file_a: Vec<u8> = (0..300_000u32).map(|i| (i % 251) as u8).collect();
        let file_b = b"hello sophon".to_vec();
        let mut served = HashMap::new();
        let mut manifest = Vec::new();
        for (name, data, splits) in [("Game_Data/a.bin", &file_a, 2usize), ("b.txt", &file_b, 1)] {
            let mut asset = Vec::new();
            pb_bytes(1, name.as_bytes(), &mut asset);
            let step = data.len().div_ceil(splits);
            for (i, part) in data.chunks(step).enumerate() {
                let chunk_name = format!("{}_{i}", name.replace('/', "_"));
                let compressed = zstd::stream::encode_all(&part[..], 3).unwrap();
                served.insert(format!("/chunks/{chunk_name}"), compressed.clone());
                let mut c = Vec::new();
                pb_bytes(1, chunk_name.as_bytes(), &mut c);
                pb_bytes(2, format!("{:x}", md5::compute(part)).as_bytes(), &mut c);
                pb_int(3, (i * step) as u64, &mut c);
                pb_int(4, compressed.len() as u64, &mut c);
                pb_int(5, part.len() as u64, &mut c);
                pb_bytes(2, &c, &mut asset);
            }
            pb_int(4, data.len() as u64, &mut asset);
            pb_bytes(
                5,
                format!("{:x}", md5::compute(data)).as_bytes(),
                &mut asset,
            );
            pb_bytes(1, &asset, &mut manifest);
        }
        served.insert(
            "/manifests/m1".into(),
            zstd::stream::encode_all(&manifest[..], 3).unwrap(),
        );
        let base = serve(served).await;
        let build = sophon::Build {
            tag: "9.9.0".into(),
            manifests: vec![sophon::ManifestRef {
                matching_field: "game".into(),
                manifest_url: format!("{base}/manifests/m1"),
                manifest_compressed: true,
                chunk_url_prefix: format!("{base}/chunks"),
                chunk_url_suffix: String::new(),
                chunks_compressed: true,
                uncompressed_size: 0,
                compressed_size: 0,
            }],
        };
        let root = std::env::temp_dir().join(format!("gachahub-sophon-{}", Uuid::new_v4()));
        std::fs::create_dir_all(&root).unwrap();
        let mut opts = DownloadOptions::default();

        // install
        let t = tracker();
        run_sophon(&t, &build, &root, &opts).await.unwrap();
        assert_eq!(std::fs::read(root.join("Game_Data/a.bin")).unwrap(), file_a);
        assert_eq!(std::fs::read(root.join("b.txt")).unwrap(), file_b);
        assert_eq!(
            t.done.load(Ordering::Relaxed),
            (file_a.len() + file_b.len()) as u64
        );

        // update with nothing changed: no re-download, everything counted as done
        opts.kind = JobKind::Update;
        let t = tracker();
        run_sophon(&t, &build, &root, &opts).await.unwrap();
        assert_eq!(
            t.done.load(Ordering::Relaxed),
            (file_a.len() + file_b.len()) as u64
        );

        // repair: corrupt a byte (same size) and verify
        let mut broken = file_a.clone();
        broken[1234] ^= 0xff;
        std::fs::write(root.join("Game_Data/a.bin"), &broken).unwrap();
        opts.kind = JobKind::Verify;
        run_sophon(&tracker(), &build, &root, &opts).await.unwrap();
        assert_eq!(std::fs::read(root.join("Game_Data/a.bin")).unwrap(), file_a);
        let _ = std::fs::remove_dir_all(&root);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn direct_download_with_extraction() {
        std::env::set_var("NO_PROXY", "127.0.0.1");
        std::env::set_var("no_proxy", "127.0.0.1");
        let mut buf = std::io::Cursor::new(Vec::new());
        {
            let mut zip = zip::ZipWriter::new(&mut buf);
            let o = zip::write::SimpleFileOptions::default();
            zip.start_file("ZenlessZoneZero.exe", o).unwrap();
            zip.write_all(&vec![7u8; 40_000]).unwrap();
            zip.finish().unwrap();
        }
        let bytes = buf.into_inner();
        let half = bytes.len() / 2;
        let (p1, p2) = (bytes[..half].to_vec(), bytes[half..].to_vec());
        let mut served = HashMap::new();
        served.insert("/Game.zip.001".to_string(), p1.clone());
        served.insert("/Game.zip.002".to_string(), p2.clone());
        let base = serve(served).await;
        let pkg = |name: &str, data: &[u8]| providers::PackageFile {
            url: format!("{base}/{name}"),
            destination: name.into(),
            size: data.len() as u64,
            hash: Some(format!("{:x}", md5::compute(data))),
        };
        let plan = ProviderDownloadPlan {
            game_id: "zzz".into(),
            provider: "test".into(),
            version: "1.0".into(),
            mode: "direct".into(),
            packages: vec![pkg("Game.zip.001", &p1), pkg("Game.zip.002", &p2)],
            total_bytes: bytes.len() as u64,
            resource_list_url: None,
            notes: vec![],
        };
        let root = std::env::temp_dir().join(format!("gachahub-direct-{}", Uuid::new_v4()));
        std::fs::create_dir_all(&root).unwrap();
        // simulate an interrupted earlier download of part 1 (resume via Range)
        std::fs::create_dir_all(root.join(CACHE_DIR)).unwrap();
        std::fs::write(
            root.join(CACHE_DIR).join("Game.zip.001.part"),
            &p1[..p1.len() / 2],
        )
        .unwrap();
        run_direct(&tracker(), &plan, &root).await.unwrap();
        assert_eq!(
            std::fs::read(root.join("ZenlessZoneZero.exe"))
                .unwrap()
                .len(),
            40_000
        );
        assert!(
            !root.join(CACHE_DIR).exists(),
            "download cache should be removed"
        );
        let _ = std::fs::remove_dir_all(&root);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn kuro_file_list_install_and_verify() {
        std::env::set_var("NO_PROXY", "127.0.0.1");
        std::env::set_var("no_proxy", "127.0.0.1");
        let exe: Vec<u8> = (0..70_000u32).map(|i| (i % 7) as u8).collect();
        let pak = b"pak data".to_vec();
        let mut served = HashMap::new();
        served.insert(
            "/res/Client/Binaries/Win64/Client-Win64-Shipping.exe".to_string(),
            exe.clone(),
        );
        served.insert("/res/Client/Content/Paks/a.pak".to_string(), pak.clone());
        let base = serve(served).await;
        let mk = |dest: &str, data: &[u8]| crate::kuro::KuroFile {
            dest: dest.into(),
            url: format!("{base}/res/{dest}"),
            size: data.len() as u64,
            md5: format!("{:x}", md5::compute(data)),
        };
        let files = vec![
            mk("Client/Binaries/Win64/Client-Win64-Shipping.exe", &exe),
            mk("Client/Content/Paks/a.pak", &pak),
        ];
        let root = std::env::temp_dir().join(format!("gachahub-kuro-{}", Uuid::new_v4()));
        std::fs::create_dir_all(&root).unwrap();
        let mut opts = DownloadOptions::default();
        run_files(&tracker(), &files, &root, &opts).await.unwrap();
        assert_eq!(
            std::fs::read(root.join("Client/Binaries/Win64/Client-Win64-Shipping.exe")).unwrap(),
            exe
        );
        // damage one file and repair it
        std::fs::write(root.join("Client/Content/Paks/a.pak"), b"pak dat!").unwrap();
        opts.kind = JobKind::Verify;
        run_files(&tracker(), &files, &root, &opts).await.unwrap();
        assert_eq!(
            std::fs::read(root.join("Client/Content/Paks/a.pak")).unwrap(),
            pak
        );
        let _ = std::fs::remove_dir_all(&root);
    }
}
