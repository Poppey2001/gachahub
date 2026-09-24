use reqwest::Client;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    env,
    fs,
    io::{self, Read, Write},
    path::{Path, PathBuf},
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};
use tauri::{AppHandle, Emitter};
use tokio::{fs as async_fs, io::AsyncWriteExt};
use uuid::Uuid;

const XXMI_RELEASE_API: &str = "https://api.github.com/repos/SpectrumQT/XXMI-Launcher/releases/latest";
const MANAGED_STATE_FILE: &str = ".gachahub-xxmi.json";
const SUPPORTED_IMPORTERS: [&str; 6] = ["GIMI", "SRMI", "ZZMI", "WWMI", "EFMI", "HIMI"];

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct XxmiStatus {
    pub importer: String,
    pub launcher_path: String,
    pub exists: bool,
    pub platform: String,
    pub launch_command: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct XxmiImporterStatus {
    pub id: String,
    pub installed: bool,
    pub path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct XxmiReleaseInfo {
    pub version: String,
    pub tag_name: String,
    pub asset_name: String,
    pub download_url: String,
    pub size: u64,
    pub digest: Option<String>,
    pub published_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct XxmiManagerStatus {
    pub installed: bool,
    pub valid: bool,
    pub managed: bool,
    pub using_custom_path: bool,
    pub version: Option<String>,
    pub latest_version: Option<String>,
    pub update_available: bool,
    pub install_path: String,
    pub launcher_path: String,
    pub platform: String,
    pub importers: Vec<XxmiImporterStatus>,
    pub latest_release: Option<XxmiReleaseInfo>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct XxmiProgress {
    pub phase: String,
    pub progress: f64,
    pub downloaded_bytes: u64,
    pub total_bytes: u64,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ManagedState {
    version: String,
    asset_name: String,
    asset_url: String,
    digest: Option<String>,
    launcher_path: String,
    installed_at_unix: u64,
}

#[derive(Debug, Deserialize)]
struct GithubRelease {
    tag_name: String,
    published_at: Option<String>,
    assets: Vec<GithubAsset>,
}

#[derive(Debug, Deserialize)]
struct GithubAsset {
    name: String,
    browser_download_url: String,
    size: u64,
    digest: Option<String>,
}

fn platform_name() -> &'static str {
    if cfg!(target_os = "windows") {
        "windows"
    } else if cfg!(target_os = "linux") {
        "linux"
    } else {
        "unsupported"
    }
}

fn home_dir() -> Option<PathBuf> {
    env::var_os("HOME")
        .or_else(|| env::var_os("USERPROFILE"))
        .map(PathBuf::from)
}

fn expand_user(raw: &str) -> PathBuf {
    let raw = raw.trim();
    if raw == "~" || raw.starts_with("~/") || raw.starts_with("~\\") {
        if let Some(home) = home_dir() {
            if raw.len() == 1 {
                return home;
            }
            return home.join(&raw[2..]);
        }
    }
    PathBuf::from(raw)
}

fn default_install_path() -> PathBuf {
    if cfg!(target_os = "windows") {
        if let Some(local) = env::var_os("LOCALAPPDATA") {
            return PathBuf::from(local).join("GachaHub").join("tools").join("xxmi");
        }
    }

    if let Some(xdg) = env::var_os("XDG_DATA_HOME") {
        return PathBuf::from(xdg).join("gachahub").join("tools").join("xxmi");
    }

    home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".local")
        .join("share")
        .join("gachahub")
        .join("tools")
        .join("xxmi")
}

fn managed_root(raw: &str) -> PathBuf {
    if raw.trim().is_empty() {
        default_install_path()
    } else {
        expand_user(raw)
    }
}

fn normalize_version(tag: &str) -> String {
    tag.trim().trim_start_matches('v').to_string()
}

fn validate_importer(importer: &str) -> Result<String, String> {
    let importer = importer.trim().to_ascii_uppercase();
    if SUPPORTED_IMPORTERS.contains(&importer.as_str()) {
        Ok(importer)
    } else {
        Err(format!("unsupported XXMI importer: {importer}"))
    }
}

fn read_state(root: &Path) -> Option<ManagedState> {
    let data = fs::read_to_string(root.join(MANAGED_STATE_FILE)).ok()?;
    serde_json::from_str(&data).ok()
}

fn write_state(root: &Path, state: &ManagedState) -> Result<(), String> {
    fs::create_dir_all(root).map_err(|e| format!("failed to create XXMI directory: {e}"))?;
    let data = serde_json::to_vec_pretty(state).map_err(|e| e.to_string())?;
    fs::write(root.join(MANAGED_STATE_FILE), data).map_err(|e| format!("failed to write XXMI state: {e}"))
}

fn find_named_file(root: &Path, file_name: &str, max_depth: usize) -> Option<PathBuf> {
    fn walk(current: &Path, file_name: &str, depth: usize, max_depth: usize) -> Option<PathBuf> {
        if depth > max_depth {
            return None;
        }
        let entries = fs::read_dir(current).ok()?;
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file()
                && path
                    .file_name()
                    .and_then(|x| x.to_str())
                    .map(|x| x.eq_ignore_ascii_case(file_name))
                    .unwrap_or(false)
            {
                return Some(path);
            }
        }
        let entries = fs::read_dir(current).ok()?;
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                if let Some(found) = walk(&path, file_name, depth + 1, max_depth) {
                    return Some(found);
                }
            }
        }
        None
    }

    if !root.is_dir() {
        return None;
    }
    walk(root, file_name, 0, max_depth)
}

