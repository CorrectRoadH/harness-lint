use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::Serialize;

use crate::config::GENERATED_GRIT_DIR;
use crate::model::{CompiledRules, RuleBody, RuleDefinition, Severity};

#[derive(Debug, Serialize)]
struct GritYaml {
    version: String,
    patterns: Vec<GritPatternEntry>,
}

#[derive(Debug, Serialize)]
struct GritPatternEntry {}

pub fn compile_rule_set(root: &Path, rules: Vec<RuleDefinition>) -> Result<CompiledRules> {
    let grit_dir = root.join(GENERATED_GRIT_DIR);
    let patterns_dir = grit_dir.join("patterns");
    fs::create_dir_all(&patterns_dir)
        .with_context(|| format!("failed to create {}", patterns_dir.display()))?;

    let mut grit_rules = Vec::new();
    let mut expected_files = std::collections::BTreeSet::new();
    for rule in rules {
        let filename = format!("{}.md", safe_pattern_filename(&rule.id));
        write_grit_pattern(&patterns_dir, &filename, &rule)?;
        expected_files.insert(filename);
        grit_rules.push(rule);
    }

    remove_stale_patterns(&patterns_dir, &expected_files)?;
    write_if_changed(&grit_dir.join("grit.yaml"), &grit_yaml()?)?;

    Ok(CompiledRules {
        grit_dir,
        grit_rules,
    })
}

fn grit_yaml() -> Result<String> {
    let yaml = GritYaml {
        version: "0.0.2".to_string(),
        patterns: Vec::new(),
    };
    serde_yaml::to_string(&yaml).context("failed to serialize generated grit.yaml")
}

#[derive(Serialize)]
struct GritPatternFrontmatter<'a> {
    title: &'a str,
    level: &'a str,
    tags: &'a [String],
}

fn write_grit_pattern(patterns_dir: &Path, filename: &str, rule: &RuleDefinition) -> Result<()> {
    let path = patterns_dir.join(filename);
    let RuleBody::Grit(body) = &rule.body;
    let level = match rule.level {
        Severity::None => "none",
        Severity::Info => "info",
        Severity::Warn => "warn",
        Severity::Error => "error",
    };
    // serde_yaml handles quoting/escaping; hand-rolled escaping broke on
    // titles containing backslashes or quotes.
    let frontmatter = serde_yaml::to_string(&GritPatternFrontmatter {
        title: &rule.title,
        level,
        tags: &rule.tags,
    })
    .with_context(|| format!("failed to serialize frontmatter for rule {}", rule.id))?;
    let content = format!(
        "---\n{frontmatter}---\n\n# {}\n\n{}\n\n```grit\n{}\n```\n",
        rule.title, rule.description, body
    );
    write_if_changed(&path, &content)?;
    Ok(())
}

fn write_if_changed(path: &Path, content: &str) -> Result<()> {
    if fs::read_to_string(path).ok().as_deref() == Some(content) {
        return Ok(());
    }
    fs::write(path, content).with_context(|| format!("failed to write {}", path.display()))
}

fn remove_stale_patterns(
    patterns_dir: &Path,
    expected_files: &std::collections::BTreeSet<String>,
) -> Result<()> {
    for entry in fs::read_dir(patterns_dir)
        .with_context(|| format!("failed to read {}", patterns_dir.display()))?
    {
        let entry = entry.with_context(|| format!("failed to read {}", patterns_dir.display()))?;
        if !entry.file_type()?.is_file() {
            continue;
        }
        let filename = entry.file_name().to_string_lossy().to_string();
        if filename.ends_with(".md") && !expected_files.contains(&filename) {
            match fs::remove_file(entry.path()) {
                Ok(()) => {}
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
                Err(error) => {
                    return Err(error)
                        .with_context(|| format!("failed to remove {}", entry.path().display()));
                }
            }
        }
    }
    Ok(())
}

pub(crate) fn safe_pattern_filename(rule_id: &str) -> String {
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
    use crate::model::RuleBody;

    use super::*;

    #[test]
    fn compiler_writes_grit_rules_and_grit_yaml() {
        let tempdir = tempfile::tempdir().unwrap();
        let rules = vec![RuleDefinition {
            id: "local.warn".to_string(),
            title: "Warn".to_string(),
            language: Some("python".to_string()),
            level: Severity::Warn,
            skill: None,
            tags: vec!["local".to_string()],
            runs_on: vec![],
            description: "desc".to_string(),
            body: RuleBody::Grit("language python\n`print($x)`".to_string()),
            examples: vec![],
            source_path: PathBuf::from("warn.md"),
            pack_id: Some("local".to_string()),
        }];
        let compiled = compile_rule_set(tempdir.path(), rules).unwrap();
        assert_eq!(compiled.grit_rules.len(), 1);
        assert!(generated_grit_yaml_path(tempdir.path()).exists());
        let yaml = std::fs::read_to_string(generated_grit_yaml_path(tempdir.path())).unwrap();
        assert!(yaml.contains("patterns: []"));
    }
}
