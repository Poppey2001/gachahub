use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ModPolicy { Supported, Restricted, Disabled }

pub fn policy_for(game_id: &str) -> Option<ModPolicy> {
    Some(match game_id {
        "endfield" | "hsr" => ModPolicy::Restricted,
        "arknights" | "genshin" | "zzz" | "wuwa" | "gfl2" => ModPolicy::Supported,
        _ => return None,
    })
}

// Mod files will be staged outside the game directory first. Actual activation
// must go through a game-specific adapter. This prevents destructive overwrites
// and keeps restricted games in vanilla mode by default.
