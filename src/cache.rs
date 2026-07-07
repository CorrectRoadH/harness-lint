use std::collections::BTreeMap;
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::config::CACHE_DIR;
use crate::model::Diagnostic;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedFileDiagnostics {
    pub diagnostics: Vec<Diagnostic>,
}

pub fn file_hash(root: &Path, path: &Path) -> Result<String> {
    let full = root.join(path);
    let bytes = fs::read(&full).with_context(|| format!("failed to read {}", full.display()))?;
    Ok(format!("{:x}", Sha256::digest(bytes)))
}

pub fn file_cache_key(
    path: &Path,
    file_hash: &str,
    rule_fingerprint: &str,
    config_fingerprint: &str,
) -> String {
    let mut hasher = Sha256::new();
    hasher.update(b"file-v1");
    hasher.update(rule_fingerprint.as_bytes());
    hasher.update(config_fingerprint.as_bytes());
    hasher.update(path.to_string_lossy().as_bytes());
    hasher.update(file_hash.as_bytes());
    format!("{:x}", hasher.finalize())
}

pub fn load_file(root: &Path, key: &str) -> Result<Option<Vec<Diagnostic>>> {
    let path = file_cache_path(root, key);
    if !path.exists() {
        return Ok(None);
    }
    let content = fs::read_to_string(&path)
        .with_context(|| format!("failed to read cache {}", path.display()))?;
    // A corrupt or truncated entry is a cache miss, not a fatal error; drop it
    // so the next store rewrites it.
    match serde_json::from_str::<CachedFileDiagnostics>(&content) {
        Ok(cached) => Ok(Some(cached.diagnostics)),
        Err(_) => {
            let _ = fs::remove_file(&path);
            Ok(None)
        }
    }
}

pub fn store_file(root: &Path, key: &str, diagnostics: Vec<Diagnostic>) -> Result<()> {
    let path = file_cache_path(root, key);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create cache dir {}", parent.display()))?;
    }
    let content = serde_json::to_string(&CachedFileDiagnostics { diagnostics })
        .context("failed to serialize file diagnostics cache")?;
    let temp_path = path.with_extension(format!("tmp.{}", std::process::id()));
    fs::write(&temp_path, content)
        .with_context(|| format!("failed to write cache {}", temp_path.display()))?;
    if let Err(error) = fs::rename(&temp_path, &path) {
        let _ = fs::remove_file(&temp_path);
        return Err(error).with_context(|| format!("failed to write cache {}", path.display()));
    }
    Ok(())
}

pub fn group_by_path(
    root: &Path,
    diagnostics: Vec<Diagnostic>,
) -> BTreeMap<PathBuf, Vec<Diagnostic>> {
    let mut by_path: BTreeMap<PathBuf, Vec<Diagnostic>> = BTreeMap::new();
    for mut diagnostic in diagnostics {
        diagnostic.path = normalize_path(root, &diagnostic.path);
        by_path
            .entry(diagnostic.path.clone())
            .or_default()
            .push(diagnostic);
    }
    by_path
}

pub fn normalize_path(root: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.strip_prefix(root).unwrap_or(path).to_path_buf()
    } else {
        path.strip_prefix("./").unwrap_or(path).to_path_buf()
    }
}

pub fn clear(root: &Path) -> Result<()> {
    let path = root.join(CACHE_DIR);
    if path.exists() {
        fs::remove_dir_all(&path).with_context(|| format!("failed to clear {}", path.display()))?;
    }
    fs::create_dir_all(&path).with_context(|| format!("failed to create {}", path.display()))?;
    Ok(())
}

/// Entries older than this are pruned after a check run that stored new
/// results. Every file edit creates a new key, so without garbage collection
/// the directory grows without bound.
const FILE_CACHE_MAX_AGE: std::time::Duration = std::time::Duration::from_secs(30 * 24 * 60 * 60);

/// Best-effort garbage collection of stale file-cache entries.
pub fn prune_stale_file_entries(root: &Path) {
    let dir = root.join(CACHE_DIR).join("files");
    let Ok(entries) = fs::read_dir(&dir) else {
        return;
    };
    let now = std::time::SystemTime::now();
    for entry in entries.flatten() {
        let stale = entry
            .metadata()
            .and_then(|metadata| metadata.modified())
            .ok()
            .and_then(|modified| now.duration_since(modified).ok())
            .is_some_and(|age| age > FILE_CACHE_MAX_AGE);
        if stale {
            let _ = fs::remove_file(entry.path());
        }
    }
}

pub fn fingerprint<T: Hash>(value: &T) -> String {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    value.hash(&mut hasher);
    hasher.finish().to_string()
}

fn file_cache_path(root: &Path, key: &str) -> PathBuf {
    root.join(CACHE_DIR)
        .join("files")
        .join(format!("{key}.json"))
}
