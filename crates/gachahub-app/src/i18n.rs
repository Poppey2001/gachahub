//! Minimal two-language support (Deutsch / English).
//!
//! Every UI string is written inline as `tr("Deutsch", "English")`, which keeps
//! translations next to where they are used and makes missing ones impossible.

use gachahub_core::settings::Language;
use std::sync::atomic::{AtomicU8, Ordering};

static LANG: AtomicU8 = AtomicU8::new(0);

pub fn set_language(lang: Language) {
    LANG.store(matches!(lang, Language::En) as u8, Ordering::Relaxed);
}

pub fn tr(de: &'static str, en: &'static str) -> &'static str {
    if LANG.load(Ordering::Relaxed) == 1 {
        en
    } else {
        de
    }
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
