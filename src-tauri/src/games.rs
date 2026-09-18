use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GameAssets {
    pub background: Option<String>,
    pub video: Option<String>,
    pub logo: Option<String>,
    pub icon: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Game {
    pub id: String,
    pub name: String,
    pub short_name: String,
    pub subtitle: String,
    pub publisher: String,
    pub provider: String,
    pub platforms: Vec<String>,
    pub install_modes: Vec<String>,
    pub status: String,
    pub mod_policy: String,
    pub accent: String,
    pub accent2: String,
    pub version: Option<String>,
    pub notes: Option<String>,
    pub xxmi_importer: Option<String>,
    pub assets: Option<GameAssets>,
}

#[derive(Debug, Clone)]
pub struct GameRegistry {
    pub games: Vec<Game>,
}

impl GameRegistry {
    pub fn builtin() -> Self {
        let raw = include_str!("../game-manifests/games.json");
        let games: Vec<Game> = serde_json::from_str(raw).expect("invalid built-in game manifest");
        Self { games }
    }
}
