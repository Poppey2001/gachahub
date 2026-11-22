//! Persistent launcher configuration.
//!
//! Port of the former `src/lib/settings.ts` + zustand stores. Instead of the web
//! view's `localStorage`, everything is stored as JSON in the platform config
//! directory (e.g. `~/.config/gachahub/config.json` on Linux).

use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, HashMap},
    fs,
    path::PathBuf,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum RuntimeKind {
    #[default]
    Auto,
    Native,
    Wine,
    Proton,
    GeProton,
    Umu,
}

impl RuntimeKind {
    pub const ALL: [RuntimeKind; 6] = [
        RuntimeKind::Auto,
        RuntimeKind::Native,
        RuntimeKind::Wine,
        RuntimeKind::Proton,
        RuntimeKind::GeProton,
        RuntimeKind::Umu,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            RuntimeKind::Auto => "auto",
            RuntimeKind::Native => "native",
            RuntimeKind::Wine => "wine",
            RuntimeKind::Proton => "proton",
            RuntimeKind::GeProton => "ge-proton",
            RuntimeKind::Umu => "umu",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            RuntimeKind::Auto => "Automatisch",
            RuntimeKind::Native => "Nativ",
            RuntimeKind::Wine => "Wine",
            RuntimeKind::Proton => "Proton",
            RuntimeKind::GeProton => "GE-Proton",
            RuntimeKind::Umu => "UMU",
        }
    }

    pub fn parse(raw: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|r| r.as_str().eq_ignore_ascii_case(raw.trim()))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum ModDeploymentMode {
    #[default]
    Copy,
    Symlink,
}

impl ModDeploymentMode {
    pub fn as_str(self) -> &'static str {
        match self {
            ModDeploymentMode::Copy => "copy",
            ModDeploymentMode::Symlink => "symlink",
        }
    }
}

