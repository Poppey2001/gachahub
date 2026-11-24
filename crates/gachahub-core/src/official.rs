//! Install games through their official (publisher) launcher.
//!
//! Some games ship several regional PC builds from different publishers, e.g.
//! GIRLS' FRONTLINE 2 (Darkwinter for the global server, Haoplay for Asia).
//! GachaHub downloads the official installer of the chosen edition, runs it,
//! and then watches for the finished installation so the game appears in the
//! library without manual importing.

use crate::{games::OfficialLauncher, USER_AGENT};
use futures::StreamExt;
use std::path::{Path, PathBuf};
use tokio::io::AsyncWriteExt;

/// Download the edition's installer into `dir`. Tries every installer URL in
/// order and returns the first that works. `progress(downloaded, total)`.
pub async fn download_installer(
    launcher: &OfficialLauncher,
    dir: &Path,
    progress: impl Fn(u64, u64),
) -> Result<PathBuf, String> {
    if launcher.installer_urls.is_empty() {
        return Err("no installer url".into());
    }
    tokio::fs::create_dir_all(dir)
        .await
        .map_err(|e| format!("{}: {e}", dir.display()))?;
    let client = reqwest::Client::builder()
        .user_agent(USER_AGENT)
        .build()
        .map_err(|e| e.to_string())?;
    let mut last_err = String::new();
    for url in &launcher.installer_urls {
        match fetch(&client, url, dir, &progress).await {
            Ok(path) => return Ok(path),
            Err(e) => last_err = format!("{url}: {e}"),
        }
    }
    Err(last_err)
}

async fn fetch(
    client: &reqwest::Client,
    url: &str,
    dir: &Path,
    progress: &impl Fn(u64, u64),
) -> Result<PathBuf, String> {
    let resp = client
        .get(url)
        .send()
        .await
        .and_then(|r| r.error_for_status())
        .map_err(|e| e.to_string())?;
    let total = resp.content_length().unwrap_or(0);
    let name = installer_file_name(url);
    let path = dir.join(&name);
    let part = dir.join(format!("{name}.part"));
    let mut file = tokio::fs::File::create(&part)
        .await
        .map_err(|e| e.to_string())?;
    let mut done = 0u64;
    let mut stream = resp.bytes_stream();
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| e.to_string())?;
        file.write_all(&chunk).await.map_err(|e| e.to_string())?;
        done += chunk.len() as u64;
        progress(done, total);
    }
    file.flush().await.map_err(|e| e.to_string())?;
    drop(file);
    if done < 1024 || (total > 0 && done != total) {
        let _ = tokio::fs::remove_file(&part).await;
        return Err(format!("incomplete download ({done} bytes)"));
    }
    tokio::fs::rename(&part, &path)
        .await
        .map_err(|e| e.to_string())?;
    Ok(path)
}

fn installer_file_name(url: &str) -> String {
    let name = url
        .split(['?', '#'])
        .next()
        .unwrap_or(url)
        .rsplit('/')
        .next()
        .unwrap_or("")
        .trim();
    let safe: String = name
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-'))
        .collect();
    if safe.is_empty() || !safe.to_ascii_lowercase().ends_with(".exe") {
        "installer.exe".into()
    } else {
        safe
    }
}

