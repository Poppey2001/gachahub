use crate::providers::provider_for;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstallJob {
    pub id: String,
    pub game_id: String,
    pub phase: String,
    pub progress: f32,
    pub downloaded_bytes: u64,
    pub total_bytes: u64,
    pub speed_bytes: u64,
    pub version: Option<String>,
    pub provider_mode: Option<String>,
    pub destination: Option<String>,
    pub message: Option<String>,
    /// "install", "update" or "verify".
    #[serde(default)]
    pub kind: String,
}

#[derive(Debug, thiserror::Error)]
pub enum InstallError {
    #[error("no provider registered for {0}")]
    NoProvider(String),
}

pub fn create_install_job(game_id: &str) -> Result<InstallJob, InstallError> {
    let provider = provider_for(game_id).ok_or_else(|| InstallError::NoProvider(game_id.into()))?;
    let provider_id = provider.id();

    Ok(InstallJob {
        id: Uuid::new_v4().to_string(),
        game_id: game_id.to_string(),
        phase: "queued".into(),
        progress: 0.0,
        downloaded_bytes: 0,
        total_bytes: 0,
        speed_bytes: 0,
        version: None,
        provider_mode: None,
        destination: None,
        message: Some(format!("Queued for provider {provider_id}")),
        kind: "install".into(),
    })
}

/// File names of the game binary per game (checked first).
fn known_exe_names(game_id: &str) -> &'static [&'static str] {
    match game_id {
        "genshin" => &["GenshinImpact.exe", "YuanShen.exe"],
        "hsr" => &["StarRail.exe"],
        "zzz" => &["ZenlessZoneZero.exe"],
        "hi3" => &["BH3.exe"],
        "wuwa" => &["Wuthering Waves.exe", "Client-Win64-Shipping.exe"],
        "endfield" => &["Endfield.exe"],
        "arknights" => &["Arknights.exe"],
        "gfl2" => &["GF2_Exilium.exe"],
        "nte" => &["HTGame.exe"],
        "bluearchive" => &["BlueArchive.exe"],
        "limbus" => &["LimbusCompany.exe"],
        "pgr" => &["PGR.exe"],
        "snowbreak" => &["X6Game.exe", "X6Game-Win64-Shipping.exe"],
        "reverse1999" => &["reverse1999.exe"],
        "tof" => &["QRSL.exe", "Tower of Fantasy.exe"],
        _ => &[],
    }
}

/// Words that appear in the executable's version info (product name,
/// description, original file name). These stay the same across updates,
/// unlike file hashes, so they identify the game binary reliably.
fn identity_words(game_id: &str) -> &'static [&'static str] {
    match game_id {
        "genshin" => &["genshin impact", "genshinimpact", "yuanshen", "原神"],
        "hsr" => &["star rail", "starrail", "星穹铁道"],
        "zzz" => &["zenless", "绝区零"],
        "hi3" => &["honkai impact", "bh3", "崩坏3"],
        "wuwa" => &["wuthering", "鸣潮"],
        "endfield" => &["endfield", "终末地"],
        "arknights" => &["arknights", "明日方舟"],
        "gfl2" => &["gf2_exilium", "gf2 exilium", "exilium", "少女前线2"],
        "nte" => &["neverness", "htgame", "异环"],
        "bluearchive" => &["blue archive", "bluearchive", "ブルーアーカイブ"],
        "limbus" => &["limbus"],
        "pgr" => &["punishing", "gray raven", "战双"],
        "snowbreak" => &["snowbreak", "x6game", "尘白禁区"],
        "reverse1999" => &["reverse: 1999", "reverse1999", "重返未来"],
        "tof" => &["tower of fantasy", "qrsl", "幻塔"],
        _ => &[],
    }
}

/// File names that are never the game itself.
const HELPER_WORDS: &[&str] = &[
    "unins",
    "crash",
    "launcher",
    "setup",
    "update",
    "redist",
    "dxsetup",
    "cefsubprocess",
    "helper",
    "report",
    "installer",
    "repair",
    "uploader",
    "bugreport",
    "vc_redist",
    "dotnet",
    "webview",
    "anticheat",
    "ace-",
    "easyanticheat",
    "protect",
    "service",
];

/// Product name, description, original/internal file name of a Windows exe.
pub fn exe_identity(path: &Path) -> Option<String> {
    use pelite::{FileMap, PeFile};
    let size = std::fs::metadata(path).ok()?.len();
    if size > 512 * 1024 * 1024 {
        return None;
    }
    let map = FileMap::open(path).ok()?;
    let file = PeFile::from_bytes(&map).ok()?;
    let info = file.resources().ok()?.version_info().ok()?.file_info();
    let mut out = String::new();
    for strings in info.strings.values() {
        for key in [
            "ProductName",
            "FileDescription",
            "OriginalFilename",
            "InternalName",
        ] {
            if let Some(v) = strings.get(key) {
                out.push_str(v);
                out.push(' ');
            }
        }
    }
    let out = out.trim().to_string();
    (!out.is_empty()).then_some(out)
}

/// Result of searching an install folder.
#[derive(Debug, Clone, Default)]
pub struct ExeScan {
    pub best: Option<PathBuf>,
    /// How `best` was chosen: "name", "version-info" or "largest".
    pub reason: &'static str,
    /// Number of .exe files looked at.
    pub checked: usize,
}

