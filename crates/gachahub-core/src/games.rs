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
    #[serde(default)]
    pub subtitle_de: Option<String>,
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
    #[serde(default)]
    pub notes_de: Option<String>,
    pub xxmi_importer: Option<String>,
    pub assets: Option<GameAssets>,
    /// Names under which the game appears in Steam (for detection / store search).
    #[serde(default)]
    pub steam_names: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct GameRegistry {
    pub games: Vec<Game>,
}

impl Game {
    /// Subtitle in the UI language (`german` = true → German if available).
    pub fn subtitle_for(&self, german: bool) -> &str {
        if german {
            self.subtitle_de.as_deref().unwrap_or(&self.subtitle)
        } else {
            &self.subtitle
        }
    }
    pub fn notes_for(&self, german: bool) -> Option<&str> {
        if german {
            self.notes_de.as_deref().or(self.notes.as_deref())
        } else {
            self.notes.as_deref()
        }
    }
}

impl GameRegistry {
    pub fn builtin() -> Self {
        let raw = include_str!("../game-manifests/games.json");
        let games: Vec<Game> = serde_json::from_str(raw).expect("invalid built-in game manifest");
        Self { games }
    }
}
