//! Curated per-game compatibility presets (Linux runner hints, modding backend).

use serde::{Deserialize, Serialize};
use std::sync::OnceLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlatformCompat {
    pub state: String,
    pub label: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LinuxCompat {
    pub state: String,
    pub label: String,
    pub recommended_runtime: String,
    pub recommended_runner: String,
    #[serde(default)]
    pub runner_candidates: Vec<String>,
    #[serde(default)]
    pub blocked_runners: Vec<String>,
    #[serde(default)]
    pub launch_arguments: String,
    #[serde(default)]
    pub environment_variables: String,
    #[serde(default)]
    pub optional_environment_variables: Option<String>,
    #[serde(default)]
    pub notes: Vec<String>,
    #[serde(default)]
    pub xxmi_launch_arguments: Option<String>,
    #[serde(default)]
    pub xxmi_notes: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModdingCompat {
    pub backend: String,
    #[serde(default)]
    pub importer: Option<String>,
    pub label: String,
    pub note: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompatibilityPreset {
    pub game_id: String,
    pub checked_at: String,
    pub windows: PlatformCompat,
    pub linux: LinuxCompat,
    pub modding: ModdingCompat,
}

pub fn presets() -> &'static [CompatibilityPreset] {
    static PRESETS: OnceLock<Vec<CompatibilityPreset>> = OnceLock::new();
    PRESETS.get_or_init(|| {
        serde_json::from_str(include_str!("../game-manifests/compatibility.json"))
            .expect("invalid built-in compatibility manifest")
    })
}

pub fn preset_for(game_id: &str) -> Option<&'static CompatibilityPreset> {
    presets().iter().find(|p| p.game_id == game_id)
}

#[cfg(test)]
mod tests {
    #[test]
    fn manifests_parse_and_cover_every_game() {
        let games = crate::GameRegistry::builtin().games;
        assert!(!games.is_empty());
        for game in &games {
            assert!(super::preset_for(&game.id).is_some(), "missing compatibility preset for {}", game.id);
        }
    }
}
