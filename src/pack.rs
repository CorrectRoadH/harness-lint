use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result, anyhow, bail};
use serde::Deserialize;

use crate::config::PACKS_DIR;
use crate::model::{LockEntry, PackSourceKind, PackSpec, ResolvedPack, RulePack};
use crate::rule::{discover_rules, parse_rule_file};

pub const PACK_MANIFEST: &str = "harness-pack.toml";

#[derive(Debug, Deserialize)]
struct PackManifest {
    pack: PackSection,
    #[serde(default)]
    compat: CompatSection,
    #[serde(default)]
    rules: toml::Table,
}

#[derive(Debug, Deserialize)]
struct PackSection {
    id: String,
    name: String,
    version: String,
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    license: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
#[allow(dead_code)]
struct CompatSection {
    #[serde(default)]
    harness: Option<String>,
    #[serde(default)]
    grit: Option<String>,
    #[serde(default)]
    languages: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct ManifestRule {
    path: PathBuf,
}

pub fn parse_pack_spec(id: &str, spec: &str) -> PackSpec {
    let (source, rest) = if let Some(rest) = spec.strip_prefix("github:") {
        (PackSourceKind::Git, rest)
    } else if let Some(rest) = spec.strip_prefix("git:") {
        (PackSourceKind::Git, rest)
    } else if let Some(rest) = spec.strip_prefix("local:") {
        (PackSourceKind::Local, rest)
    } else if let Some(rest) = spec.strip_prefix("npm:") {
        (PackSourceKind::Npm, rest)
    } else if let Some(rest) = spec.strip_prefix("cargo:") {
        (PackSourceKind::Cargo, rest)
    } else if let Some(rest) = spec.strip_prefix("pip:") {
        (PackSourceKind::Pip, rest)
    } else if spec.starts_with("http://") || spec.starts_with("https://") {
        (PackSourceKind::Url, spec)
    } else {
        (PackSourceKind::Local, spec)
    };

    let (rest, fragment) = split_fragment(rest);
    let (mut spec, version_req) = split_version(rest);
    if let Some(fragment) = fragment {
        spec = format!("{spec}#{fragment}");
    }
    PackSpec {
        id: id.to_string(),
        source,
        spec,
        version_req,
    }
}

fn split_fragment(value: &str) -> (&str, Option<&str>) {
    if let Some((left, right)) = value.split_once('#') {
        if !right.is_empty() {
            return (left, Some(right));
        }
    }
    (value, None)
}

fn split_version(value: &str) -> (String, Option<String>) {
    if let Some((left, right)) = value.rsplit_once('@') {
        if !right.is_empty() && !left.is_empty() {
            return (left.to_string(), Some(right.to_string()));
        }
    }
    (value.to_string(), None)
}

pub fn resolve_local_pack(root: &Path, spec: PackSpec) -> Result<ResolvedPack> {
    let (pack_path, fragment) = split_spec_path(&spec.spec);
    let local_path = if Path::new(pack_path).is_absolute() {
        PathBuf::from(pack_path)
    } else {
        root.join(pack_path)
    };
    if !local_path.exists() {
        bail!("local pack path does not exist: {}", local_path.display());
    }
    let local_path = resolve_pack_root(&local_path, &spec.id, fragment)?;
    Ok(ResolvedPack {
        spec,
        local_path,
        version: None,
        checksum: None,
    })
}

pub fn install_git_pack(root: &Path, spec: PackSpec) -> Result<ResolvedPack> {
    let target = root.join(PACKS_DIR).join(&spec.id);
    let temp_target = root.join(PACKS_DIR).join(format!("{}.tmp", spec.id));
    if target.exists() {
        fs::remove_dir_all(&target)
            .with_context(|| format!("failed to clear {}", target.display()))?;
    }
    if temp_target.exists() {
        fs::remove_dir_all(&temp_target)
            .with_context(|| format!("failed to clear {}", temp_target.display()))?;
    }
    fs::create_dir_all(target.parent().expect("pack target has parent"))
        .with_context(|| format!("failed to create {}", root.join(PACKS_DIR).display()))?;

    let (git_spec, fragment) = split_spec_path(&spec.spec);
    let url = git_url(git_spec);
    let mut command = Command::new("git");
    command.arg("clone").arg("--depth").arg("1");
    if let Some(version) = &spec.version_req {
        command.arg("--branch").arg(version);
    }
    command.arg(&url).arg(&temp_target);
    let output = command
        .output()
        .with_context(|| format!("failed to clone {url}"))?;
    if !output.status.success() {
        let _ = fs::remove_dir_all(&temp_target);
        bail!(
            "failed to clone {url}: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    fs::rename(&temp_target, &target).with_context(|| {
        format!(
            "failed to move installed pack from {} to {}",
            temp_target.display(),
            target.display()
        )
    })?;

    let local_path = resolve_pack_root(&target, &spec.id, fragment)?;
    let commit = git_commit(&target).ok();
    Ok(ResolvedPack {
        spec,
        local_path,
        version: commit,
        checksum: None,
    })
}

pub fn update_git_pack(root: &Path, lock: &LockEntry) -> Result<ResolvedPack> {
    let target = if lock.local_path.is_absolute() {
        lock.local_path.clone()
    } else {
        root.join(&lock.local_path)
    };
    if !target.exists() {
        let spec = PackSpec {
            id: lock.id.clone(),
            source: lock.source.clone(),
            spec: lock.spec.clone(),
            version_req: lock.version.clone(),
        };
        return install_git_pack(root, spec);
    }
    let output = Command::new("git")
        .current_dir(&target)
        .args(["pull", "--ff-only"])
        .output()
        .with_context(|| format!("failed to update {}", target.display()))?;
    if !output.status.success() {
        bail!(
            "failed to update {}: {}",
            target.display(),
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    let commit = git_commit(&target).ok();
    Ok(ResolvedPack {
        spec: PackSpec {
            id: lock.id.clone(),
            source: lock.source.clone(),
            spec: lock.spec.clone(),
            version_req: lock.version.clone(),
        },
        local_path: resolve_pack_root(&target, &lock.id, split_spec_path(&lock.spec).1)?,
        version: commit,
        checksum: None,
    })
}

pub fn lock_entry(resolved: &ResolvedPack, root: &Path) -> LockEntry {
    let local_path = resolved
        .local_path
        .strip_prefix(root)
        .unwrap_or(&resolved.local_path)
        .to_path_buf();
    LockEntry {
        id: resolved.spec.id.clone(),
        source: resolved.spec.source.clone(),
        spec: resolved.spec.spec.clone(),
        version: resolved
            .version
            .clone()
            .or(resolved.spec.version_req.clone()),
        checksum: resolved.checksum.clone(),
        local_path,
    }
}

pub fn load_rule_pack(resolved: &ResolvedPack) -> Result<RulePack> {
    let manifest_path = resolved.local_path.join(PACK_MANIFEST);
    let content = fs::read_to_string(&manifest_path)
        .with_context(|| format!("failed to read {}", manifest_path.display()))?;
    let manifest: PackManifest = toml::from_str(&content)
        .with_context(|| format!("failed to parse {}", manifest_path.display()))?;

    let _compat = manifest.compat;
    let _metadata = (manifest.pack.description, manifest.pack.license);

    let mut rules = Vec::new();
    if manifest.rules.is_empty() {
        rules = discover_rules(&resolved.local_path.join("rules"), Some(&manifest.pack.id))?;
    } else {
        for (rule_id, value) in manifest.rules {
            let rule: ManifestRule = value
                .try_into()
                .map_err(|error| anyhow!("invalid rule entry {rule_id}: {error}"))?;
            let path = resolved.local_path.join(rule.path);
            rules.push(parse_rule_file(&path, Some(&manifest.pack.id))?);
        }
    }

    ensure_unique_rule_ids(&rules)?;

    Ok(RulePack {
        id: manifest.pack.id,
        name: manifest.pack.name,
        version: manifest.pack.version,
        rules,
    })
}

fn ensure_unique_rule_ids(rules: &[crate::model::RuleDefinition]) -> Result<()> {
    let mut seen = BTreeSet::new();
    for rule in rules {
        if !seen.insert(rule.id.clone()) {
            bail!("duplicate rule id in pack: {}", rule.id);
        }
    }
    Ok(())
}

pub fn load_local_rules_pack(root: &Path, dirs: &[PathBuf]) -> Result<RulePack> {
    let mut rules = Vec::new();
    for dir in dirs {
        let path = if dir.is_absolute() {
            dir.clone()
        } else {
            root.join(dir)
        };
        rules.extend(discover_rules(&path, None)?);
    }
    ensure_unique_rule_ids(&rules)?;
    Ok(RulePack {
        id: "local".to_string(),
        name: "Local Rules".to_string(),
        version: "0.0.0".to_string(),
        rules,
    })
}

fn git_url(spec: &str) -> String {
    if spec.starts_with("http://") || spec.starts_with("https://") || spec.starts_with("git@") {
        spec.to_string()
    } else {
        format!("https://github.com/{spec}.git")
    }
}

fn split_spec_path(spec: &str) -> (&str, Option<&str>) {
    if let Some((left, right)) = spec.split_once('#') {
        (left, Some(right))
    } else {
        (spec, None)
    }
}

fn resolve_pack_root(path: &Path, id: &str, fragment: Option<&str>) -> Result<PathBuf> {
    if let Some(fragment) = fragment {
        let candidate = path.join(fragment);
        if candidate.join(PACK_MANIFEST).exists() {
            return Ok(candidate);
        }
        bail!("pack `{id}` subdirectory `{fragment}` does not contain {PACK_MANIFEST}");
    }
    if path.join(PACK_MANIFEST).exists() {
        return Ok(path.to_path_buf());
    }
    let candidate = path.join("packs").join(id);
    if candidate.join(PACK_MANIFEST).exists() {
        return Ok(candidate);
    }
    bail!(
        "pack `{id}` does not contain {PACK_MANIFEST} at {} or packs/{id}",
        path.display()
    )
}

fn git_commit(path: &Path) -> Result<String> {
    let output = Command::new("git")
        .current_dir(path)
        .args(["rev-parse", "HEAD"])
        .output()
        .with_context(|| format!("failed to read commit for {}", path.display()))?;
    if !output.status.success() {
        bail!("failed to read commit for {}", path.display());
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_github_pack_spec_with_version() {
        let spec = parse_pack_spec("python", "github:harness-lint/rules-python@1.2.0");
        assert_eq!(spec.source, PackSourceKind::Git);
        assert_eq!(spec.spec, "harness-lint/rules-python");
        assert_eq!(spec.version_req.as_deref(), Some("1.2.0"));
    }

    #[test]
    fn parses_github_pack_spec_with_subdirectory() {
        let spec = parse_pack_spec(
            "python",
            "github:CorrectRoadH/harness-lint@main#packs/python",
        );
        assert_eq!(spec.source, PackSourceKind::Git);
        assert_eq!(spec.spec, "CorrectRoadH/harness-lint#packs/python");
        assert_eq!(spec.version_req.as_deref(), Some("main"));
    }
}
