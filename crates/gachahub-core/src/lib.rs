//! GachaHub core library.
//!
//! Everything that is not UI lives here: the game registry, provider metadata,
//! the resumable download manager, Wine/Proton runner management, the mod
//! manager, XXMI integration, post-install detection and persistent settings.

pub mod compat;
pub mod download_manager;
pub mod events;
pub mod exe_icon;
pub mod games;
pub mod install;
pub mod launcher;
pub mod mod_manager;
pub mod mods;
pub mod platform;
pub mod postinstall;
pub mod providers;
pub mod runners;
pub mod settings;
pub mod sophon;
pub mod xxmi;

pub use events::{CoreEvent, EventSink};
pub use games::{Game, GameRegistry};

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const USER_AGENT: &str = concat!("GachaHub/", env!("CARGO_PKG_VERSION"));

/// Expand a leading `~` to the user's home directory.
pub fn expand_user(raw: &str) -> std::path::PathBuf {
    let raw = raw.trim();
    if raw == "~" || raw.starts_with("~/") || raw.starts_with("~\\") {
        if let Some(home) = std::env::var_os("HOME").or_else(|| std::env::var_os("USERPROFILE")) {
            let home = std::path::PathBuf::from(home);
            return if raw.len() == 1 {
                home
            } else {
                home.join(&raw[2..])
            };
        }
    }
    std::path::PathBuf::from(raw)
}
