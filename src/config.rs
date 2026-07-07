use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};

use crate::model::{HarnessLock, Severity};

pub const CONFIG_FILE: &str = "harness.toml";
pub const LOCK_FILE: &str = "harness.lock";
/// Stable URL of the migration guide. Deprecation/legacy-construct warnings link
/// here so an AI agent (via the harness-lint skill) can fetch it and apply the
/// matching migration automatically.
pub const MIGRATION_GUIDE_URL: &str =
    "https://github.com/CorrectRoadH/harness-lint/blob/main/MIGRATE.md";
/// Stable URL of the "what's new" guide: per-version feature highlights with
/// explicit "when to adopt / when not to" guidance. Surfaced by `harness-lint
/// whatsnew` and consulted by the harness-lint skill on setup/review/upgrade.
pub const WHATS_NEW_URL: &str =
    "https://github.com/CorrectRoadH/harness-lint/blob/main/WHATS-NEW.md";
pub const USER_RULE_DIR: &str = "rules";
pub const WORK_DIR: &str = ".harness";
pub const PACKS_DIR: &str = ".harness/packs";
pub const REPOS_DIR: &str = ".harness/repos";
pub const GENERATED_GRIT_DIR: &str = ".harness/generated/.grit";
pub const CACHE_DIR: &str = ".harness/cache";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ProjectConfig {
    #[serde(default)]
    pub project: ProjectSection,
    #[serde(default)]
    pub lint: LintSection,
    #[serde(default)]
    pub rules: RulesSection,
    #[serde(default)]
    pub packs: BTreeMap<String, String>,
    #[serde(default)]
    pub overrides: BTreeMap<String, Severity>,
    #[serde(default)]
    pub disabled: DisabledSection,
    #[serde(default)]
    pub ignore: IgnoreSection,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub file_sets: BTreeMap<String, FileSetSection>,
    #[serde(default, alias = "suppressions", skip_serializing_if = "Vec::is_empty")]
    pub exceptions: Vec<RuleExceptionSection>,
    #[serde(default)]
    pub registry: RegistrySection,
    #[serde(skip)]
    pub used_legacy_exceptions_key: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ProjectSection {
    pub name: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LintSection {
    pub default_level: Severity,
    pub changed_base: String,
    pub cache: bool,
}

impl Default for LintSection {
    fn default() -> Self {
        Self {
            default_level: Severity::Warn,
            changed_base: "origin/main".to_string(),
            cache: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RulesSection {
    pub local: Vec<PathBuf>,
}

impl Default for RulesSection {
    fn default() -> Self {
        Self {
            local: vec![PathBuf::from(USER_RULE_DIR)],
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct DisabledSection {
    #[serde(default)]
    pub rules: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct IgnoreSection {
    #[serde(default)]
    pub paths: Vec<String>,
}

/// A named region of the repository that a rule can opt into with `runs_on`.
///
/// Unlike `[ignore]` (never scanned by anyone), a file set is *scannable* by
/// rules that name it. With `default_rules = false` it is removed from the
/// `default` region, so ordinary rules (no `runs_on`) skip it while opted-in
/// rules still reach it. `provides` lists portable concept names a pack rule
/// may reference, decoupling the project's chosen set name from pack vocabulary.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FileSetSection {
    #[serde(default)]
    pub paths: Vec<String>,
    /// Whether rules without an explicit `runs_on` still scan this region.
    /// `false` makes the set default-closed (the typical generated-code case).
    #[serde(default = "default_true")]
    pub default_rules: bool,
    /// Portable concept names this set satisfies (e.g. `generated`). A rule
    /// `runs_on = ["generated"]` binds to every set whose `provides` lists it.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub provides: Vec<String>,
}

impl Default for FileSetSection {
    fn default() -> Self {
        Self {
            paths: Vec::new(),
            default_rules: true,
            provides: Vec::new(),
        }
    }
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuleExceptionSection {
    pub rule: String,
    #[serde(default)]
    pub paths: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RegistrySection {
    pub url: Option<String>,
}

impl Default for RegistrySection {
    fn default() -> Self {
        Self {
            url: Some(
                "https://raw.githubusercontent.com/CorrectRoadH/harness-lint/main/site/catalog.json"
                    .to_string(),
            ),
        }
    }
}

pub fn find_project_root(start: &Path) -> Result<PathBuf> {
    let mut current = start
        .canonicalize()
        .with_context(|| format!("failed to resolve {}", start.display()))?;
    loop {
        if current.join(CONFIG_FILE).exists() || current.join(".git").exists() {
            return Ok(current);
        }
        if !current.pop() {
            bail!(
                "could not find project root from {}; run `harness-lint init` first",
                start.display()
            );
        }
    }
}

pub fn load_config(root: &Path, explicit: Option<&Path>) -> Result<ProjectConfig> {
    let path = explicit
        .map(PathBuf::from)
        .unwrap_or_else(|| root.join(CONFIG_FILE));
    if !path.exists() {
        bail!(
            "missing {}; run `harness-lint init` in {}",
            path.display(),
            root.display()
        );
    }
    let content =
        fs::read_to_string(&path).with_context(|| format!("failed to read {}", path.display()))?;
    let mut config: ProjectConfig =
        toml::from_str(&content).with_context(|| format!("failed to parse {}", path.display()))?;
    if uses_legacy_suppressions_key(&content) {
        config.used_legacy_exceptions_key = true;
        eprintln!(
            "warning: `[[suppressions]]` is deprecated; rename it to `[[exceptions]]` in {}; \
             see {MIGRATION_GUIDE_URL}",
            path.display()
        );
    }
    if uses_unsupported_scan_ignored_key(&content) {
        eprintln!(
            "warning: `[[scan_ignored]]` in {} is not a supported key and is silently ignored; \
             migrate to `[file_sets]` + rule `runs_on` — see {MIGRATION_GUIDE_URL}",
            path.display()
        );
    }
    Ok(config)
}

fn uses_legacy_suppressions_key(content: &str) -> bool {
    content.lines().any(|line| {
        let trimmed = line.trim();
        trimmed == "[suppressions]" || trimmed == "[[suppressions]]"
    })
}

fn uses_unsupported_scan_ignored_key(content: &str) -> bool {
    content.lines().any(|line| {
        let trimmed = line.trim();
        trimmed == "[scan_ignored]" || trimmed == "[[scan_ignored]]"
    })
}

pub fn write_config(root: &Path, config: &ProjectConfig) -> Result<()> {
    let path = root.join(CONFIG_FILE);
    let content = toml::to_string_pretty(config).context("failed to serialize harness.toml")?;
    write_atomic(&path, content).with_context(|| format!("failed to write {}", path.display()))?;
    Ok(())
}

pub fn load_lock(root: &Path) -> Result<HarnessLock> {
    let path = root.join(LOCK_FILE);
    if !path.exists() {
        return Ok(HarnessLock::default());
    }
    let content =
        fs::read_to_string(&path).with_context(|| format!("failed to read {}", path.display()))?;
    let lock =
        toml::from_str(&content).with_context(|| format!("failed to parse {}", path.display()))?;
    Ok(lock)
}

pub fn write_lock(root: &Path, lock: &HarnessLock) -> Result<()> {
    let path = root.join(LOCK_FILE);
    let content = toml::to_string_pretty(lock).context("failed to serialize harness.lock")?;
    write_atomic(&path, content).with_context(|| format!("failed to write {}", path.display()))?;
    Ok(())
}

fn write_atomic(path: &Path, content: String) -> Result<()> {
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("harness-lint");
    let temp_path = path.with_file_name(format!(".{file_name}.{}.tmp", std::process::id()));
    fs::write(&temp_path, content)
        .with_context(|| format!("failed to write {}", temp_path.display()))?;
    if let Err(error) = fs::rename(&temp_path, path) {
        let _ = fs::remove_file(&temp_path);
        return Err(error).with_context(|| {
            format!(
                "failed to replace {} with {}",
                path.display(),
                temp_path.display()
            )
        });
    }
    Ok(())
}

pub fn default_config(project_name: Option<String>) -> ProjectConfig {
    let mut config = ProjectConfig::default();
    config.project.name = project_name;
    config
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_project_config() {
        let config: ProjectConfig = toml::from_str(
            r#"
[project]
name = "demo"

[lint]
default_level = "error"
changed_base = "main"
cache = false

[rules]
local = ["rules"]

[packs]
python = "local:../rules"

[overrides]
"python.x" = "warn"

[disabled]
rules = ["python.y"]

[ignore]
paths = ["dist/**"]

[[exceptions]]
rule = "python.z"
paths = ["generated/**"]
reason = "Generated adapters intentionally use this pattern."
"#,
        )
        .unwrap();
        assert_eq!(config.project.name.as_deref(), Some("demo"));
        assert_eq!(config.lint.default_level, Severity::Error);
        assert_eq!(config.rules.local, vec![PathBuf::from("rules")]);
        assert_eq!(config.packs["python"], "local:../rules");
        assert_eq!(config.disabled.rules, vec!["python.y"]);
        assert_eq!(config.exceptions.len(), 1);
        assert_eq!(config.exceptions[0].rule, "python.z");
        assert_eq!(config.exceptions[0].paths, vec!["generated/**"]);
        assert_eq!(
            config.exceptions[0].reason.as_deref(),
            Some("Generated adapters intentionally use this pattern.")
        );
    }

    #[test]
    fn parses_file_sets() {
        let config: ProjectConfig = toml::from_str(
            r#"
[file_sets.generated]
paths = ["apps/backend/gen/**/*.pb.go", "packages/proto/gen/**"]
default_rules = false
provides = ["generated"]

[file_sets.api]
paths = ["apps/api/**"]
"#,
        )
        .unwrap();
        let generated = &config.file_sets["generated"];
        assert_eq!(generated.paths.len(), 2);
        assert!(!generated.default_rules);
        assert_eq!(generated.provides, vec!["generated"]);
        // default_rules defaults to true; provides defaults to empty.
        let api = &config.file_sets["api"];
        assert!(api.default_rules);
        assert!(api.provides.is_empty());
    }

    #[test]
    fn parses_legacy_suppressions_key() {
        let config: ProjectConfig = toml::from_str(
            r#"
[[suppressions]]
rule = "python.z"
paths = ["generated/**"]
"#,
        )
        .unwrap();

        assert_eq!(config.exceptions.len(), 1);
        assert_eq!(config.exceptions[0].rule, "python.z");
        assert_eq!(config.exceptions[0].paths, vec!["generated/**"]);
    }

    #[test]
    fn load_config_marks_legacy_suppressions_key() {
        let tempdir = tempfile::tempdir().unwrap();
        fs::write(
            tempdir.path().join(CONFIG_FILE),
            r#"
[[suppressions]]
rule = "python.z"
paths = ["generated/**"]
"#,
        )
        .unwrap();

        let config = load_config(tempdir.path(), None).unwrap();

        assert!(config.used_legacy_exceptions_key);
        assert_eq!(config.exceptions.len(), 1);
    }

    #[test]
    fn detects_unsupported_scan_ignored_key() {
        assert!(uses_unsupported_scan_ignored_key(
            "[[scan_ignored]]\nrule = \"x\"\n"
        ));
        assert!(uses_unsupported_scan_ignored_key("[scan_ignored]\n"));
        assert!(!uses_unsupported_scan_ignored_key("[ignore]\npaths = []\n"));
    }

    #[test]
    fn missing_config_points_to_harness_lint_init() {
        let tempdir = tempfile::tempdir().unwrap();
        let error = load_config(tempdir.path(), None).unwrap_err().to_string();
        assert!(error.contains("harness-lint init"));
    }
}
