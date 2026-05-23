use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};

use crate::model::{HarnessLock, Severity};

pub const CONFIG_FILE: &str = "harness.toml";
pub const LOCK_FILE: &str = "harness.lock";
pub const USER_RULE_DIR: &str = "harness/rules/local";
pub const WORK_DIR: &str = ".harness";
pub const PACKS_DIR: &str = ".harness/packs";
pub const GENERATED_GRIT_DIR: &str = ".harness/generated/.grit";
pub const CACHE_DIR: &str = ".harness/cache";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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
    #[serde(default)]
    pub registry: RegistrySection,
}

impl Default for ProjectConfig {
    fn default() -> Self {
        Self {
            project: ProjectSection::default(),
            lint: LintSection::default(),
            rules: RulesSection::default(),
            packs: BTreeMap::new(),
            overrides: BTreeMap::new(),
            disabled: DisabledSection::default(),
            ignore: IgnoreSection::default(),
            registry: RegistrySection::default(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectSection {
    pub name: Option<String>,
}

impl Default for ProjectSection {
    fn default() -> Self {
        Self { name: None }
    }
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RegistrySection {
    pub url: Option<String>,
}

impl Default for RegistrySection {
    fn default() -> Self {
        Self {
            url: Some("https://registry.harness-lint.dev".to_string()),
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
                "could not find project root from {}; run `harness init` first",
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
            "missing {}; run `harness init` in {}",
            path.display(),
            root.display()
        );
    }
    let content =
        fs::read_to_string(&path).with_context(|| format!("failed to read {}", path.display()))?;
    let config =
        toml::from_str(&content).with_context(|| format!("failed to parse {}", path.display()))?;
    Ok(config)
}

pub fn write_config(root: &Path, config: &ProjectConfig) -> Result<()> {
    let path = root.join(CONFIG_FILE);
    let content = toml::to_string_pretty(config).context("failed to serialize harness.toml")?;
    fs::write(&path, content).with_context(|| format!("failed to write {}", path.display()))?;
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
    fs::write(&path, content).with_context(|| format!("failed to write {}", path.display()))?;
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
"#,
        )
        .unwrap();
        assert_eq!(config.project.name.as_deref(), Some("demo"));
        assert_eq!(config.lint.default_level, Severity::Error);
        assert_eq!(config.rules.local, vec![PathBuf::from("rules")]);
        assert_eq!(config.packs["python"], "local:../rules");
        assert_eq!(config.disabled.rules, vec!["python.y"]);
    }
}
