use std::fs;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::config::CACHE_DIR;
use crate::model::Diagnostic;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedDiagnostics {
    pub diagnostics: Vec<Diagnostic>,
}

pub fn cache_key(
    root: &Path,
    paths: &[PathBuf],
    rule_fingerprint: &str,
    config_fingerprint: &str,
) -> Result<String> {
    let mut hasher = Sha256::new();
    hasher.update(rule_fingerprint.as_bytes());
    hasher.update(config_fingerprint.as_bytes());
    for path in paths {
        hasher.update(path.to_string_lossy().as_bytes());
        let full = root.join(path);
        if full.exists() {
            hasher.update(
                fs::read(&full).with_context(|| format!("failed to read {}", full.display()))?,
            );
        }
    }
    Ok(format!("{:x}", hasher.finalize()))
}

pub fn load(root: &Path, key: &str) -> Result<Option<Vec<Diagnostic>>> {
    let path = cache_path(root, key);
    if !path.exists() {
        return Ok(None);
    }
    let content = fs::read_to_string(&path)
        .with_context(|| format!("failed to read cache {}", path.display()))?;
    let cached: CachedDiagnostics = serde_json::from_str(&content)
        .with_context(|| format!("failed to parse cache {}", path.display()))?;
    Ok(Some(cached.diagnostics))
}

pub fn store(root: &Path, key: &str, diagnostics: Vec<Diagnostic>) -> Result<()> {
    let path = cache_path(root, key);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create cache dir {}", parent.display()))?;
    }
    let content = serde_json::to_string(&CachedDiagnostics { diagnostics })
        .context("failed to serialize diagnostics cache")?;
    fs::write(&path, content)
        .with_context(|| format!("failed to write cache {}", path.display()))?;
    Ok(())
}

pub fn clear(root: &Path) -> Result<()> {
    let path = root.join(CACHE_DIR);
    if path.exists() {
        fs::remove_dir_all(&path).with_context(|| format!("failed to clear {}", path.display()))?;
    }
    fs::create_dir_all(&path).with_context(|| format!("failed to create {}", path.display()))?;
    Ok(())
}

pub fn fingerprint<T: Hash>(value: &T) -> String {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    value.hash(&mut hasher);
    hasher.finish().to_string()
}

fn cache_path(root: &Path, key: &str) -> PathBuf {
    root.join(CACHE_DIR).join(format!("{key}.json"))
}