/// Which GachaHub builds the self-updater offers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum AppChannel {
    #[default]
    Stable,
    Prerelease,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum Language {
    #[default]
    De,
    En,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct GlobalSettings {
    pub language: Language,
    pub animated_backgrounds: bool,
    pub background_dim: u8,
    pub default_install_path: String,
    pub parallel_downloads: u8,
    pub bandwidth_limit_mbps: Option<u32>,
    pub default_runtime: RuntimeKind,
    pub default_runner_version: String,
    pub default_runner_path: String,
    pub prefix_base_path: String,
    pub default_gamescope_enabled: bool,
    pub default_game_mode_enabled: bool,
    pub default_mango_hud_enabled: bool,
    pub default_prevent_sleep: bool,
    /// Check for game updates on start and install them automatically.
    pub auto_update: bool,
    /// Voice pack for HoYoverse games (`en-us`, `ja-jp`, `zh-cn`, `ko-kr`).
    pub voice_language: String,
    pub mod_root_path: String,
    pub default_mod_deployment_mode: ModDeploymentMode,
    pub xxmi_install_path: String,
    pub xxmi_launcher_path: String,
    pub xxmi_no_gui: bool,
    pub xxmi_auto_update: bool,
    /// Look for new GachaHub versions on start.
    pub app_update_check: bool,
    pub app_update_channel: AppChannel,
}

impl Default for GlobalSettings {
    fn default() -> Self {
        Self {
            language: Language::De,
            animated_backgrounds: true,
            background_dim: 58,
            default_install_path: "~/Games/GachaHub/Games".into(),
            parallel_downloads: 8,
            bandwidth_limit_mbps: None,
            default_runtime: if cfg!(windows) {
                RuntimeKind::Native
            } else {
                RuntimeKind::Auto
            },
            default_runner_version: if cfg!(windows) { "" } else { "GE-Proton" }.into(),
            default_runner_path: String::new(),
            prefix_base_path: if cfg!(windows) {
                "~/AppData/Local/GachaHub/prefixes"
            } else {
                "~/.local/share/gachahub/prefixes"
            }
            .into(),
            default_gamescope_enabled: false,
            default_game_mode_enabled: false,
            default_mango_hud_enabled: false,
            default_prevent_sleep: true,
            auto_update: true,
            voice_language: "en-us".into(),
            mod_root_path: if cfg!(windows) {
                "~/AppData/Local/GachaHub/mods"
            } else {
                "~/.local/share/gachahub/mods"
            }
            .into(),
            default_mod_deployment_mode: ModDeploymentMode::Copy,
            xxmi_install_path: String::new(),
            xxmi_launcher_path: String::new(),
            xxmi_no_gui: true,
            xxmi_auto_update: true,
            app_update_check: true,
            app_update_channel: AppChannel::Stable,
        }
    }
}

/// Per-game overrides. `None` means "inherit from global".
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct GameSettings {
    pub override_enabled: bool,
    pub install_path: Option<String>,
    pub executable_path: Option<String>,
    pub installation_detected_at: Option<String>,
    pub runtime: Option<RuntimeKind>,
    pub runner_version: Option<String>,
    pub runner_path: Option<String>,
    pub prefix_path: Option<String>,
    pub launch_arguments: Option<String>,
    pub environment_variables: Option<String>,
    pub gamescope_enabled: Option<bool>,
    pub game_mode_enabled: Option<bool>,
    pub mango_hud_enabled: Option<bool>,
    pub prevent_sleep: Option<bool>,
    pub auto_update: Option<bool>,
    pub mod_library_path: Option<String>,
    pub active_mods_path: Option<String>,
    pub mod_deployment_mode: Option<ModDeploymentMode>,
    pub xxmi_enabled: bool,
    pub post_install_profile: Option<String>,
    pub post_install_applied_at: Option<String>,
    /// Accumulated play time in seconds (tracked while the launched process runs).
    pub playtime_seconds: u64,
    pub last_played_unix: Option<u64>,
}

impl Default for GameSettings {
    fn default() -> Self {
        Self {
            override_enabled: true,
            install_path: None,
            executable_path: None,
            installation_detected_at: None,
            runtime: None,
            runner_version: None,
            runner_path: None,
            prefix_path: None,
            launch_arguments: None,
            environment_variables: None,
            gamescope_enabled: None,
            game_mode_enabled: None,
            mango_hud_enabled: None,
            prevent_sleep: None,
            auto_update: None,
            mod_library_path: None,
            active_mods_path: None,
            mod_deployment_mode: None,
            xxmi_enabled: false,
            post_install_profile: None,
            post_install_applied_at: None,
            playtime_seconds: 0,
            last_played_unix: None,
        }
    }
}

impl GameSettings {
    /// Apply values detected by a post-install profile. Only unset fields are
    /// filled, so manual values always win.
    pub fn apply_detected(&mut self, detected: &crate::postinstall::PostInstallSettings) {
        fn fill_str(slot: &mut Option<String>, value: &str) {
            if slot.is_none() && !value.trim().is_empty() {
                *slot = Some(value.to_string());
            }
        }
        fn fill_bool(slot: &mut Option<bool>, value: bool) {
            if slot.is_none() {
                *slot = Some(value);
            }
        }
        self.override_enabled = true;
        fill_str(&mut self.install_path, &detected.install_path);
        if self.runtime.is_none() {
            self.runtime = RuntimeKind::parse(&detected.runtime);
        }
        fill_str(&mut self.runner_version, &detected.runner_version);
        fill_str(&mut self.runner_path, &detected.runner_path);
        fill_str(&mut self.prefix_path, &detected.prefix_path);
        fill_str(&mut self.launch_arguments, &detected.launch_arguments);
        fill_str(
            &mut self.environment_variables,
            &detected.environment_variables,
        );
        fill_bool(&mut self.gamescope_enabled, detected.gamescope_enabled);
        fill_bool(&mut self.game_mode_enabled, detected.game_mode_enabled);
        fill_bool(&mut self.mango_hud_enabled, detected.mango_hud_enabled);
        fill_bool(&mut self.prevent_sleep, detected.prevent_sleep);
        fill_bool(&mut self.auto_update, detected.auto_update);
        self.post_install_profile = Some(detected.post_install_profile.clone());
        self.post_install_applied_at = Some(detected.post_install_applied_at.clone());
    }
}

/// Effective settings after merging per-game overrides over global defaults.
#[derive(Debug, Clone)]
pub struct ResolvedGameSettings {
    pub install_path: String,
    pub executable_path: String,
    pub runtime: RuntimeKind,
    pub runner_version: String,
    pub runner_path: String,
    pub prefix_path: String,
    pub launch_arguments: String,
    pub environment_variables: HashMap<String, String>,
    pub gamescope_enabled: bool,
    pub game_mode_enabled: bool,
    pub mango_hud_enabled: bool,
    pub prevent_sleep: bool,
    pub mod_library_path: String,
    pub active_mods_path: String,
    pub mod_deployment_mode: ModDeploymentMode,
    pub xxmi_enabled: bool,
    pub auto_update: bool,
}

pub fn parse_environment_variables(value: &str) -> HashMap<String, String> {
    value
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .filter_map(|line| line.split_once('='))
        .filter_map(|(k, v)| {
            let k = k.trim();
            (!k.is_empty()).then(|| (k.to_string(), v.trim().to_string()))
        })
        .collect()
}

pub fn resolve(
    game_id: &str,
    global: &GlobalSettings,
    game: Option<&GameSettings>,
) -> ResolvedGameSettings {
    let over = game.filter(|g| g.override_enabled);
    let pick_str = |f: fn(&GameSettings) -> &Option<String>, fallback: String| -> String {
        over.and_then(|g| f(g).clone())
            .filter(|v| !v.is_empty())
            .unwrap_or(fallback)
    };
    let pick_bool = |f: fn(&GameSettings) -> Option<bool>, fallback: bool| -> bool {
        over.and_then(f).unwrap_or(fallback)
    };

    ResolvedGameSettings {
        install_path: pick_str(
            |g| &g.install_path,
            format!("{}/{game_id}", global.default_install_path),
        ),
        executable_path: pick_str(|g| &g.executable_path, String::new()),
        runtime: over
            .and_then(|g| g.runtime)
            .unwrap_or(global.default_runtime),
        runner_version: pick_str(|g| &g.runner_version, global.default_runner_version.clone()),
        runner_path: over
            .and_then(|g| g.runner_path.clone())
            .unwrap_or_else(|| global.default_runner_path.clone()),
        prefix_path: pick_str(
            |g| &g.prefix_path,
            format!("{}/{game_id}", global.prefix_base_path),
        ),
        launch_arguments: pick_str(|g| &g.launch_arguments, String::new()),
        environment_variables: parse_environment_variables(&pick_str(
            |g| &g.environment_variables,
            String::new(),
        )),
        gamescope_enabled: pick_bool(|g| g.gamescope_enabled, global.default_gamescope_enabled),
        game_mode_enabled: pick_bool(|g| g.game_mode_enabled, global.default_game_mode_enabled),
        mango_hud_enabled: pick_bool(|g| g.mango_hud_enabled, global.default_mango_hud_enabled),
        prevent_sleep: pick_bool(|g| g.prevent_sleep, global.default_prevent_sleep),
        mod_library_path: pick_str(
            |g| &g.mod_library_path,
            format!("{}/{game_id}/library", global.mod_root_path),
        ),
        active_mods_path: pick_str(|g| &g.active_mods_path, String::new()),
        mod_deployment_mode: over
            .and_then(|g| g.mod_deployment_mode)
            .unwrap_or(global.default_mod_deployment_mode),
        // XXMI is a per-game launch toggle independent from the override switch.
        xxmi_enabled: game.map(|g| g.xxmi_enabled).unwrap_or(false),
        auto_update: pick_bool(|g| g.auto_update, global.auto_update),
    }
}

/// Everything the launcher persists between sessions.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Config {
    pub global: GlobalSettings,
    pub games: BTreeMap<String, GameSettings>,
    /// Games pinned to the sidebar, in display order.
    pub library: Vec<String>,
    pub selected_game: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            global: GlobalSettings::default(),
            games: BTreeMap::new(),
            library: vec!["genshin".into()],
            selected_game: "genshin".into(),
        }
    }
}

