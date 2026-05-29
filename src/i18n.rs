use std::path::PathBuf;
use std::sync::OnceLock;

use anyhow::Result;
use gettextrs::{TextDomain, gettext};

static INIT: OnceLock<()> = OnceLock::new();

pub fn init() -> Result<()> {
    if INIT.get().is_some() {
        return Ok(());
    }

    let locale_root = locale_root();
    let mut domain = TextDomain::new("taskforce")
        .skip_system_data_paths()
        .push(locale_root);

    if let Some(locale) = preferred_locale() {
        domain = domain.locale(&locale);
    }

    domain.init()?;

    let _ = INIT.set(());
    Ok(())
}

pub fn tr(message: &str) -> String {
    gettext(message)
}

fn locale_root() -> PathBuf {
    if let Ok(path) = std::env::var("TASKFORCE_LOCALE_ROOT") {
        return PathBuf::from(path);
    }

    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn preferred_locale() -> Option<String> {
    for key in ["TASKFORCE_LOCALE", "LC_MESSAGES", "LANG"] {
        if let Ok(value) = std::env::var(key)
            && is_meaningful_locale(&value)
        {
            return Some(value);
        }
    }

    None
}

fn is_meaningful_locale(value: &str) -> bool {
    let normalized = value.trim();
    !normalized.is_empty() && normalized != "C" && normalized != "C.UTF-8" && normalized != "POSIX"
}
