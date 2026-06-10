use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, anyhow, bail};
use serde::Deserialize;
use walkdir::WalkDir;

use crate::model::{RuleBody, RuleDefinition, RuleExample, RuleExampleKind, Severity};

#[derive(Debug, Deserialize)]
struct RuleFrontmatter {
    id: String,
    title: String,
    #[serde(default)]
    language: Option<String>,
    #[serde(default)]
    level: Severity,
    #[serde(default)]
    skill: Option<String>,
    #[serde(default)]
    tags: Vec<String>,
}

pub fn discover_rules(dir: &Path, pack_id: Option<&str>) -> Result<Vec<RuleDefinition>> {
    if !dir.exists() {
        return Ok(Vec::new());
    }

    let mut rules = Vec::new();
    for entry in WalkDir::new(dir).into_iter().filter_map(Result::ok) {
        let path = entry.path();
        if !entry.file_type().is_file()
            || path.extension().and_then(|ext| ext.to_str()) != Some("md")
        {
            continue;
        }
        rules.push(parse_rule_file(path, pack_id)?);
    }
    rules.sort_by(|left, right| left.id.cmp(&right.id));
    Ok(rules)
}

pub fn parse_rule_file(path: &Path, pack_id: Option<&str>) -> Result<RuleDefinition> {
    let content =
        fs::read_to_string(path).with_context(|| format!("failed to read {}", path.display()))?;
    parse_rule(&content, path.to_path_buf(), pack_id)
}

pub fn parse_rule(
    content: &str,
    source_path: PathBuf,
    pack_id: Option<&str>,
) -> Result<RuleDefinition> {
    let (frontmatter, markdown) = split_frontmatter(content)
        .ok_or_else(|| anyhow!("{} is missing YAML frontmatter", source_path.display()))?;
    let frontmatter: RuleFrontmatter = serde_yaml::from_str(frontmatter)
        .with_context(|| format!("failed to parse frontmatter in {}", source_path.display()))?;

    let description = extract_description(markdown);
    let grit_blocks = grit_fenced_codes(markdown);
    let grit_block_count = grit_blocks.len();
    let body = extract_body(grit_blocks, &source_path)?;
    let examples = extract_examples(markdown);

    let rule = RuleDefinition {
        id: frontmatter.id,
        title: frontmatter.title,
        language: frontmatter.language,
        level: frontmatter.level,
        skill: frontmatter.skill,
        tags: frontmatter.tags,
        description,
        body,
        examples,
        source_path,
        pack_id: pack_id.map(ToOwned::to_owned),
    };

    validate_rule(&rule, grit_block_count)?;
    Ok(rule)
}

fn split_frontmatter(content: &str) -> Option<(&str, &str)> {
    let rest = content.strip_prefix("---\n")?;
    let end = rest.find("\n---")?;
    let (frontmatter, after) = rest.split_at(end);
    let markdown = after.strip_prefix("\n---\n").unwrap_or(after);
    Some((frontmatter, markdown))
}

fn extract_description(markdown: &str) -> String {
    let mut description = Vec::new();
    for line in markdown.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') || trimmed.starts_with("```") {
            if !description.is_empty() {
                break;
            }
            continue;
        }
        description.push(trimmed.to_string());
    }
    description.join(" ")
}

fn extract_body(grit_blocks: Vec<String>, source_path: &Path) -> Result<RuleBody> {
    if let Some(code) = grit_blocks.into_iter().next() {
        if has_executable_grit(&code) {
            return Ok(RuleBody::Grit(code));
        }
        bail!(
            "{} has a ```grit block but no executable GritQL",
            source_path.display()
        );
    }

    bail!(
        "{} is missing an executable ```grit block; every rule must be enforceable with GritQL",
        source_path.display()
    )
}

pub(crate) fn has_executable_grit(code: &str) -> bool {
    code.lines().any(|line| {
        let trimmed = line.trim();
        !trimmed.is_empty()
            && !trimmed.starts_with("//")
            && !trimmed.starts_with('#')
            && !trimmed.starts_with("/*")
            && !trimmed.starts_with('*')
            && !trimmed.starts_with("*/")
            && !trimmed.to_ascii_uppercase().contains("TODO")
    })
}

