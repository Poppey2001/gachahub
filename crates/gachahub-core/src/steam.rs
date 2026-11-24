//! Steam integration: find installed Steam games locally and launch them through Steam.
//!
//! Nothing is fetched from the network. Steam's own files are read:
//! `steamapps/libraryfolders.vdf` lists all library folders, and every installed
//! game has a `steamapps/appmanifest_<appid>.acf` with its name and install dir.

use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

#[derive(Debug, Clone, PartialEq)]
pub struct SteamApp {
    pub app_id: String,
    pub name: String,
    /// Absolute path of the game's install folder.
    pub install_dir: PathBuf,
}

// ------------------------------------------------------------------ VDF parsing

#[derive(Debug, Clone, PartialEq)]
pub enum Vdf {
    Str(String),
    Map(Vec<(String, Vdf)>),
}

impl Vdf {
    pub fn get(&self, key: &str) -> Option<&Vdf> {
        match self {
            Vdf::Map(entries) => entries
                .iter()
                .find(|(k, _)| k.eq_ignore_ascii_case(key))
                .map(|(_, v)| v),
            Vdf::Str(_) => None,
        }
    }
    pub fn str(&self, key: &str) -> Option<&str> {
        match self.get(key) {
            Some(Vdf::Str(s)) => Some(s),
            _ => None,
        }
    }
    pub fn entries(&self) -> &[(String, Vdf)] {
        match self {
            Vdf::Map(e) => e,
            Vdf::Str(_) => &[],
        }
    }
}

fn tokenize(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut chars = text.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '"' => {
                let mut s = String::new();
                while let Some(c) = chars.next() {
                    match c {
                        '\\' => {
                            if let Some(n) = chars.next() {
                                s.push(match n {
                                    'n' => '\n',
                                    't' => '\t',
                                    other => other,
                                });
                            }
                        }
                        '"' => break,
                        other => s.push(other),
                    }
                }
                out.push(format!("\"{s}"));
            }
            '{' | '}' => out.push(c.to_string()),
            '/' if chars.peek() == Some(&'/') => {
                for c in chars.by_ref() {
                    if c == '\n' {
                        break;
                    }
                }
            }
            _ => {}
        }
    }
    out
}

/// Parse Valve's KeyValues text format (`"key" "value"` / `"key" { … }`).
pub fn parse_vdf(text: &str) -> Vdf {
    fn parse_map(tokens: &[String], pos: &mut usize) -> Vdf {
        let mut entries = Vec::new();
        while *pos < tokens.len() {
            let tok = &tokens[*pos];
            if tok == "}" {
                *pos += 1;
                break;
            }
            let Some(key) = tok.strip_prefix('"') else {
                *pos += 1;
                continue;
            };
            *pos += 1;
            let Some(next) = tokens.get(*pos) else { break };
            if next == "{" {
                *pos += 1;
                entries.push((key.to_string(), parse_map(tokens, pos)));
            } else if let Some(value) = next.strip_prefix('"') {
                *pos += 1;
                entries.push((key.to_string(), Vdf::Str(value.to_string())));
            }
        }
        Vdf::Map(entries)
    }
    let tokens = tokenize(text);
    let mut pos = 0;
    parse_map(&tokens, &mut pos)
}

// ------------------------------------------------------------------ discovery

/// Typical Steam installation folders on this platform.
pub fn steam_roots() -> Vec<PathBuf> {
    let mut roots = Vec::new();
    if cfg!(windows) {
        for var in ["ProgramFiles(x86)", "ProgramFiles"] {
            if let Some(p) = std::env::var_os(var) {
                roots.push(PathBuf::from(p).join("Steam"));
            }
        }
        for drive in ["C", "D", "E", "F"] {
            roots.push(PathBuf::from(format!("{drive}:\\Steam")));
            roots.push(PathBuf::from(format!("{drive}:\\SteamLibrary")));
        }
    } else {
        let home = crate::expand_user("~");
        roots.push(home.join(".steam").join("steam"));
        roots.push(home.join(".local").join("share").join("Steam"));
        roots.push(home.join(".var/app/com.valvesoftware.Steam/.local/share/Steam"));
        roots.push(home.join("snap/steam/common/.local/share/Steam"));
    }
    roots
}

/// All Steam library folders (each containing `steamapps/`).
pub fn library_folders(roots: &[PathBuf]) -> Vec<PathBuf> {
    let mut libs: Vec<PathBuf> = Vec::new();
    let mut push = |p: PathBuf| {
        if p.join("steamapps").is_dir() {
            let canon = p.canonicalize().unwrap_or(p);
            if !libs.contains(&canon) {
                libs.push(canon);
            }
        }
    };
    for root in roots {
        push(root.clone());
        let vdf_path = root.join("steamapps").join("libraryfolders.vdf");
        let Ok(text) = std::fs::read_to_string(&vdf_path) else {
            continue;
        };
        let vdf = parse_vdf(&text);
        let folders = vdf.get("libraryfolders").unwrap_or(&vdf);
        for (_, entry) in folders.entries() {
            // new format: "0" { "path" "…" }, old format: "1" "D:\\SteamLibrary"
            match entry {
                Vdf::Map(_) => {
                    if let Some(p) = entry.str("path") {
                        push(PathBuf::from(p));
                    }
                }
                Vdf::Str(p) if p.contains('/') || p.contains('\\') => push(PathBuf::from(p)),
                _ => {}
            }
        }
    }
    libs
}

