//! Self-updater: checks GitHub releases for a newer GachaHub build, downloads
//! the matching asset, verifies its SHA-256 digest and replaces the running
//! executable (works on Windows, where a running .exe cannot be overwritten,
//! via the `self-replace` crate).

use crate::{CoreEvent, EventSink};
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::{
    io::{Read, Write},
    path::{Path, PathBuf},
};

pub const REPOSITORY: &str = "Poppey2001/gachahub";

#[derive(Debug, Clone)]
pub struct UpdateInfo {
    pub version: String,
    pub tag: String,
    pub prerelease: bool,
    pub notes: String,
    pub page_url: String,
    pub asset_name: String,
    pub asset_url: String,
    pub asset_size: u64,
    /// `sha256:<hex>` as published by GitHub for the asset.
    pub digest: Option<String>,
}

#[derive(Debug, Deserialize)]
struct Release {
    tag_name: String,
    #[serde(default)]
    prerelease: bool,
    #[serde(default)]
    draft: bool,
    #[serde(default)]
    body: Option<String>,
    #[serde(default)]
    html_url: String,
    #[serde(default)]
    assets: Vec<Asset>,
}

#[derive(Debug, Deserialize)]
struct Asset {
    name: String,
    browser_download_url: String,
    #[serde(default)]
    size: u64,
    #[serde(default)]
    digest: Option<String>,
}

/// Release asset that contains the build for this platform.
pub fn platform_asset() -> Option<&'static str> {
    if cfg!(all(target_os = "windows", target_arch = "x86_64")) {
        Some("gachahub.exe")
    } else if cfg!(all(target_os = "linux", target_arch = "x86_64")) {
        Some("gachahub-linux-x86_64.tar.gz")
    } else {
        None
    }
}

fn parse_version(raw: &str) -> Option<semver::Version> {
    semver::Version::parse(raw.trim().trim_start_matches('v')).ok()
}

/// Pick the newest applicable release that is newer than `current`.
fn select_update(
    releases: Vec<Release>,
    current: &str,
    include_prereleases: bool,
    asset: &str,
) -> Option<UpdateInfo> {
    let current = parse_version(current)?;
    releases
        .into_iter()
        .filter(|r| !r.draft && (include_prereleases || !r.prerelease))
        .filter_map(|r| {
            let version = parse_version(&r.tag_name)?;
            let a = r.assets.iter().find(|a| a.name == asset)?;
            Some((
                version,
                UpdateInfo {
                    version: r.tag_name.trim_start_matches('v').to_string(),
                    tag: r.tag_name.clone(),
                    prerelease: r.prerelease,
                    notes: r.body.clone().unwrap_or_default(),
                    page_url: r.html_url.clone(),
                    asset_name: a.name.clone(),
                    asset_url: a.browser_download_url.clone(),
                    asset_size: a.size,
                    digest: a.digest.clone(),
                },
            ))
        })
        .filter(|(v, _)| *v > current)
        .max_by(|(a, _), (b, _)| a.cmp(b))
        .map(|(_, info)| info)
}

fn client() -> Result<reqwest::Client, String> {
    reqwest::Client::builder()
        .user_agent(crate::USER_AGENT)
        .connect_timeout(std::time::Duration::from_secs(15))
        .build()
        .map_err(|e| e.to_string())
}

/// Is there a newer GachaHub than the running one?
pub async fn check(include_prereleases: bool) -> Result<Option<UpdateInfo>, String> {
    let asset = platform_asset().ok_or("self-update is not available for this platform")?;
    let releases: Vec<Release> = client()?
        .get(format!(
            "https://api.github.com/repos/{REPOSITORY}/releases?per_page=30"
        ))
        .header("Accept", "application/vnd.github+json")
        .send()
        .await
        .and_then(|r| r.error_for_status())
        .map_err(|e| format!("update check failed: {e}"))?
        .json()
        .await
        .map_err(|e| format!("update check failed: {e}"))?;
    Ok(select_update(
        releases,
        crate::VERSION,
        include_prereleases,
        asset,
    ))
}

fn staging_dir() -> PathBuf {
    std::env::temp_dir().join("gachahub-update")
}