fn normalize(s: &str) -> String {
    s.chars()
        .filter(|c| c.is_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect()
}

fn name_matches(candidate: &str, names: &[String]) -> bool {
    let c = normalize(candidate);
    !c.is_empty()
        && names.iter().any(|n| {
            let n = normalize(n);
            !n.is_empty() && (c == n || c.contains(&n))
        })
}

/// Find an installed copy of the game. Returns (install folder, game exe).
///
/// Looks at the Windows uninstall registry first, then scans the usual install
/// folders (all drives on Windows; on Linux the given Wine prefix).
pub fn detect(
    game_id: &str,
    names: &[String],
    prefix: Option<&Path>,
) -> Option<(PathBuf, PathBuf)> {
    if names.is_empty() {
        return None;
    }
    let mut roots: Vec<PathBuf> = registry_install_dirs(names);
    roots.extend(candidate_dirs(names, prefix));
    for root in roots {
        if let Some(exe) = crate::install::find_executable(&root, game_id) {
            // a plain launcher folder (only the launcher exe) is not the game yet
            if is_game_exe(&exe) {
                return Some((root, exe));
            }
        }
    }
    None
}

/// Only accept the real game binary, not a launcher stub.
fn is_game_exe(exe: &Path) -> bool {
    let name = exe
        .file_name()
        .map(|n| n.to_string_lossy().to_ascii_lowercase())
        .unwrap_or_default();
    if name == "gf2_exilium.exe" {
        return true;
    }
    let size = std::fs::metadata(exe).map(|m| m.len()).unwrap_or(0);
    size > 20 * 1024 * 1024 || exe.parent().is_some_and(has_unity_or_unreal_data)
}

fn has_unity_or_unreal_data(dir: &Path) -> bool {
    std::fs::read_dir(dir).is_ok_and(|entries| {
        entries.flatten().any(|e| {
            let n = e.file_name().to_string_lossy().to_ascii_lowercase();
            n.ends_with("_data") || n == "engine" || n == "unityplayer.dll"
        })
    })
}

/// Folders below the usual install locations whose name matches the game.
fn candidate_dirs(names: &[String], prefix: Option<&Path>) -> Vec<PathBuf> {
    let mut bases: Vec<PathBuf> = Vec::new();
    #[cfg(windows)]
    {
        let _ = prefix;
        for drive in b'C'..=b'Z' {
            let root = PathBuf::from(format!("{}:\\", drive as char));
            if !root.exists() {
                continue;
            }
            bases.push(root.join("Program Files"));
            bases.push(root.join("Program Files (x86)"));
            bases.push(root.join("Games"));
            bases.push(root.clone());
        }
        if let Some(local) = std::env::var_os("LOCALAPPDATA") {
            bases.push(PathBuf::from(&local));
            bases.push(PathBuf::from(local).join("Programs"));
        }
    }
    #[cfg(not(windows))]
    if let Some(prefix) = prefix {
        let c = prefix.join("drive_c");
        bases.push(c.join("Program Files"));
        bases.push(c.join("Program Files (x86)"));
        bases.push(c.join("Games"));
        bases.push(c.clone());
        if let Ok(users) = std::fs::read_dir(c.join("users")) {
            for u in users.flatten() {
                bases.push(u.path().join("AppData").join("Local"));
                bases.push(u.path().join("AppData").join("Local").join("Programs"));
            }
        }
    }
    let mut out = Vec::new();
    for base in bases {
        out.extend(matching_children(&base, names, 2));
    }
    out
}

/// Sub-folders (up to `depth` levels) whose name matches one of `names`.
fn matching_children(base: &Path, names: &[String], depth: usize) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let Ok(entries) = std::fs::read_dir(base) else {
        return out;
    };
    for e in entries.flatten() {
        let path = e.path();
        if !path.is_dir() {
            continue;
        }
        let n = e.file_name().to_string_lossy().into_owned();
        if name_matches(&n, names) {
            out.push(path);
        } else if depth > 1 && !n.starts_with('.') && !n.eq_ignore_ascii_case("windows") {
            out.extend(matching_children(&path, names, depth - 1));
        }
    }
    out
}

