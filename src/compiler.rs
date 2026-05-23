use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::Serialize;

use crate::config::GENERATED_GRIT_DIR;
use crate::model::{CompiledRules, RuleBody, RuleDefinition, RulePack, RuleStatus, Severity};

#[derive(Debug, Serialize)]
struct GritYaml {
    version: String,
    patterns: Vec<GritPatternEntry>,
}

#[derive(Debug, Serialize)]
struct GritPatternEntry {}

pub fn compile_grit_rules(
    root: &Path,
    packs: Vec<RulePack>,
    overrides: &BTreeMap<String, Severity>,
    disabled: &[String],
) -> Result<CompiledRules> {
    let grit_dir = root.join(GENERATED_GRIT_DIR);
    let patterns_dir = grit_dir.join("patterns");
    if patterns_dir.exists() {
        fs::remove_dir_all(&patterns_dir)
            .with_context(|| format!("failed to clear {}", patterns_dir.display()))?;
    }
    fs::create_dir_all(&patterns_dir)
        .with_context(|| format!("failed to create {}", patterns_dir.display()))?;

    let mut by_id: BTreeMap<String, RuleDefinition> = BTreeMap::new();
    for pack in packs {
        for mut rule in pack.rules {
            if disabled.iter().any(|disabled| disabled == &rule.id) {
                continue;
            }
            if let Some(level) = overrides.get(&rule.id) {
                rule.level = *level;
            }
            by_id.insert(rule.id.clone(), rule);
        }
    }

    let mut grit_rules = Vec::new();
    let mut skipped_drafts = Vec::new();
    for rule in by_id.into_values() {
        if rule.status == RuleStatus::Draft {
            skipped_drafts.push(rule);
            continue;
        }
        if matches!(rule.body, RuleBody::Grit(_)) {
            write_grit_pattern(&patterns_dir, &rule)?;
            grit_rules.push(rule);
        }
    }

    let yaml = GritYaml {
        version: "0.0.2".to_string(),
        patterns: Vec::new(),
    };
    let yaml = serde_yaml::to_string(&yaml).context("failed to serialize generated grit.yaml")?;
    fs::write(grit_dir.join("grit.yaml"), yaml)
        .with_context(|| format!("failed to write {}", grit_dir.join("grit.yaml").display()))?;

    Ok(CompiledRules {
        grit_dir,
        grit_rules,
        skipped_drafts,
    })
}

fn write_grit_pattern(patterns_dir: &Path, rule: &RuleDefinition) -> Result<()> {
    let path = patterns_dir.join(format!("{}.md", safe_pattern_filename(&rule.id)));
    let body = match &rule.body {
        RuleBody::Grit(body) => body,
        _ => return Ok(()),
    };
    let tags = if rule.tags.is_empty() {
        "[]".to_string()
    } else {
        format!(
            "[{}]",
            rule.tags
                .iter()
                .map(|tag| format!("\"{tag}\""))
                .collect::<Vec<_>>()
                .join(", ")
        )
    };
    let level = match rule.level {
        Severity::None => "none",
        Severity::Info => "info",
        Severity::Warn => "warn",
        Severity::Error => "error",
    };
    let content = format!(
        "---\ntitle: \"{}\"\nlevel: {level}\ntags: {tags}\n---\n\n# {}\n\n{}\n\n```grit\n{}\n```\n",
        rule.title.replace('"', "\\\""),
        rule.title,
        rule.description,
        body
    );
    fs::write(&path, content).with_context(|| format!("failed to write {}", path.display()))?;
    Ok(())
}

fn safe_pattern_filename(rule_id: &str) -> String {
    let filename: String = rule_id
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || ch == '_' {
                ch
            } else {
                '_'
            }
        })
        .collect();
    if filename
        .chars()
        .next()
        .map(|ch| ch.is_ascii_alphabetic() || ch == '_')
        .unwrap_or(false)
    {
        filename
    } else {
        format!("rule_{filename}")
    }
}

pub fn generated_grit_yaml_path(root: &Path) -> PathBuf {
    root.join(GENERATED_GRIT_DIR).join("grit.yaml")
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use crate::model::{RuleBody, RulePack, RuleStatus};

    use super::*;

    #[test]
    fn compiler_skips_drafts_and_writes_grit_yaml() {
        let tempdir = tempfile::tempdir().unwrap();
        let pack = RulePack {
            id: "local".to_string(),
            name: "Local".to_string(),
            version: "0.0.0".to_string(),
            rules: vec![
                RuleDefinition {
                    id: "local.warn".to_string(),
                    title: "Warn".to_string(),
                    language: Some("python".to_string()),
                    level: Severity::Warn,
                    status: RuleStatus::Warn,
                    tags: vec!["local".to_string()],
                    description: "desc".to_string(),
                    body: RuleBody::Grit("language python\n`print($x)`".to_string()),
                    examples: vec![],
                    source_path: PathBuf::from("warn.md"),
                    pack_id: Some("local".to_string()),
                },
                RuleDefinition {
                    id: "local.draft".to_string(),
                    title: "Draft".to_string(),
                    language: Some("python".to_string()),
                    level: Severity::Warn,
                    status: RuleStatus::Draft,
                    tags: vec![],
                    description: String::new(),
                    body: RuleBody::Missing,
                    examples: vec![],
                    source_path: PathBuf::from("draft.md"),
                    pack_id: Some("local".to_string()),
                },
            ],
        };
        let compiled =
            compile_grit_rules(tempdir.path(), vec![pack], &BTreeMap::new(), &[]).unwrap();
        assert_eq!(compiled.grit_rules.len(), 1);
        assert_eq!(compiled.skipped_drafts.len(), 1);
        assert!(generated_grit_yaml_path(tempdir.path()).exists());
        let yaml = std::fs::read_to_string(generated_grit_yaml_path(tempdir.path())).unwrap();
        assert!(yaml.contains("patterns: []"));
    }
}