/// Download, verify and install an update. The new version is active after a restart.
pub async fn install(info: &UpdateInfo, sink: EventSink) -> Result<(), String> {
    let dir = staging_dir();
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let download = dir.join(&info.asset_name);

    // ---- download with progress + SHA-256
    let mut response = client()?
        .get(&info.asset_url)
        .send()
        .await
        .and_then(|r| r.error_for_status())
        .map_err(|e| format!("update download failed: {e}"))?;
    let total = response.content_length().unwrap_or(info.asset_size);
    let mut file = std::fs::File::create(&download).map_err(|e| e.to_string())?;
    let mut hasher = Sha256::new();
    let mut done = 0u64;
    while let Some(chunk) = response
        .chunk()
        .await
        .map_err(|e| format!("update download failed: {e}"))?
    {
        file.write_all(&chunk).map_err(|e| e.to_string())?;
        hasher.update(&chunk);
        done += chunk.len() as u64;
        sink(CoreEvent::SelfUpdate(done, total));
    }
    file.flush().map_err(|e| e.to_string())?;
    drop(file);

    if let Some(expected) = info
        .digest
        .as_deref()
        .and_then(|d| d.strip_prefix("sha256:"))
    {
        let actual = format!("{:x}", hasher.finalize());
        if !actual.eq_ignore_ascii_case(expected) {
            return Err(format!(
                "update checksum mismatch (expected {expected}, got {actual})"
            ));
        }
    }

    // ---- unpack (Linux ships a tarball) and swap the executable
    let (dir2, download2) = (dir.clone(), download.clone());
    tokio::task::spawn_blocking(move || {
        let binary = if download2.to_string_lossy().ends_with(".tar.gz") {
            extract_binary(&download2, &dir2)?
        } else {
            download2
        };
        replace_running_exe(&binary)
    })
    .await
    .map_err(|e| e.to_string())?
}

/// Find `gachahub` inside the release tarball.
fn extract_binary(tarball: &Path, dir: &Path) -> Result<PathBuf, String> {
    let file = std::fs::File::open(tarball).map_err(|e| e.to_string())?;
    let mut archive = tar::Archive::new(flate2::read::GzDecoder::new(file));
    for entry in archive.entries().map_err(|e| e.to_string())? {
        let mut entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path().map_err(|e| e.to_string())?.into_owned();
        if path.file_name().and_then(|n| n.to_str()) == Some("gachahub") {
            let out = dir.join("gachahub-new");
            let mut data = Vec::new();
            entry.read_to_end(&mut data).map_err(|e| e.to_string())?;
            std::fs::write(&out, data).map_err(|e| e.to_string())?;
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                std::fs::set_permissions(&out, std::fs::Permissions::from_mode(0o755))
                    .map_err(|e| e.to_string())?;
            }
            return Ok(out);
        }
    }
    Err("update package does not contain the gachahub binary".into())
}

fn replace_running_exe(new_binary: &Path) -> Result<(), String> {
    self_replace::self_replace(new_binary).map_err(|e| {
        if e.kind() == std::io::ErrorKind::PermissionDenied {
            "no permission to replace GachaHub – it is installed system-wide. Re-run the installer with sudo or update via the release page.".to_string()
        } else {
            format!("could not install the update: {e}")
        }
    })?;
    let _ = std::fs::remove_dir_all(staging_dir());
    Ok(())
}

/// Start the (updated) executable again and quit the current process.
/// `exe` must be captured *before* the update – on Linux the running binary's
/// path changes to "… (deleted)" once it has been replaced.
pub fn restart(exe: &Path) -> Result<(), String> {
    std::process::Command::new(exe)
        .spawn()
        .map_err(|e| format!("could not restart GachaHub: {e}"))?;
    std::process::exit(0);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rel(tag: &str, pre: bool) -> Release {
        Release {
            tag_name: tag.into(),
            prerelease: pre,
            draft: false,
            body: None,
            html_url: String::new(),
            assets: vec![Asset {
                name: "gachahub.exe".into(),
                browser_download_url: format!("https://example/{tag}"),
                size: 1,
                digest: None,
            }],
        }
    }

    #[test]
    fn picks_newest_matching_release() {
        let list = || {
            vec![
                rel("v0.15.0", false),
                rel("v0.16.0", false),
                rel("v0.16.1-pre.7", true),
                rel("v0.14.0", false),
            ]
        };
        assert_eq!(
            select_update(list(), "0.15.0", false, "gachahub.exe")
                .unwrap()
                .version,
            "0.16.0"
        );
        assert_eq!(
            select_update(list(), "0.15.0", true, "gachahub.exe")
                .unwrap()
                .version,
            "0.16.1-pre.7"
        );
        assert!(select_update(list(), "0.16.0", false, "gachahub.exe").is_none());
        // a pre-release of the running version is older than the final release
        assert_eq!(
            select_update(list(), "0.16.0-pre.3", false, "gachahub.exe")
                .unwrap()
                .version,
            "0.16.0"
        );
        // missing asset for this platform → ignored
        assert!(select_update(list(), "0.15.0", false, "gachahub-linux-x86_64.tar.gz").is_none());
    }
}
