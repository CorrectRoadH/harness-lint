use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result, anyhow, bail};
use serde::Deserialize;
use sha2::{Digest, Sha256};
use walkdir::WalkDir;

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
    let spec = spec.trim();
    if let Some(rest) = spec.strip_prefix("local:") {
        return parse_source_parts(id, PackSourceKind::Local, rest);
    }
    if let Some(rest) = spec.strip_prefix("github:") {
        return parse_git_source(id, rest);
    }
    if let Some(rest) = spec.strip_prefix("git:") {
        return parse_source_parts(id, PackSourceKind::Git, rest);
    }
    if let Some(rest) = spec.strip_prefix("cargo:") {
        return parse_source_parts(id, PackSourceKind::Cargo, rest);
    }
    if let Some(rest) = spec.strip_prefix("pip:") {
        return parse_source_parts(id, PackSourceKind::Pip, rest);
    }
    if let Some(parsed) = parse_github_url(id, spec) {
        return parsed;
    }
    if looks_like_github_shorthand(spec) {
        return parse_git_source(id, spec);
    }
    if spec.starts_with("http://") || spec.starts_with("https://") {
        return parse_source_parts(id, PackSourceKind::Url, spec);
    }

    parse_source_parts(id, PackSourceKind::Local, spec)
}

fn parse_git_source(id: &str, spec: &str) -> PackSpec {
    if let Some(parsed) = parse_github_shorthand(id, spec) {
        return parsed;
    }
    parse_source_parts(id, PackSourceKind::Git, spec)
}

