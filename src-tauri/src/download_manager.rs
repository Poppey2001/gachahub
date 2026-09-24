use crate::{install::InstallJob, providers::{self, ProviderDownloadPlan}};
use reqwest::header::{CONTENT_LENGTH, RANGE};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    env,
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex, OnceLock,
    },
    time::{Duration, Instant},
};
use tauri::{AppHandle, Emitter};
use tokio::{
    fs::{self, OpenOptions},
    io::{AsyncReadExt, AsyncWriteExt},
    time::sleep,
};
use uuid::Uuid;

#[derive(Debug)]
struct DownloadControl {
    paused: AtomicBool,
    cancelled: AtomicBool,
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

fn expand_user(raw: &str) -> PathBuf {
    let raw = raw.trim();
    if raw == "~" || raw.starts_with("~/") || raw.starts_with("~\\") {
        if let Some(home) = env::var_os("HOME").or_else(|| env::var_os("USERPROFILE")) {
            if raw.len() == 1 {
                return PathBuf::from(home);
            }
            return PathBuf::from(home).join(&raw[2..]);
        }
    }
    PathBuf::from(raw)
}

fn default_install_path(game_id: &str) -> PathBuf {
    let base = home_dir().unwrap_or_else(|| PathBuf::from("."));
    base.join("Games").join("GachaHub").join("Games").join(game_id)
}

fn home_dir() -> Option<PathBuf> {
    env::var_os("HOME").or_else(|| env::var_os("USERPROFILE")).map(PathBuf::from)
}

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

pub async fn resolve_plan(game_id: &str) -> Result<ProviderDownloadPlan, String> {
    match game_id {
        "genshin" | "hsr" | "zzz" | "hi3" => providers::hoyoverse::fetch_download_plan(game_id)
            .await
            .map_err(|e| e.to_string()),
        _ => Err(format!("real download metadata is not connected for {game_id} yet")),
    }
}

async fn fill_missing_sizes(client: &reqwest::Client, plan: &mut ProviderDownloadPlan) {
    for package in &mut plan.packages {
        if package.size > 0 {
            continue;
        }
        if let Ok(response) = client.head(&package.url).send().await {
            if let Some(length) = response.headers().get(CONTENT_LENGTH).and_then(|x| x.to_str().ok()).and_then(|x| x.parse::<u64>().ok()) {
                package.size = length;
            }
        }
    }
    plan.total_bytes = plan.packages.iter().map(|p| p.size).sum();
}

async fn verify_md5(path: &Path, expected: &str) -> Result<bool, String> {
    let mut file = fs::File::open(path).await.map_err(|e| format!("cannot verify {}: {e}", path.display()))?;
    let mut context = md5::Context::new();
    let mut buffer = vec![0u8; 1024 * 1024];
    loop {
        let read = file.read(&mut buffer).await.map_err(|e| format!("verification read failed: {e}"))?;
        if read == 0 { break; }
        context.consume(&buffer[..read]);
    }
    let actual = format!("{:x}", context.compute());
    Ok(actual.eq_ignore_ascii_case(expected.trim()))
}

fn emit_job(app: &AppHandle, job: &InstallJob) {
    let _ = app.emit("download-progress", job);
}

async fn wait_if_paused(control: &DownloadControl) -> Result<(), String> {
    loop {
        if control.cancelled.load(Ordering::Relaxed) {
            return Err("download cancelled".into());
        }
        if !control.paused.load(Ordering::Relaxed) {
            return Ok(());
        }
        sleep(Duration::from_millis(150)).await;
    }
}

async fn run_download(
    app: AppHandle,
    mut job: InstallJob,
    plan: ProviderDownloadPlan,
    install_path: PathBuf,
    control: Arc<DownloadControl>,
) -> Result<(), String> {
    let client = reqwest::Client::builder()
        .user_agent("GachaHub/0.9")
        .connect_timeout(Duration::from_secs(20))
        .timeout(Duration::from_secs(120))
        .build()
        .map_err(|e| e.to_string())?;

    let cache_dir = install_path.join(".gachahub-downloads");
    fs::create_dir_all(&cache_dir).await.map_err(|e| format!("cannot create download directory: {e}"))?;

    let mut completed = 0u64;

    let mut emitted_at = Instant::now();

    job.phase = "downloading".into();
    job.downloaded_bytes = completed;
    job.progress = if job.total_bytes > 0 { completed as f32 / job.total_bytes as f32 } else { 0.0 };
    emit_job(&app, &job);

    for (index, package) in plan.packages.iter().enumerate() {
        wait_if_paused(&control).await?;

        let name = safe_file_name(&package.destination, index);
        let final_path = cache_dir.join(&name);
        let part_path = cache_dir.join(format!("{name}.part"));

        if let Ok(meta) = fs::metadata(&final_path).await {
            if package.size == 0 || meta.len() == package.size {
                completed = completed.saturating_add(if package.size > 0 { package.size } else { meta.len() });
                job.downloaded_bytes = completed;
                job.progress = if job.total_bytes > 0 { (completed as f64 / job.total_bytes as f64).min(1.0) as f32 } else { 0.0 };
                emit_job(&app, &job);
                continue;
            }
            let _ = fs::remove_file(&final_path).await;
        }

        let mut resume_from = fs::metadata(&part_path).await.map(|x| x.len()).unwrap_or(0);
        if package.size > 0 && resume_from > package.size {
            let _ = fs::remove_file(&part_path).await;
            resume_from = 0;
        }
        let mut request = client.get(&package.url);
        if resume_from > 0 {
            request = request.header(RANGE, format!("bytes={resume_from}-"));
        }
        let mut response = request.send().await.map_err(|e| format!("download request failed for {name}: {e}"))?;

        if resume_from > 0 && response.status() != reqwest::StatusCode::PARTIAL_CONTENT {
            resume_from = 0;
            let _ = fs::remove_file(&part_path).await;
            response = client.get(&package.url).send().await.map_err(|e| format!("download restart failed for {name}: {e}"))?;
        }
        response = response.error_for_status().map_err(|e| format!("download failed for {name}: {e}"))?;

        let mut speed_at = Instant::now();
        let mut speed_base = completed.saturating_add(resume_from);

        let mut file = OpenOptions::new()
            .create(true)
            .append(resume_from > 0)
            .write(true)
            .truncate(resume_from == 0)
            .open(&part_path)
            .await
            .map_err(|e| format!("cannot open {name}: {e}"))?;

        let mut current = resume_from;
        while let Some(chunk) = response.chunk().await.map_err(|e| format!("network read failed for {name}: {e}"))? {
            wait_if_paused(&control).await?;
            file.write_all(&chunk).await.map_err(|e| format!("disk write failed for {name}: {e}"))?;
            current = current.saturating_add(chunk.len() as u64);
            let overall = completed.saturating_add(current);

            if emitted_at.elapsed() >= Duration::from_millis(180) {
                let elapsed = speed_at.elapsed().as_secs_f64().max(0.001);
                let delta = overall.saturating_sub(speed_base);
                job.speed_bytes = (delta as f64 / elapsed) as u64;
                job.downloaded_bytes = overall;
                job.progress = if job.total_bytes > 0 { (overall as f64 / job.total_bytes as f64).min(1.0) as f32 } else { 0.0 };
                job.message = Some(format!("Downloading {name}"));
                emit_job(&app, &job);
                emitted_at = Instant::now();
                if speed_at.elapsed() >= Duration::from_secs(1) {
                    speed_at = Instant::now();
                    speed_base = overall;
                }
            }
        }
        file.flush().await.map_err(|e| format!("flush failed for {name}: {e}"))?;
        drop(file);

        if package.size > 0 {
            let actual = fs::metadata(&part_path).await.map_err(|e| e.to_string())?.len();
            if actual != package.size {
                return Err(format!("size mismatch for {name}: expected {}, got {actual}", package.size));
            }
        }

        if let Some(expected) = package.hash.as_deref().filter(|x| !x.trim().is_empty()) {
            job.phase = "verifying".into();
            job.message = Some(format!("Verifying {name}"));
            job.speed_bytes = 0;
            emit_job(&app, &job);
            if !verify_md5(&part_path, expected).await? {
                return Err(format!("MD5 verification failed for {name}"));
            }
            job.phase = "downloading".into();
        }

        fs::rename(&part_path, &final_path).await.map_err(|e| format!("cannot finalize {name}: {e}"))?;
        completed = completed.saturating_add(if package.size > 0 { package.size } else { current });
    }

    job.phase = "downloaded".into();
    job.progress = 1.0;
    job.downloaded_bytes = if job.total_bytes > 0 { job.total_bytes } else { completed };
    job.speed_bytes = 0;
    job.message = Some("Packages downloaded. Extraction/install processing is the next integration step.".into());
    emit_job(&app, &job);
    Ok(())
}

pub async fn start(app: AppHandle, game_id: String, install_path: String) -> Result<InstallJob, String> {
    let mut plan = resolve_plan(&game_id).await?;
    let client = reqwest::Client::builder()
        .user_agent("GachaHub/0.9")
        .timeout(Duration::from_secs(30))
        .build()
        .map_err(|e| e.to_string())?;
    fill_missing_sizes(&client, &mut plan).await;

    if plan.mode != "direct" || plan.packages.is_empty() {
        let extra = plan.resource_list_url.as_deref().unwrap_or("no resource list URL");
        return Err(format!("{} {} uses {} download metadata ({extra}). GachaHub v0.9 resolves it correctly, but the Sophon chunk assembler is the next step.", game_id, plan.version, plan.mode));
    }

    let destination = if install_path.trim().is_empty() {
        default_install_path(&game_id)
    } else {
        expand_user(&install_path)
    };
    let id = Uuid::new_v4().to_string();
    let control = Arc::new(DownloadControl {
        paused: AtomicBool::new(false),
        cancelled: AtomicBool::new(false),
    });
    controls().lock().map_err(|_| "download control lock poisoned")?.insert(id.clone(), control.clone());

    let job = InstallJob {
        id: id.clone(),
        game_id: game_id.clone(),
        phase: "queued".into(),
        progress: 0.0,
        downloaded_bytes: 0,
        total_bytes: plan.total_bytes,
        speed_bytes: 0,
        version: Some(plan.version.clone()),
        provider_mode: Some(plan.mode.clone()),
        destination: Some(destination.to_string_lossy().into_owned()),
        message: Some(format!("Resolved {} package(s) from {}", plan.packages.len(), plan.provider)),
    };

    let app_for_task = app.clone();
    let job_for_task = job.clone();
    tauri::async_runtime::spawn(async move {
        if let Err(error) = run_download(app_for_task.clone(), job_for_task.clone(), plan, destination, control).await {
            let mut failed = job_for_task;
            failed.phase = if error == "download cancelled" { "cancelled".into() } else { "error".into() };
            failed.speed_bytes = 0;
            failed.message = Some(error);
            emit_job(&app_for_task, &failed);
        }
        if let Ok(mut map) = controls().lock() {
            map.remove(&id);
        }
    });

    Ok(job)
}

pub fn pause(job_id: &str, paused: bool) -> Result<DownloadControlState, String> {
    let map = controls().lock().map_err(|_| "download control lock poisoned")?;
    let control = map.get(job_id).ok_or_else(|| format!("download job not active: {job_id}"))?;
    control.paused.store(paused, Ordering::Relaxed);
    Ok(DownloadControlState {
        job_id: job_id.into(),
        paused,
        cancelled: control.cancelled.load(Ordering::Relaxed),
    })
}

pub fn cancel(job_id: &str) -> Result<DownloadControlState, String> {
    let map = controls().lock().map_err(|_| "download control lock poisoned")?;
    let control = map.get(job_id).ok_or_else(|| format!("download job not active: {job_id}"))?;
    control.cancelled.store(true, Ordering::Relaxed);
    Ok(DownloadControlState {
        job_id: job_id.into(),
        paused: control.paused.load(Ordering::Relaxed),
        cancelled: true,
    })
}