#[cfg(windows)]
fn registry_install_dirs(names: &[String]) -> Vec<PathBuf> {
    use winreg::{enums::*, RegKey};
    let mut out = Vec::new();
    let hives = [
        (
            HKEY_CURRENT_USER,
            r"Software\Microsoft\Windows\CurrentVersion\Uninstall",
        ),
        (
            HKEY_LOCAL_MACHINE,
            r"Software\Microsoft\Windows\CurrentVersion\Uninstall",
        ),
        (
            HKEY_LOCAL_MACHINE,
            r"Software\WOW6432Node\Microsoft\Windows\CurrentVersion\Uninstall",
        ),
    ];
    for (hive, path) in hives {
        let Ok(key) = RegKey::predef(hive).open_subkey(path) else {
            continue;
        };
        for sub in key.enum_keys().flatten() {
            let Ok(app) = key.open_subkey(&sub) else {
                continue;
            };
            let name: String = app.get_value("DisplayName").unwrap_or_default();
            if !name_matches(&name, names) {
                continue;
            }
            let loc: String = app.get_value("InstallLocation").unwrap_or_default();
            let loc = loc.trim().trim_matches('"');
            if !loc.is_empty() {
                out.push(PathBuf::from(loc));
                continue;
            }
            let icon: String = app.get_value("DisplayIcon").unwrap_or_default();
            let icon = icon
                .split(',')
                .next()
                .unwrap_or("")
                .trim()
                .trim_matches('"');
            if let Some(dir) = Path::new(icon)
                .parent()
                .filter(|p| !p.as_os_str().is_empty())
            {
                out.push(dir.to_path_buf());
            }
        }
    }
    out
}

#[cfg(not(windows))]
fn registry_install_dirs(_names: &[String]) -> Vec<PathBuf> {
    Vec::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn file_names_are_sanitised() {
        assert_eq!(
            installer_file_name("https://x.y/pc/GF2_v1.0.3.exe?t=1"),
            "GF2_v1.0.3.exe"
        );
        assert_eq!(installer_file_name("https://x.y/download"), "installer.exe");
        assert_eq!(installer_file_name("https://x.y/../evil/..exe"), "..exe");
    }

    #[test]
    fn detects_game_in_matching_folder() {
        let tmp = std::env::temp_dir().join(format!("gachahub-official-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&tmp);
        let prefix = tmp.join("prefix");
        let game = prefix.join("drive_c/Program Files/GF2 Exilium/Game");
        std::fs::create_dir_all(&game).unwrap();
        // only the launcher so far → not detected
        std::fs::write(game.parent().unwrap().join("GF2_Launcher.exe"), b"x").unwrap();
        let names = vec!["GF2 Exilium".to_string(), "Girls Frontline 2".to_string()];
        assert!(detect("gfl2", &names, Some(&prefix)).is_none());
        std::fs::write(game.join("GF2_Exilium.exe"), b"x").unwrap();
        let (root, exe) = detect("gfl2", &names, Some(&prefix)).expect("detected");
        assert!(root.ends_with("GF2 Exilium"));
        assert!(exe.ends_with("GF2_Exilium.exe"));
        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[tokio::test]
    async fn downloads_first_working_installer() {
        use std::io::{Read, Write};
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        std::thread::spawn(move || {
            for stream in listener.incoming().take(2) {
                let mut s = stream.unwrap();
                let mut buf = [0u8; 2048];
                let n = s.read(&mut buf).unwrap();
                let req = String::from_utf8_lossy(&buf[..n]).to_string();
                if req.starts_with("GET /missing") {
                    let _ = s.write_all(
                        b"HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\nConnection: close\r\n\r\n",
                    );
                } else {
                    let body = vec![7u8; 4096];
                    let _ = s.write_all(
                        format!(
                            "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                            body.len()
                        )
                        .as_bytes(),
                    );
                    let _ = s.write_all(&body);
                }
            }
        });
        let launcher = OfficialLauncher {
            id: "test".into(),
            label: "Test".into(),
            label_de: None,
            region: None,
            region_de: None,
            installer_urls: vec![
                format!("http://127.0.0.1:{port}/missing/setup.exe"),
                format!("http://127.0.0.1:{port}/pc/GF2_v9.exe"),
            ],
            website: String::new(),
            detect_names: vec![],
        };
        let dir = std::env::temp_dir().join(format!("gachahub-inst-{}", std::process::id()));
        let path = download_installer(&launcher, &dir, |_, _| {})
            .await
            .unwrap();
        assert!(path.ends_with("GF2_v9.exe"));
        assert_eq!(std::fs::metadata(&path).unwrap().len(), 4096);
        let _ = std::fs::remove_dir_all(&dir);
    }
}