fn extract_examples(markdown: &str) -> Vec<RuleExample> {
    let mut examples = Vec::new();
    let mut current = None;
    let lines: Vec<_> = markdown.lines().collect();
    let mut index = 0;
    while index < lines.len() {
        let trimmed = lines[index].trim();
        if trimmed.eq_ignore_ascii_case("## bad") {
            current = Some(RuleExampleKind::Bad);
        } else if trimmed.eq_ignore_ascii_case("## good") {
            current = Some(RuleExampleKind::Good);
        } else if trimmed.starts_with("```") {
            if let Some(kind) = current {
                let language = trimmed.trim_start_matches("```").trim();
                let language = (!language.is_empty()).then(|| language.to_string());
                let mut code = Vec::new();
                index += 1;
                while index < lines.len() && !lines[index].trim_start().starts_with("```") {
                    code.push(lines[index]);
                    index += 1;
                }
                examples.push(RuleExample {
                    kind,
                    language,
                    code: code.join("\n"),
                });
            }
        }
        index += 1;
    }
    examples
}

fn grit_fenced_codes(markdown: &str) -> Vec<String> {
    let mut matches = Vec::new();
    let lines: Vec<_> = markdown.lines().collect();
    let mut index = 0;
    while index < lines.len() {
        let trimmed = lines[index].trim();
        if trimmed.starts_with("```") {
            let fence_language = trimmed.trim_start_matches("```").trim();
            let mut code = Vec::new();
            index += 1;
            while index < lines.len() && !lines[index].trim_start().starts_with("```") {
                code.push(lines[index]);
                index += 1;
            }
            if fence_language.eq_ignore_ascii_case("grit") {
                matches.push(code.join("\n"));
            }
        }
        index += 1;
    }
    matches
}

fn validate_rule(rule: &RuleDefinition, grit_block_count: usize) -> Result<()> {
    if rule.id.trim().is_empty() {
        bail!("{} has an empty rule id", rule.source_path.display());
    }
    if rule.title.trim().is_empty() {
        bail!("{} has an empty rule title", rule.source_path.display());
    }
    if grit_block_count > 1 {
        bail!(
            "{} has {grit_block_count} GritQL blocks; keep exactly one ```grit block per rule file",
            rule.source_path.display()
        );
    }
    if rule.level.is_failing() {
        let has_bad = rule
            .examples
            .iter()
            .any(|example| example.kind == RuleExampleKind::Bad);
        let has_good = rule
            .examples
            .iter()
            .any(|example| example.kind == RuleExampleKind::Good);
        if !has_bad || !has_good {
            bail!(
                "{} is error-level but does not include both Bad and Good examples",
                rule.source_path.display()
            );
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_rule_markdown() {
        let content = r#"---
id: python.prefer-pydantic
title: Prefer Pydantic
language: python
level: warn
skill: tdd
tags: [python]
---

# Prefer Pydantic

Prefer Pydantic for structured validation.

```grit
language python
`print($x)`
```

## Bad

```python
print("x")
```

## Good

```python
logger.info("x")
```
"#;

        let rule = parse_rule(content, PathBuf::from("rule.md"), Some("python")).unwrap();
        assert_eq!(rule.id, "python.prefer-pydantic");
        assert_eq!(rule.pack_id.as_deref(), Some("python"));
        assert_eq!(rule.examples.len(), 2);
        assert!(matches!(rule.body, RuleBody::Grit(_)));
    }

    #[test]
    fn rejects_rule_without_gritql() {
        let content = r#"---
id: ai.some-rule
title: Some Rule
---

# Some Rule

TODO.
"#;

        let error = parse_rule(content, PathBuf::from("rule.md"), None)
            .unwrap_err()
            .to_string();
        assert!(error.contains("missing an executable ```grit block"));
    }

    #[test]
    fn rejects_comment_only_grit_block() {
        let content = r#"---
id: ai.some-rule
title: Some Rule
---

# Some Rule

TODO.

```grit
// TODO: write this GritQL once the target code stabilizes.
```
"#;

        let error = parse_rule(content, PathBuf::from("rule.md"), None)
            .unwrap_err()
            .to_string();
        assert!(error.contains("has a ```grit block but no executable GritQL"));
    }

    #[test]
    fn ignores_unknown_frontmatter_fields() {
        let content = r#"---
id: ai.some-rule
title: Some Rule
status: draft
owner: local-team
---

# Some Rule

TODO.

```grit
language js
`console.log($value)`
```
"#;

        let rule = parse_rule(content, PathBuf::from("rule.md"), None).unwrap();
        assert_eq!(rule.id, "ai.some-rule");
        assert_eq!(rule.title, "Some Rule");
    }

    #[test]
    fn rejects_multiple_gritql_blocks() {
        let content = r#"---
id: ai.some-rule
title: Some Rule
---

# Some Rule

```grit
language js
`console.log($value)`
```

```grit
language js
`debugger`
```
"#;

        let error = parse_rule(content, PathBuf::from("rule.md"), None)
            .unwrap_err()
            .to_string();
        assert!(error.contains("has 2 GritQL blocks"));
    }
}