fn parse_source_parts(id: &str, source: PackSourceKind, value: &str) -> PackSpec {
    let (rest, fragment) = split_fragment(value);
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

fn parse_github_url(id: &str, value: &str) -> Option<PackSpec> {
    let marker = "github.com/";
    let path = value.split_once(marker)?.1;
    let path = path.split('?').next().unwrap_or(path);
    let path = path.trim_end_matches('/');
    let segments: Vec<&str> = path.split('/').filter(|part| !part.is_empty()).collect();
    if segments.len() < 2 {
        return None;
    }

    let owner = segments[0];
    let repo = segments[1].trim_end_matches(".git");
    if owner.is_empty() || repo.is_empty() {
        return None;
    }

    if segments.get(2) == Some(&"tree") && segments.len() >= 4 {
        let ref_name = segments[3].to_string();
        let fragment = if segments.len() > 4 {
            Some(segments[4..].join("/"))
        } else {
            None
        };
        let spec = with_fragment(&format!("{owner}/{repo}"), fragment.as_deref());
        return Some(PackSpec {
            id: id.to_string(),
            source: PackSourceKind::Git,
            spec,
            version_req: Some(ref_name),
        });
    }

    Some(PackSpec {
        id: id.to_string(),
        source: PackSourceKind::Git,
        spec: format!("{owner}/{repo}"),
        version_req: None,
    })
}

fn parse_github_shorthand(id: &str, value: &str) -> Option<PackSpec> {
    if !looks_like_github_shorthand(value) {
        return None;
    }
    let (rest, fragment) = split_fragment(value);
    let (rest, version_req) = split_version(rest);
    let mut parts = rest.splitn(3, '/');
    let owner = parts.next()?;
    let repo = parts.next()?;
    let path = parts.next();
    let fragment = fragment.or(path);
    Some(PackSpec {
        id: id.to_string(),
        source: PackSourceKind::Git,
        spec: with_fragment(&format!("{owner}/{repo}"), fragment),
        version_req,
    })
}

fn looks_like_github_shorthand(value: &str) -> bool {
    if value.starts_with('.') || value.starts_with('/') || value.contains(':') {
        return false;
    }
    let mut parts = value.split('/');
    let Some(owner) = parts.next() else {
        return false;
    };
    let Some(repo) = parts.next() else {
        return false;
    };
    !owner.is_empty() && !repo.is_empty()
}

fn with_fragment(spec: &str, fragment: Option<&str>) -> String {
    match fragment.filter(|fragment| !fragment.is_empty()) {
        Some(fragment) => format!("{spec}#{fragment}"),
        None => spec.to_string(),
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
    let pack_path = fragment.map(PathBuf::from);
    let checksum = compute_pack_content_hash(&local_path)?;
    Ok(ResolvedPack {
        spec,
        local_path,
        pack_path,
        version: None,
        checksum: Some(checksum),
    })
}

pub fn install_git_pack(root: &Path, spec: PackSpec) -> Result<ResolvedPack> {
    let target = root.join(PACKS_DIR).join(&spec.id);
    let temp_target = root.join(PACKS_DIR).join(format!("{}.tmp", spec.id));
    if temp_target.exists() {
        fs::remove_dir_all(&temp_target)
            .with_context(|| format!("failed to clear {}", temp_target.display()))?;
    }
    fs::create_dir_all(target.parent().expect("pack target has parent"))
        .with_context(|| format!("failed to create {}", root.join(PACKS_DIR).display()))?;

    clone_git_source(&spec, &temp_target)?;
    if target.exists() {
        fs::remove_dir_all(&target)
            .with_context(|| format!("failed to clear {}", target.display()))?;
    }
    fs::rename(&temp_target, &target).with_context(|| {
        format!(
            "failed to move installed pack from {} to {}",
            temp_target.display(),
            target.display()
        )
    })?;

    let (_, fragment) = split_spec_path(&spec.spec);
    let local_path = resolve_pack_root(&target, &spec.id, fragment)?;
    let commit = git_commit(&target).ok();
    let pack_path = pack_path_from_repo(&target, &local_path)?;
    let checksum = git_tree_hash(&target, pack_path.as_deref()).ok();
    Ok(ResolvedPack {
        spec,
        local_path,
        pack_path,
        version: commit,
        checksum,
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackRemoteStatus {
    pub id: String,
    pub installed_checksum: Option<String>,
    pub latest_checksum: Option<String>,
    pub latest_version: Option<String>,
    pub update_available: bool,
}

pub fn check_git_pack_update(
    root: &Path,
    spec: PackSpec,
    lock: Option<&LockEntry>,
) -> Result<PackRemoteStatus> {
    let temp_target = root.join(PACKS_DIR).join(format!("{}.check.tmp", spec.id));
    if temp_target.exists() {
        fs::remove_dir_all(&temp_target)
            .with_context(|| format!("failed to clear {}", temp_target.display()))?;
    }
    fs::create_dir_all(temp_target.parent().expect("pack target has parent"))
        .with_context(|| format!("failed to create {}", root.join(PACKS_DIR).display()))?;
    clone_git_source(&spec, &temp_target)?;
    let (_, fragment) = split_spec_path(&spec.spec);
    let local_path = resolve_pack_root(&temp_target, &spec.id, fragment)?;
    let pack_path = pack_path_from_repo(&temp_target, &local_path)?;
    let latest_checksum = git_tree_hash(&temp_target, pack_path.as_deref()).ok();
    let latest_version = git_commit(&temp_target).ok();
    let _ = fs::remove_dir_all(&temp_target);

    let installed_checksum = lock.and_then(|entry| entry.checksum.clone());
    let update_available = installed_checksum.is_some()
        && latest_checksum.is_some()
        && installed_checksum != latest_checksum;
    Ok(PackRemoteStatus {
        id: spec.id,
        installed_checksum,
        latest_checksum,
        latest_version,
        update_available,
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
        requested_ref: resolved.spec.version_req.clone(),
        version: resolved
            .version
            .clone()
            .or(resolved.spec.version_req.clone()),
        checksum: resolved.checksum.clone(),
        local_path,
        pack_path: resolved.pack_path.clone(),
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

fn clone_git_source(spec: &PackSpec, target: &Path) -> Result<()> {
    let (git_spec, _) = split_spec_path(&spec.spec);
    let url = git_url(git_spec);
    let mut command = Command::new("git");
    command
        .arg("-c")
        .arg("filter.lfs.required=false")
        .arg("-c")
        .arg("filter.lfs.smudge=")
        .arg("-c")
        .arg("filter.lfs.clean=")
        .arg("-c")
        .arg("filter.lfs.process=")
        .arg("clone")
        .arg("--depth")
        .arg("1");
    if let Some(version) = &spec.version_req {
        command.arg("--branch").arg(version);
    }
    command
        .env("GIT_TERMINAL_PROMPT", "0")
        .env("GIT_LFS_SKIP_SMUDGE", "1")
        .arg(&url)
        .arg(target);
    let output = command
        .output()
        .with_context(|| format!("failed to clone {url}"))?;
    if !output.status.success() {
        let _ = fs::remove_dir_all(target);
        bail!(
            "failed to clone {url}: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    Ok(())
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

fn pack_path_from_repo(repo_root: &Path, pack_root: &Path) -> Result<Option<PathBuf>> {
    let relative = pack_root.strip_prefix(repo_root).with_context(|| {
        format!(
            "failed to derive pack path for {} inside {}",
            pack_root.display(),
            repo_root.display()
        )
    })?;
    if relative.as_os_str().is_empty() {
        Ok(None)
    } else {
        Ok(Some(relative.to_path_buf()))
    }
}

fn git_tree_hash(repo_root: &Path, pack_path: Option<&Path>) -> Result<String> {
    let object = match pack_path {
        Some(path) if !path.as_os_str().is_empty() => {
            format!("HEAD:{}", path.to_string_lossy().replace('\\', "/"))
        }
        _ => "HEAD^{tree}".to_string(),
    };
    let output = Command::new("git")
        .current_dir(repo_root)
        .args(["rev-parse", &object])
        .output()
        .with_context(|| format!("failed to read tree hash for {}", repo_root.display()))?;
    if !output.status.success() {
        bail!("failed to read tree hash for {}", repo_root.display());
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

fn compute_pack_content_hash(path: &Path) -> Result<String> {
    let mut files = Vec::new();
    for entry in WalkDir::new(path) {
        let entry = entry.with_context(|| format!("failed to walk {}", path.display()))?;
        if !entry.file_type().is_file() {
            continue;
        }
        if entry
            .path()
            .components()
            .any(|component| component.as_os_str() == ".git")
        {
            continue;
        }
        files.push(entry.path().to_path_buf());
    }
    files.sort();

    let mut hasher = Sha256::new();
    for file in files {
        let relative = file.strip_prefix(path).unwrap_or(&file);
        hasher.update(relative.to_string_lossy().replace('\\', "/").as_bytes());
        hasher
            .update(fs::read(&file).with_context(|| format!("failed to read {}", file.display()))?);
    }
    Ok(format!("{:x}", hasher.finalize()))
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

    #[test]
    fn parses_github_tree_url() {
        let spec = parse_pack_spec(
            "python",
            "https://github.com/CorrectRoadH/harness-lint/tree/main/packs/python",
        );
        assert_eq!(spec.source, PackSourceKind::Git);
        assert_eq!(spec.spec, "CorrectRoadH/harness-lint#packs/python");
        assert_eq!(spec.version_req.as_deref(), Some("main"));
    }

    #[test]
    fn parses_github_shorthand_with_pack_path() {
        let spec = parse_pack_spec("python", "CorrectRoadH/harness-lint/packs/python@main");
        assert_eq!(spec.source, PackSourceKind::Git);
        assert_eq!(spec.spec, "CorrectRoadH/harness-lint#packs/python");
        assert_eq!(spec.version_req.as_deref(), Some("main"));
    }
}