fn find_launcher(root: &Path) -> Option<PathBuf> {
    find_named_file(root, "XXMI Launcher.exe", 8)
}

fn find_importer(root: &Path, importer: &str) -> Option<PathBuf> {
    fn walk(current: &Path, importer: &str, depth: usize) -> Option<PathBuf> {
        if depth > 8 {
            return None;
        }
        let entries = fs::read_dir(current).ok()?;
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }
            let is_importer = path
                .file_name()
                .and_then(|x| x.to_str())
                .map(|x| x.eq_ignore_ascii_case(importer))
                .unwrap_or(false);
            if is_importer {
                let has_ini = find_named_file(&path, "d3dx.ini", 4).is_some();
                let has_core = path.join("Core").is_dir();
                let has_mods = path.join("Mods").is_dir();
                if has_ini || has_core || has_mods {
                    return Some(path);
                }
            }
            if let Some(found) = walk(&path, importer, depth + 1) {
                return Some(found);
            }
        }
        None
    }

    if !root.is_dir() {
        return None;
    }
    walk(root, importer, 0)
}

fn importer_statuses(root: &Path) -> Vec<XxmiImporterStatus> {
    SUPPORTED_IMPORTERS
        .iter()
        .map(|id| {
            let path = find_importer(root, id);
            XxmiImporterStatus {
                id: (*id).to_string(),
                installed: path.is_some(),
                path: path.map(|p| p.to_string_lossy().into_owned()),
            }
        })
        .collect()
}

fn launcher_command(launcher: &Path, importer: &str, wine_executable: &str, no_gui: bool) -> Vec<String> {
    let mut command = Vec::new();
    if cfg!(target_os = "linux") {
        command.push(if wine_executable.trim().is_empty() {
            "wine".into()
        } else {
            wine_executable.trim().into()
        });
    }
    command.push(launcher.to_string_lossy().into_owned());
    if no_gui {
        command.push("--nogui".into());
    }
    command.push("--xxmi".into());
    command.push(importer.to_string());
    command
}

async fn latest_release() -> Result<XxmiReleaseInfo, String> {
    let client = Client::builder()
        .user_agent("GachaHub/0.10 (XXMI managed installer)")
        .build()
        .map_err(|e| e.to_string())?;

    let response = client
        .get(XXMI_RELEASE_API)
        .header("Accept", "application/vnd.github+json")
        .send()
        .await
        .map_err(|e| format!("failed to query XXMI releases: {e}"))?
        .error_for_status()
        .map_err(|e| format!("XXMI release query failed: {e}"))?;

    let release: GithubRelease = response
        .json()
        .await
        .map_err(|e| format!("invalid XXMI release response: {e}"))?;

    let asset = release
        .assets
        .into_iter()
        .find(|asset| {
            let lower = asset.name.to_ascii_lowercase();
            lower.contains("xxmi-launcher-portable") && lower.ends_with(".zip")
        })
        .ok_or_else(|| "latest XXMI release has no Portable ZIP asset".to_string())?;

    Ok(XxmiReleaseInfo {
        version: normalize_version(&release.tag_name),
        tag_name: release.tag_name,
        asset_name: asset.name,
        download_url: asset.browser_download_url,
        size: asset.size,
        digest: asset.digest,
        published_at: release.published_at,
    })
}