impl Config {
    pub fn path() -> PathBuf {
        directories::ProjectDirs::from("", "", "gachahub")
            .map(|dirs| dirs.config_dir().to_path_buf())
            .unwrap_or_else(|| PathBuf::from(".gachahub"))
            .join("config.json")
    }

    pub fn load() -> Self {
        let path = Self::path();
        fs::read_to_string(&path)
            .ok()
            .and_then(|raw| serde_json::from_str(&raw).ok())
            .unwrap_or_default()
    }

    pub fn save(&self) -> Result<(), String> {
        let path = Self::path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| format!("cannot create config dir: {e}"))?;
        }
        let raw = serde_json::to_string_pretty(self).map_err(|e| e.to_string())?;
        let tmp = path.with_extension("json.tmp");
        fs::write(&tmp, raw).map_err(|e| format!("cannot write config: {e}"))?;
        fs::rename(&tmp, &path).map_err(|e| format!("cannot replace config: {e}"))
    }

    pub fn game_mut(&mut self, game_id: &str) -> &mut GameSettings {
        self.games.entry(game_id.to_string()).or_default()
    }

    pub fn resolved(&self, game_id: &str) -> ResolvedGameSettings {
        resolve(game_id, &self.global, self.games.get(game_id))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn env_parsing_skips_comments_and_garbage() {
        let env = parse_environment_variables("# c\nA=1\n  B = two words \nbroken\n=x\n");
        assert_eq!(env.get("A").map(String::as_str), Some("1"));
        assert_eq!(env.get("B").map(String::as_str), Some("two words"));
        assert_eq!(env.len(), 2);
    }

    #[test]
    fn overrides_win_only_when_enabled() {
        let global = GlobalSettings::default();
        let mut game = GameSettings {
            install_path: Some("/x".into()),
            ..Default::default()
        };
        assert_eq!(resolve("genshin", &global, Some(&game)).install_path, "/x");
        game.override_enabled = false;
        assert_eq!(
            resolve("genshin", &global, Some(&game)).install_path,
            "~/Games/GachaHub/Games/genshin"
        );
    }

    #[test]
    fn config_roundtrip_tolerates_missing_fields() {
        let cfg: Config = serde_json::from_str(r#"{"library":["zzz"]}"#).unwrap();
        assert_eq!(cfg.library, vec!["zzz".to_string()]);
        assert_eq!(cfg.global.background_dim, 58);
    }
}
