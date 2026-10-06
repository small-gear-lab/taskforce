// Copyright (c) 2026- Masaki Ishii
// Copyright (c) 2026- Small Gear Lab
// SPDX-License-Identifier: MIT OR Apache-2.0

use std::fs::File;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use anyhow::Result;
use gettext::Catalog;

static CATALOG: OnceLock<Option<Catalog>> = OnceLock::new();

pub fn init() -> Result<()> {
    if CATALOG.get().is_some() {
        return Ok(());
    }

    let catalog = load_catalog()?;
    let _ = CATALOG.set(catalog);
    Ok(())
}

pub fn tr(message: &str) -> String {
    CATALOG
        .get()
        .and_then(|catalog| catalog.as_ref())
        .map(|catalog| catalog.gettext(message).to_string())
        .unwrap_or_else(|| message.to_string())
}

fn load_catalog() -> Result<Option<Catalog>> {
    match find_catalog_path(&locale_roots(), &preferred_locale_candidates()) {
        Some(path) => Ok(Some(parse_catalog(&path)?)),
        None => Ok(None),
    }
}

/// First existing `<root>/locale/<locale>/LC_MESSAGES/taskforce.mo`. Roots take
/// priority over locale candidates, so an earlier root wins over a later one
/// even when the later one has a more specific locale.
fn find_catalog_path(roots: &[PathBuf], candidates: &[String]) -> Option<PathBuf> {
    roots
        .iter()
        .flat_map(|root| {
            candidates.iter().map(move |locale| {
                root.join("locale")
                    .join(locale)
                    .join("LC_MESSAGES")
                    .join("taskforce.mo")
            })
        })
        .find(|path| path.is_file())
}

fn parse_catalog(path: &Path) -> Result<Catalog> {
    let file = File::open(path)?;
    Ok(Catalog::parse(file)?)
}

/// Directories searched for `locale/`, highest priority first: an explicit
/// `TASKFORCE_LOCALE_ROOT`, the config directory, then the source tree this
/// binary was built from (kept for backward compatibility).
fn locale_roots() -> Vec<PathBuf> {
    let mut roots = Vec::new();
    roots.extend(std::env::var_os("TASKFORCE_LOCALE_ROOT").map(PathBuf::from));
    roots.extend(crate::config::config_dir());
    roots.push(PathBuf::from(env!("CARGO_MANIFEST_DIR")));
    roots
}

pub(crate) fn preferred_locale_candidates() -> Vec<String> {
    let mut candidates = Vec::new();

    for key in ["TASKFORCE_LOCALE", "LC_MESSAGES", "LANG"] {
        if let Ok(value) = std::env::var(key) {
            push_locale_candidates(&mut candidates, &value);
        }
    }

    candidates
}

fn push_locale_candidates(candidates: &mut Vec<String>, value: &str) {
    let normalized = value.trim();

    if !is_meaningful_locale(normalized) {
        return;
    }

    let without_encoding = normalized
        .split_once('.')
        .map(|(locale, _)| locale)
        .unwrap_or(normalized);
    let without_modifier = without_encoding
        .split_once('@')
        .map(|(locale, _)| locale)
        .unwrap_or(without_encoding);

    for candidate in [
        normalized,
        without_encoding,
        without_modifier,
        without_modifier
            .split_once('_')
            .map(|(language, _)| language)
            .unwrap_or(without_modifier),
    ] {
        if is_meaningful_locale(candidate)
            && !candidates.iter().any(|existing| existing == candidate)
        {
            candidates.push(candidate.to_string());
        }
    }
}

fn is_meaningful_locale(value: &str) -> bool {
    !value.is_empty() && value != "C" && value != "C.UTF-8" && value != "POSIX"
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::{find_catalog_path, parse_catalog, push_locale_candidates};

    fn unique_temp_dir(prefix: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        std::env::temp_dir().join(format!("{prefix}-{nanos}"))
    }

    fn touch_catalog(root: &Path, locale: &str) -> PathBuf {
        let dir = root.join("locale").join(locale).join("LC_MESSAGES");
        fs::create_dir_all(&dir).expect("create catalog dir");
        let path = dir.join("taskforce.mo");
        fs::write(&path, b"").expect("write catalog");
        path
    }

    #[test]
    fn builds_locale_fallback_candidates() {
        let mut candidates = Vec::new();
        push_locale_candidates(&mut candidates, "ja_JP.UTF-8");
        assert_eq!(candidates, vec!["ja_JP.UTF-8", "ja_JP", "ja"]);
    }

    #[test]
    fn ignores_non_meaningful_locales() {
        let mut candidates = Vec::new();
        push_locale_candidates(&mut candidates, "C.UTF-8");
        assert!(candidates.is_empty());
    }

    #[test]
    fn locale_candidates_do_not_duplicate_entries() {
        let mut candidates = Vec::new();
        push_locale_candidates(&mut candidates, "ja_JP.UTF-8");
        push_locale_candidates(&mut candidates, "ja_JP");
        assert_eq!(candidates, vec!["ja_JP.UTF-8", "ja_JP", "ja"]);
    }

    #[test]
    fn earlier_roots_win_over_more_specific_locales() {
        let first = unique_temp_dir("taskforce-i18n-first");
        let second = unique_temp_dir("taskforce-i18n-second");
        let first_ja = touch_catalog(&first, "ja");
        touch_catalog(&second, "ja_JP");
        let candidates = vec!["ja_JP".to_string(), "ja".to_string()];

        let found = find_catalog_path(&[first.clone(), second.clone()], &candidates);

        assert_eq!(found, Some(first_ja));
        fs::remove_dir_all(first).expect("cleanup");
        fs::remove_dir_all(second).expect("cleanup");
    }

    #[test]
    fn falls_through_to_later_roots_when_earlier_ones_lack_the_locale() {
        let empty = unique_temp_dir("taskforce-i18n-empty");
        let populated = unique_temp_dir("taskforce-i18n-populated");
        let expected = touch_catalog(&populated, "ja");
        let candidates = vec!["ja_JP".to_string(), "ja".to_string()];

        let found = find_catalog_path(&[empty.clone(), populated.clone()], &candidates);

        assert_eq!(found, Some(expected));
        fs::remove_dir_all(populated).expect("cleanup");
    }

    #[test]
    fn returns_none_without_a_matching_catalog() {
        let empty = unique_temp_dir("taskforce-i18n-none");
        assert_eq!(find_catalog_path(&[empty], &["ja".to_string()]), None);
    }

    #[test]
    fn bundled_japanese_catalog_translates_open_tasks() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("locale")
            .join("ja")
            .join("LC_MESSAGES")
            .join("taskforce.mo");
        let catalog = parse_catalog(&path).expect("bundled catalog");
        assert_eq!(catalog.gettext("Open Tasks"), "オープンタスク");
    }
}