fn linux_wine_warning(wine_executable: &str) -> Option<String> {
    if !cfg!(target_os = "linux") {
        return None;
    }
    let wine = if wine_executable.trim().is_empty() {
        "wine"
    } else {
        wine_executable.trim()
    };
    let output = Command::new(wine).arg("--version").output().ok()?;
    if !output.status.success() {
        return Some(format!("{wine} is present but --version failed"));
    }
    let version = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if version.is_empty() {
        None
    } else {
        Some(format!("Linux XXMI uses {version}; upstream recommends Wine 9.22 or newer for Portable mode."))
    }
}

pub async fn manager_status(
    install_path: &str,
    custom_launcher_path: &str,
    wine_executable: &str,
    wine_prefix: &str,
) -> Result<XxmiManagerStatus, String> {
    let root = managed_root(install_path);
    let custom = if custom_launcher_path.trim().is_empty() {
        None
    } else {
        Some(expand_user(custom_launcher_path))
    };

    let managed_launcher = find_launcher(&root);
    let custom_launcher = custom.filter(|path| path.is_file());
    let selected_launcher = custom_launcher.clone().or_else(|| managed_launcher.clone());
    let using_custom_path = custom_launcher.is_some();
    let importer_scan_root = custom_launcher
        .as_ref()
        .and_then(|launcher| launcher.parent())
        .and_then(|bin| bin.parent())
        .and_then(|resources| resources.parent())
        .map(Path::to_path_buf)
        .unwrap_or_else(|| root.clone());
    let state = read_state(&root);

    let mut warnings = Vec::new();
    if platform_name() == "unsupported" {
        warnings.push("XXMI managed launching is currently supported only on Windows and Linux/Wine.".into());
    }
    if cfg!(target_os = "linux") {
        if let Some(warning) = linux_wine_warning(wine_executable) {
            warnings.push(warning);
        } else {
            warnings.push("Wine executable could not be detected. XXMI Portable needs Wine on Linux.".into());
        }
        if !wine_prefix.trim().is_empty() {
            warnings.push(format!("XXMI Wine prefix: {}", expand_user(wine_prefix).display()));
        }
    }

    let latest = match latest_release().await {
        Ok(release) => Some(release),
        Err(error) => {
            warnings.push(error);
            None
        }
    };

    let version = if using_custom_path {
        Some("custom".into())
    } else {
        state.as_ref().map(|x| x.version.clone())
    };
    let latest_version = latest.as_ref().map(|x| x.version.clone());
    let update_available = !using_custom_path
        && version
            .as_ref()
            .zip(latest_version.as_ref())
            .map(|(installed, newest)| installed != newest)
            .unwrap_or(false);

    Ok(XxmiManagerStatus {
        installed: selected_launcher.is_some(),
        valid: selected_launcher.as_ref().map(|p| p.is_file()).unwrap_or(false),
        managed: !using_custom_path && managed_launcher.is_some(),
        using_custom_path,
        version,
        latest_version,
        update_available,
        install_path: root.to_string_lossy().into_owned(),
        launcher_path: selected_launcher
            .map(|p| p.to_string_lossy().into_owned())
            .unwrap_or_default(),
        platform: platform_name().into(),
        importers: importer_statuses(&importer_scan_root),
        latest_release: latest,
        warnings,
    })
}

fn emit_progress(app: &AppHandle, phase: &str, progress: f64, downloaded: u64, total: u64, message: impl Into<String>) {
    let payload = XxmiProgress {
        phase: phase.into(),
        progress: progress.clamp(0.0, 1.0),
        downloaded_bytes: downloaded,
        total_bytes: total,
        message: message.into(),
    };
    let _ = app.emit("xxmi-progress", payload);
}

async fn download_asset(app: &AppHandle, release: &XxmiReleaseInfo, destination: &Path) -> Result<(), String> {
    let client = Client::builder()
        .user_agent("GachaHub/0.10 (XXMI managed installer)")
        .build()
        .map_err(|e| e.to_string())?;

    let mut response = client
        .get(&release.download_url)
        .send()
        .await
        .map_err(|e| format!("failed to download XXMI: {e}"))?
        .error_for_status()
        .map_err(|e| format!("XXMI download failed: {e}"))?;

    if let Some(parent) = destination.parent() {
        async_fs::create_dir_all(parent).await.map_err(|e| e.to_string())?;
    }
    let mut file = async_fs::File::create(destination)
        .await
        .map_err(|e| format!("failed to create XXMI download: {e}"))?;

    let total = response.content_length().unwrap_or(release.size);
    let mut downloaded = 0u64;
    while let Some(chunk) = response
        .chunk()
        .await
        .map_err(|e| format!("XXMI download interrupted: {e}"))?
    {
        file.write_all(&chunk)
            .await
            .map_err(|e| format!("failed to write XXMI download: {e}"))?;
        downloaded += chunk.len() as u64;
        let progress = if total > 0 { downloaded as f64 / total as f64 } else { 0.0 };
        emit_progress(app, "downloading", progress, downloaded, total, "Downloading XXMI Portable…");
    }
    file.flush().await.map_err(|e| e.to_string())?;
    Ok(())
}