/// Installed Steam games in the given library folders.
pub fn installed_apps(libraries: &[PathBuf]) -> Vec<SteamApp> {
    let mut apps = Vec::new();
    for lib in libraries {
        let steamapps = lib.join("steamapps");
        let Ok(entries) = std::fs::read_dir(&steamapps) else {
            continue;
        };
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().into_owned();
            if !(name.starts_with("appmanifest_") && name.ends_with(".acf")) {
                continue;
            }
            let Ok(text) = std::fs::read_to_string(entry.path()) else {
                continue;
            };
            let vdf = parse_vdf(&text);
            let Some(state) = vdf.get("AppState") else {
                continue;
            };
            let (Some(app_id), Some(app_name), Some(dir)) = (
                state.str("appid"),
                state.str("name"),
                state.str("installdir"),
            ) else {
                continue;
            };
            let install_dir = steamapps.join("common").join(dir);
            if install_dir.is_dir() {
                apps.push(SteamApp {
                    app_id: app_id.into(),
                    name: app_name.into(),
                    install_dir,
                });
            }
        }
    }
    apps
}

/// Convenience: everything installed in the default Steam locations.
pub fn detect() -> Vec<SteamApp> {
    installed_apps(&library_folders(&steam_roots()))
}

fn normalize(s: &str) -> String {
    s.chars()
        .filter(|c| c.is_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect()
}

/// Find the Steam app for a catalog game by name (punctuation/case-insensitive).
pub fn find_app<'a>(apps: &'a [SteamApp], names: &[&str]) -> Option<&'a SteamApp> {
    let wanted: Vec<String> = names
        .iter()
        .map(|n| normalize(n))
        .filter(|n| !n.is_empty())
        .collect();
    apps.iter().find(|a| {
        let name = normalize(&a.name);
        wanted
            .iter()
            .any(|w| name == *w || name.starts_with(w.as_str()))
    })
}

/// `steam://rungameid/<id>` URL for launching through the Steam client.
pub fn launch_url(app_id: &str) -> String {
    format!("steam://rungameid/{app_id}")
}

/// Map of normalized Steam name → app (handy for UI listings).
pub fn by_name(apps: &[SteamApp]) -> HashMap<String, SteamApp> {
    apps.iter()
        .map(|a| (normalize(&a.name), a.clone()))
        .collect()
}

pub fn is_installed(path: &Path) -> bool {
    path.is_dir()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_library_folders_and_manifests() {
        let root = std::env::temp_dir().join(format!("gachahub-steam-{}", uuid::Uuid::new_v4()));
        let second = root.join("SecondLibrary");
        std::fs::create_dir_all(root.join("steamapps/common/GF2Exilium")).unwrap();
        std::fs::create_dir_all(second.join("steamapps/common/Wuthering Waves")).unwrap();
        std::fs::write(
            root.join("steamapps/libraryfolders.vdf"),
            format!(
                "\"libraryfolders\"\n{{\n\t\"0\"\n\t{{\n\t\t\"path\"\t\t\"{}\"\n\t\t\"apps\" {{ \"1\" \"2\" }}\n\t}}\n\t\"1\"\n\t{{\n\t\t\"path\"\t\t\"{}\"\n\t}}\n}}\n",
                root.display().to_string().replace('\\', "\\\\"),
                second.display().to_string().replace('\\', "\\\\")
            ),
        )
        .unwrap();
        std::fs::write(
            root.join("steamapps/appmanifest_111.acf"),
            "\"AppState\"\n{\n\t\"appid\"\t\t\"111\"\n\t\"name\"\t\t\"GIRLS' FRONTLINE 2: EXILIUM\"\n\t\"installdir\"\t\t\"GF2Exilium\"\n}\n",
        )
        .unwrap();
        std::fs::write(
            second.join("steamapps/appmanifest_222.acf"),
            "// comment\n\"AppState\" { \"appid\" \"222\" \"name\" \"Wuthering Waves\" \"installdir\" \"Wuthering Waves\" }",
        )
        .unwrap();
        // manifest whose folder is missing (uninstalled) is ignored
        std::fs::write(
            root.join("steamapps/appmanifest_333.acf"),
            "\"AppState\" { \"appid\" \"333\" \"name\" \"Gone\" \"installdir\" \"Gone\" }",
        )
        .unwrap();

        let libs = library_folders(&[root.clone()]);
        assert_eq!(libs.len(), 2);
        let apps = installed_apps(&libs);
        assert_eq!(apps.len(), 2);
        let gfl = find_app(&apps, &["Girls' Frontline 2: Exilium"]).unwrap();
        assert_eq!(gfl.app_id, "111");
        assert!(gfl.install_dir.ends_with("GF2Exilium"));
        assert_eq!(find_app(&apps, &["WUTHERING WAVES"]).unwrap().app_id, "222");
        assert!(find_app(&apps, &["Blue Archive"]).is_none());
        assert_eq!(launch_url("111"), "steam://rungameid/111");
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn vdf_handles_escapes_and_nesting() {
        let v = parse_vdf(r#""a" { "b" "C:\\Games\\X" "c" { "d" "e" } }"#);
        assert_eq!(v.get("a").unwrap().str("b"), Some("C:\\Games\\X"));
        assert_eq!(v.get("a").unwrap().get("c").unwrap().str("d"), Some("e"));
    }
}
