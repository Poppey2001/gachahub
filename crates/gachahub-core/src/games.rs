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
    /// Official PC editions (publisher launchers) GachaHub can install.
    #[serde(default)]
    pub official_launchers: Vec<OfficialLauncher>,
}

/// One official PC edition of a game, e.g. a regional publisher's launcher.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OfficialLauncher {
    pub id: String,
    pub label: String,
    #[serde(default)]
    pub label_de: Option<String>,
    /// Servers / regions this edition plays on.
    #[serde(default)]
    pub region: Option<String>,
    #[serde(default)]
    pub region_de: Option<String>,
    /// Direct installer downloads, tried in order.
    #[serde(default)]
    pub installer_urls: Vec<String>,
    /// Official website (fallback when no installer download works).
    pub website: String,
    /// Folder / program names the finished installation shows up under.
    #[serde(default)]
    pub detect_names: Vec<String>,
}

impl OfficialLauncher {
    pub fn label_for(&self, german: bool) -> &str {
        if german {
            self.label_de.as_deref().unwrap_or(&self.label)
        } else {
            &self.label
        }
    }
    pub fn region_for(&self, german: bool) -> &str {
        if german {
            self.region_de
                .as_deref()
                .or(self.region.as_deref())
                .unwrap_or("")
        } else {
            self.region.as_deref().unwrap_or("")
        }
    }
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
