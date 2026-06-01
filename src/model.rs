use std::collections::BTreeMap;
use std::fmt;
use std::path::PathBuf;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    None,
    Info,
    Warn,
    Error,
}

impl Default for Severity {
    fn default() -> Self {
        Self::Warn
    }
}

impl Severity {
    pub fn is_failing(self) -> bool {
        matches!(self, Self::Error)
    }
}

impl FromStr for Severity {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.to_ascii_lowercase().as_str() {
            "none" => Ok(Self::None),
            "info" => Ok(Self::Info),
            "warn" | "warning" => Ok(Self::Warn),
            "error" => Ok(Self::Error),
            _ => Err(format!("unknown severity `{value}`")),
        }
    }
}

impl fmt::Display for Severity {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = match self {
            Self::None => "none",
            Self::Info => "info",
            Self::Warn => "warn",
            Self::Error => "error",
        };
        formatter.write_str(value)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RuleStatus {
    Draft,
    Warn,
    Enforced,
}

impl Default for RuleStatus {
    fn default() -> Self {
        Self::Draft
    }
}

impl FromStr for RuleStatus {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.to_ascii_lowercase().as_str() {
            "draft" => Ok(Self::Draft),
            "warn" | "warning" => Ok(Self::Warn),
            "enforced" | "error" => Ok(Self::Enforced),
            _ => Err(format!("unknown rule status `{value}`")),
        }
    }
}

impl fmt::Display for RuleStatus {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = match self {
            Self::Draft => "draft",
            Self::Warn => "warn",
            Self::Enforced => "enforced",
        };
        formatter.write_str(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PackSourceKind {
    Local,
    Git,
    Cargo,
    Pip,
    Url,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PackSpec {
    pub id: String,
    pub source: PackSourceKind,
    pub spec: String,
    pub version_req: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LockEntry {
    pub id: String,
    pub source: PackSourceKind,
    pub spec: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub requested_ref: Option<String>,
    pub version: Option<String>,
    pub checksum: Option<String>,
    pub local_path: PathBuf,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pack_path: Option<PathBuf>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedPack {
    pub spec: PackSpec,
    pub local_path: PathBuf,
    pub pack_path: Option<PathBuf>,
    pub version: Option<String>,
    pub checksum: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RulePack {
    pub id: String,
    pub name: String,
    pub version: String,
    pub rules: Vec<RuleDefinition>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuleDefinition {
    pub id: String,
    pub title: String,
    pub language: Option<String>,
    pub level: Severity,
    pub status: RuleStatus,
    pub skill: Option<String>,
    pub tags: Vec<String>,
    pub description: String,
    pub body: RuleBody,
    pub examples: Vec<RuleExample>,
    pub source_path: PathBuf,
    pub pack_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RuleBody {
    Grit(String),
    Missing,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuleExample {
    pub kind: RuleExampleKind,
    pub language: Option<String>,
    pub code: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuleExampleKind {
    Bad,
    Good,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Diagnostic {
    pub rule_id: String,
    pub level: Severity,
    pub message: String,
    pub path: PathBuf,
    pub start_line: u32,
    pub start_column: u32,
    pub end_line: Option<u32>,
    pub end_column: Option<u32>,
    pub fix_available: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectContext {
    pub root: PathBuf,
    pub config_path: Option<PathBuf>,
    pub languages: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuleDraft {
    pub id: String,
    pub title: String,
    pub path: PathBuf,
    pub content: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompiledRules {
    pub grit_dir: PathBuf,
    pub grit_rules: Vec<RuleDefinition>,
    pub skipped_drafts: Vec<RuleDefinition>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct HarnessLock {
    #[serde(default)]
    pub packs: BTreeMap<String, LockEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RegistryQuery {
    pub feedback: String,
    pub languages: Vec<String>,
    pub libraries: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RegistryCandidate {
    pub rule_id: String,
    pub title: String,
    pub pack_id: String,
    pub pack_spec: String,
    pub score: u32,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RegistryPack {
    pub id: String,
    pub title: String,
    pub description: String,
    pub pack_spec: String,
    pub languages: Vec<String>,
    pub keywords: Vec<String>,
    pub rules: Vec<RegistryCandidate>,
}