fn verify_digest(path: &Path, digest: Option<&str>) -> Result<(), String> {
    let Some(digest) = digest else {
        return Ok(());
    };
    let expected = digest
        .strip_prefix("sha256:")
        .or_else(|| digest.strip_prefix("sha256="));
    let Some(expected) = expected else {
        return Ok(());
    };

    let mut file = fs::File::open(path).map_err(|e| e.to_string())?;
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 128 * 1024];
    loop {
        let read = file.read(&mut buffer).map_err(|e| e.to_string())?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    let actual = format!("{:x}", hasher.finalize());
    if !actual.eq_ignore_ascii_case(expected) {
        return Err(format!("XXMI SHA-256 mismatch: expected {expected}, got {actual}"));
    }
    Ok(())
}

fn extract_zip(archive_path: &Path, destination: &Path) -> Result<(), String> {
    let file = fs::File::open(archive_path).map_err(|e| format!("cannot open XXMI ZIP: {e}"))?;
    let mut archive = zip::ZipArchive::new(file).map_err(|e| format!("invalid XXMI ZIP: {e}"))?;

    for index in 0..archive.len() {
        let mut entry = archive.by_index(index).map_err(|e| format!("cannot read XXMI ZIP entry: {e}"))?;
        let Some(relative) = entry.enclosed_name().map(|p| p.to_path_buf()) else {
            return Err(format!("unsafe path in XXMI archive: {}", entry.name()));
        };
        let target = destination.join(relative);
        if entry.is_dir() {
            fs::create_dir_all(&target).map_err(|e| e.to_string())?;
            continue;
        }
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let mut output = fs::File::create(&target).map_err(|e| e.to_string())?;
        io::copy(&mut entry, &mut output).map_err(|e| e.to_string())?;
        output.flush().map_err(|e| e.to_string())?;
    }
    Ok(())
}

fn merge_dir(source: &Path, destination: &Path) -> Result<(), String> {
    fs::create_dir_all(destination).map_err(|e| e.to_string())?;
    for entry in fs::read_dir(source).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let from = entry.path();
        let to = destination.join(entry.file_name());
        if from.is_dir() {
            merge_dir(&from, &to)?;
        } else {
            if let Some(parent) = to.parent() {
                fs::create_dir_all(parent).map_err(|e| e.to_string())?;
            }
            fs::copy(&from, &to).map_err(|e| format!("failed to install {}: {e}", to.display()))?;
        }
    }
    Ok(())
}

pub async fn install_managed(app: AppHandle, install_path: &str) -> Result<XxmiManagerStatus, String> {
    let root = managed_root(install_path);
    let release = latest_release().await?;
    let downloads = root.join(".downloads");
    let archive_path = downloads.join(&release.asset_name);
    let staging = root
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join(format!(".xxmi-staging-{}", Uuid::new_v4()));

    emit_progress(&app, "resolving", 0.0, 0, release.size, format!("XXMI {}", release.version));
    if staging.exists() {
        fs::remove_dir_all(&staging).map_err(|e| e.to_string())?;
    }
    fs::create_dir_all(&staging).map_err(|e| e.to_string())?;

    let result: Result<(), String> = async {
        download_asset(&app, &release, &archive_path).await?;
        emit_progress(&app, "verifying", 1.0, release.size, release.size, "Verifying XXMI download…");
        verify_digest(&archive_path, release.digest.as_deref())?;

        emit_progress(&app, "extracting", 0.0, release.size, release.size, "Extracting XXMI Portable…");
        let archive_clone = archive_path.clone();
        let staging_clone = staging.clone();
        tokio::task::spawn_blocking(move || extract_zip(&archive_clone, &staging_clone))
            .await
            .map_err(|e| format!("XXMI extraction task failed: {e}"))??;

        let staged_launcher = find_launcher(&staging)
            .ok_or_else(|| "XXMI archive did not contain Resources/Bin/XXMI Launcher.exe".to_string())?;
        let relative_launcher = staged_launcher
            .strip_prefix(&staging)
            .map_err(|e| e.to_string())?
            .to_path_buf();

        let source = staging.clone();
        let destination = root.clone();
        tokio::task::spawn_blocking(move || merge_dir(&source, &destination))
            .await
            .map_err(|e| format!("XXMI install task failed: {e}"))??;

        let launcher = root.join(relative_launcher);
        if !launcher.is_file() {
            return Err("XXMI launcher is missing after extraction".into());
        }

        let installed_at_unix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        write_state(
            &root,
            &ManagedState {
                version: release.version.clone(),
                asset_name: release.asset_name.clone(),
                asset_url: release.download_url.clone(),
                digest: release.digest.clone(),
                launcher_path: launcher.to_string_lossy().into_owned(),
                installed_at_unix,
            },
        )?;

        Ok(())
    }
    .await;

    let _ = fs::remove_dir_all(&staging);
    let _ = fs::remove_file(&archive_path);

    match result {
        Ok(()) => {
            emit_progress(&app, "ready", 1.0, release.size, release.size, format!("XXMI {} installed", release.version));
            manager_status(install_path, "", "wine", "").await
        }
        Err(error) => {
            emit_progress(&app, "error", 0.0, 0, release.size, error.clone());
            Err(error)
        }
    }
}

