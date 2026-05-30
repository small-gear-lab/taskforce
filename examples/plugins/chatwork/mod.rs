mod import;
mod intake;

use std::path::PathBuf;

use crate::plugin::PluginManifest;
use anyhow::{Context, Result};

pub use import::import_chatwork_url;

pub fn manifest() -> Result<PluginManifest> {
    let path = manifest_path();
    let content = std::fs::read_to_string(&path)
        .with_context(|| format!("failed to read plugin manifest at {}", path.display()))?;
    toml::from_str(&content)
        .with_context(|| format!("failed to parse plugin manifest at {}", path.display()))
}

fn manifest_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("examples")
        .join("plugins")
        .join("chatwork")
        .join("manifest.toml")
}
