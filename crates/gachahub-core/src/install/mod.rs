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
        "hi3" => &["BH3.exe"],
        _ => &[],
    };
    // Fallback: the biggest plausible .exe is almost always the game binary.
    let mut fallback: Option<(u64, PathBuf)> = None;
    let mut stack = vec![(root.to_path_buf(), 0usize)];
    while let Some((dir, depth)) = stack.pop() {
        let Ok(entries) = std::fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                if depth < 5 {
                    stack.push((path, depth + 1));
                }
                continue;
            }
            let name = entry.file_name().to_string_lossy().into_owned();
            if preferred.iter().any(|p| p.eq_ignore_ascii_case(&name)) {
                return Some(path);
            }
            let lower = name.to_ascii_lowercase();
            let blocked = [
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
            ];
            if lower.ends_with(".exe") && !blocked.iter().any(|b| lower.contains(b)) {
                let size = entry.metadata().map(|m| m.len()).unwrap_or(0);
                if fallback.as_ref().is_none_or(|(best, _)| size > *best) {
                    fallback = Some((size, path));
                }
            }
        }
    }
    fallback.map(|(_, p)| p)
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