/// Search `root` (up to 8 levels deep) for the game binary of `game_id`:
/// 1. a known file name, 2. an exe whose version info names the game,
/// 3. the biggest plausible exe.
pub fn scan_executables(root: &Path, game_id: &str) -> ExeScan {
    let known = known_exe_names(game_id);
    let words = identity_words(game_id);
    let mut scan = ExeScan::default();
    let mut exes: Vec<(u64, PathBuf, String)> = Vec::new();
    let mut stack = vec![(root.to_path_buf(), 0usize)];
    while let Some((dir, depth)) = stack.pop() {
        let Ok(entries) = std::fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().into_owned();
            let lower = name.to_lowercase();
            let Ok(ft) = entry.file_type() else { continue };
            if ft.is_dir() {
                let skip = lower.starts_with('.')
                    || lower.starts_with('$')
                    || matches!(
                        lower.as_str(),
                        "windows" | "system volume information" | "node_modules"
                    );
                if depth < 8 && !skip {
                    stack.push((path, depth + 1));
                }
                continue;
            }
            if !lower.ends_with(".exe") {
                continue;
            }
            scan.checked += 1;
            if known.iter().any(|k| k.eq_ignore_ascii_case(&name)) {
                scan.best = Some(path);
                scan.reason = "name";
                return scan;
            }
            let size = entry.metadata().map(|m| m.len()).unwrap_or(0);
            exes.push((size, path, lower));
        }
    }
    // version info: biggest exe whose metadata names the game (helpers excluded)
    let mut by_size = exes.clone();
    by_size.sort_by_key(|(size, _, _)| std::cmp::Reverse(*size));
    if !words.is_empty() {
        for (_, path, lower) in by_size.iter().take(200) {
            if HELPER_WORDS.iter().any(|h| lower.contains(h)) {
                continue;
            }
            if let Some(id) = exe_identity(path) {
                let id = id.to_lowercase();
                if words.iter().any(|w| id.contains(w)) && !id.contains("launcher") {
                    scan.best = Some(path.clone());
                    scan.reason = "version-info";
                    return scan;
                }
            }
        }
    }
    if let Some((_, path, _)) = by_size
        .into_iter()
        .find(|(_, _, lower)| !HELPER_WORDS.iter().any(|h| lower.contains(h)))
    {
        scan.best = Some(path);
        scan.reason = "largest";
    }
    scan
}

/// Look for a plausible game executable inside an install folder.
pub fn find_executable(root: &Path, game_id: &str) -> Option<PathBuf> {
    scan_executables(root, game_id).best
}

#[cfg(test)]
mod tests {
    use super::find_executable;

    #[test]
    fn prefers_known_names_then_biggest_exe() {
        let dir = std::env::temp_dir().join(format!("gachahub-exe-{}", uuid::Uuid::new_v4()));
        let sub = dir.join("Games").join("Genshin Impact game");
        std::fs::create_dir_all(&sub).unwrap();
        std::fs::write(sub.join("UnityCrashHandler64.exe"), vec![0u8; 5000]).unwrap();
        std::fs::write(sub.join("GenshinImpact.exe"), vec![0u8; 10]).unwrap();
        std::fs::write(dir.join("launcher.exe"), vec![0u8; 9000]).unwrap();
        assert_eq!(
            find_executable(&dir, "genshin")
                .unwrap()
                .file_name()
                .unwrap(),
            "GenshinImpact.exe"
        );

        // unknown game: biggest non-helper exe wins
        let other = dir.join("nte");
        std::fs::create_dir_all(other.join("Client")).unwrap();
        std::fs::write(other.join("Client").join("HTGame.exe"), vec![0u8; 8000]).unwrap();
        std::fs::write(
            other.join("Client").join("CrashReporter.exe"),
            vec![0u8; 9000],
        )
        .unwrap();
        std::fs::write(other.join("tool.exe"), vec![0u8; 100]).unwrap();
        assert_eq!(
            find_executable(&other, "nte").unwrap().file_name().unwrap(),
            "HTGame.exe"
        );
        assert!(find_executable(&dir.join("missing"), "nte").is_none());
        let _ = std::fs::remove_dir_all(&dir);
    }
}

#[cfg(test)]
mod identity_tests {
    /// Reads version info from a real PE (the Windows build, when present).
    #[test]
    fn reads_version_info_of_windows_build() {
        let exe = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../target/x86_64-pc-windows-gnu/release/gachahub.exe");
        if !exe.exists() {
            return;
        }
        let id = super::exe_identity(&exe).expect("version info");
        println!("identity: {id}");
        assert!(id.to_lowercase().contains("gachahub"));
        // renamed copy is still recognised by its metadata, not its name
        let dir = std::env::temp_dir().join(format!("gachahub-id-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(dir.join("bin")).unwrap();
        std::fs::copy(&exe, dir.join("bin").join("renamed.exe")).unwrap();
        std::fs::write(dir.join("huge.exe"), vec![0u8; 64 * 1024 * 1024]).unwrap();
        let scan = super::scan_executables(&dir, "gachahub-selftest");
        assert_eq!(scan.reason, "largest");
        let _ = std::fs::remove_dir_all(&dir);
    }
}