pub async fn repair_managed(app: AppHandle, install_path: &str) -> Result<XxmiManagerStatus, String> {
    install_managed(app, install_path).await
}

pub async fn remove_managed(install_path: &str) -> Result<(), String> {
    let root = managed_root(install_path);
    if root.exists() {
        async_fs::remove_dir_all(&root)
            .await
            .map_err(|e| format!("failed to remove managed XXMI installation: {e}"))?;
    }
    Ok(())
}

pub fn status(launcher_path: &str, importer: &str, wine_executable: &str, no_gui: bool) -> Result<XxmiStatus, String> {
    let importer = validate_importer(importer)?;
    let launcher = expand_user(launcher_path);
    Ok(XxmiStatus {
        importer: importer.clone(),
        launcher_path: launcher.to_string_lossy().into_owned(),
        exists: launcher.is_file(),
        platform: platform_name().into(),
        launch_command: launcher_command(&launcher, &importer, wine_executable, no_gui),
    })
}

pub fn managed_launch(
    install_path: &str,
    custom_launcher_path: &str,
    importer: &str,
    wine_executable: &str,
    wine_prefix: &str,
    no_gui: bool,
) -> Result<XxmiStatus, String> {
    let root = managed_root(install_path);
    let launcher = if !custom_launcher_path.trim().is_empty() {
        let path = expand_user(custom_launcher_path);
        if path.is_file() {
            path
        } else {
            return Err(format!("custom XXMI Launcher not found: {}", path.display()));
        }
    } else {
        find_launcher(&root).ok_or_else(|| {
            format!(
                "managed XXMI Launcher not found in {}. Install XXMI from GachaHub first.",
                root.display()
            )
        })?
    };

    launch_path(&launcher, importer, wine_executable, wine_prefix, no_gui)
}

fn launch_path(launcher: &Path, importer: &str, wine_executable: &str, wine_prefix: &str, no_gui: bool) -> Result<XxmiStatus, String> {
    let importer = validate_importer(importer)?;
    let mut command = if cfg!(target_os = "windows") {
        Command::new(launcher)
    } else if cfg!(target_os = "linux") {
        let wine = if wine_executable.trim().is_empty() {
            "wine"
        } else {
            wine_executable.trim()
        };
        let mut cmd = Command::new(wine);
        if !wine_prefix.trim().is_empty() {
            cmd.env("WINEPREFIX", expand_user(wine_prefix));
        }
        cmd.arg(launcher);
        cmd
    } else {
        return Err("XXMI launching is currently implemented for Windows and Linux/Wine only".into());
    };

    if no_gui {
        command.arg("--nogui");
    }
    command.arg("--xxmi").arg(&importer);
    if let Some(parent) = launcher.parent() {
        command.current_dir(parent);
    }
    command
        .spawn()
        .map_err(|e| format!("failed to launch XXMI: {e}"))?;

    Ok(XxmiStatus {
        importer: importer.clone(),
        launcher_path: launcher.to_string_lossy().into_owned(),
        exists: launcher.is_file(),
        platform: platform_name().into(),
        launch_command: launcher_command(launcher, &importer, wine_executable, no_gui),
    })
}

pub fn launch(launcher_path: &str, importer: &str, wine_executable: &str, no_gui: bool) -> Result<XxmiStatus, String> {
    let launcher = expand_user(launcher_path);
    if !launcher.is_file() {
        return Err(format!("XXMI Launcher not found: {}", launcher.display()));
    }
    launch_path(&launcher, importer, wine_executable, "", no_gui)
}
