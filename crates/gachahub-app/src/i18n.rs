//! UI languages: Deutsch, English, Español, Français.
//!
//! Every UI string is written inline as `tr("Deutsch", "English")`, which keeps
//! the two source languages next to where they are used. Spanish and French
//! are looked up by the English text in `assets/i18n/translations.json`
//! (falling back to English); a test makes sure every string is translated.

use gachahub_core::settings::Language;
use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicU8, Ordering},
        OnceLock,
    },
};

static LANG: AtomicU8 = AtomicU8::new(0);

const TRANSLATIONS: &str = include_str!("../assets/i18n/translations.json");

/// English → (Spanish, French).
fn table() -> &'static HashMap<String, (String, String)> {
    static TABLE: OnceLock<HashMap<String, (String, String)>> = OnceLock::new();
    TABLE.get_or_init(|| {
        let raw: HashMap<String, HashMap<String, String>> =
            serde_json::from_str(TRANSLATIONS).unwrap_or_default();
        raw.into_iter()
            .map(|(en, mut t)| {
                let es = t.remove("es").unwrap_or_default();
                let fr = t.remove("fr").unwrap_or_default();
                (en, (es, fr))
            })
            .collect()
    })
}

pub fn set_language(lang: Language) {
    let v = match lang {
        Language::De => 0,
        Language::En => 1,
        Language::Es => 2,
        Language::Fr => 3,
    };
    LANG.store(v, Ordering::Relaxed);
}

/// Pick the text for the current language from a German/English pair.
pub fn text<'a>(de: &'a str, en: &'a str) -> &'a str {
    let lang = LANG.load(Ordering::Relaxed);
    match lang {
        0 => de,
        1 => en,
        _ => match table().get(en) {
            Some((es, fr)) => {
                let t = if lang == 2 { es } else { fr };
                if t.is_empty() {
                    en
                } else {
                    t.as_str()
                }
            }
            None => en,
        },
    }
}

/// Translate a list of notes given in German and English.
pub fn texts<'a>(de: &'a [String], en: &'a [String]) -> Vec<&'a str> {
    if is_german() || en.is_empty() {
        return de.iter().map(String::as_str).collect();
    }
    en.iter()
        .enumerate()
        .map(|(i, e)| text(de.get(i).map(String::as_str).unwrap_or(e), e))
        .collect()
}

pub fn tr(de: &'static str, en: &'static str) -> &'static str {
    text(de, en)
}

pub fn is_german() -> bool {
    LANG.load(Ordering::Relaxed) == 0
}

/// Translated name of a runtime kind.
pub fn runtime_label(kind: gachahub_core::settings::RuntimeKind) -> &'static str {
    use gachahub_core::settings::RuntimeKind::*;
    match kind {
        Auto => tr("Automatisch", "Automatic"),
        Native => tr("Nativ", "Native"),
        other => other.label(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every `tr("…", "…")` in the app and every game/compatibility text has
    /// a Spanish and French translation.
    #[test]
    fn all_strings_are_translated() {
        let table = table();
        assert!(table.len() > 300, "translation table failed to load");
        let mut missing = Vec::new();
        let mut check = |en: String| match table.get(&en) {
            Some((es, fr)) if !es.is_empty() && !fr.is_empty() => {}
            _ => missing.push(en),
        };
        // UI strings: second argument of tr(...)
        let src = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
        let mut files = vec![src.clone()];
        while let Some(p) = files.pop() {
            if p.is_dir() {
                files.extend(std::fs::read_dir(&p).unwrap().flatten().map(|e| e.path()));
                continue;
            }
            if p.extension().is_none_or(|e| e != "rs") || p.ends_with("i18n.rs") {
                continue;
            }
            for en in tr_english_args(&std::fs::read_to_string(&p).unwrap()) {
                check(en);
            }
        }
        // game texts
        let games: serde_json::Value = serde_json::from_str(include_str!(
            "../../gachahub-core/game-manifests/games.json"
        ))
        .unwrap();
        for g in games.as_array().unwrap() {
            for k in ["subtitle", "notes"] {
                if let Some(s) = g[k].as_str() {
                    check(s.to_string());
                }
            }
            for l in g["officialLaunchers"].as_array().into_iter().flatten() {
                for k in ["label", "region"] {
                    if let Some(s) = l[k].as_str() {
                        check(s.to_string());
                    }
                }
            }
        }
        let compat: serde_json::Value = serde_json::from_str(include_str!(
            "../../gachahub-core/game-manifests/compatibility.json"
        ))
        .unwrap();
        for c in compat.as_array().unwrap() {
            if let Some(s) = c["linux"]["labelEn"].as_str() {
                check(s.to_string());
            }
            for n in c["linux"]["notesEn"].as_array().into_iter().flatten() {
                check(n.as_str().unwrap().to_string());
            }
        }
        assert!(
            missing.is_empty(),
            "missing es/fr translations: {missing:#?}"
        );
    }

    /// English arguments of `tr("de", "en")` calls (string literals with escapes).
    fn tr_english_args(src: &str) -> Vec<String> {
        let mut out = Vec::new();
        let mut rest = src;
        while let Some(pos) = rest.find("tr(") {
            let before = rest[..pos].chars().last();
            rest = &rest[pos + 3..];
            if before.is_some_and(|c| c.is_alphanumeric() || c == '_') {
                continue;
            }
            let mut lits = Vec::new();
            let mut s = rest.trim_start();
            for _ in 0..2 {
                let Some(body) = s.strip_prefix('"') else {
                    break;
                };
                let mut lit = String::new();
                let mut chars = body.char_indices();
                let mut end = None;
                while let Some((i, c)) = chars.next() {
                    match c {
                        '\\' => {
                            if let Some((_, n)) = chars.next() {
                                lit.push(match n {
                                    'n' => '\n',
                                    't' => '\t',
                                    other => other,
                                });
                            }
                        }
                        '"' => {
                            end = Some(i);
                            break;
                        }
                        c => lit.push(c),
                    }
                }
                let Some(end) = end else { break };
                lits.push(lit);
                s = body[end + 1..].trim_start();
                s = s.strip_prefix(',').unwrap_or(s).trim_start();
            }
            if lits.len() == 2 {
                out.push(lits.pop().unwrap());
            }
        }
        out
    }

    #[test]
    fn falls_back_and_switches() {
        set_language(Language::Fr);
        assert_eq!(tr("Spielen", "Play"), "Jouer");
        assert_eq!(text("x", "not in table"), "not in table");
        set_language(Language::Es);
        assert_eq!(tr("Spielen", "Play"), "Jugar");
        set_language(Language::De);
        assert_eq!(tr("Spielen", "Play"), "Spielen");
    }
}
