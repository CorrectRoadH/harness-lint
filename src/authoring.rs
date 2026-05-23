use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use heck::{ToKebabCase, ToTitleCase};

use crate::config::USER_RULE_DIR;
use crate::model::{RuleDefinition, RuleDraft, RuleStatus};

pub fn suggest_rule(root: &Path, feedback: &str) -> Result<RuleDraft> {
    let id_tail = feedback
        .chars()
        .filter(|ch| ch.is_ascii_alphanumeric() || ch.is_whitespace() || *ch == '-' || *ch == '_')
        .collect::<String>()
        .split_whitespace()
        .take(8)
        .collect::<Vec<_>>()
        .join(" ")
        .to_kebab_case();
    let id_tail = if id_tail.is_empty() {
        "custom-preference".to_string()
    } else {
        id_tail
    };
    let id = format!("local.{id_tail}");
    let title = id_tail.replace('-', " ").to_title_case();
    let path = root.join(USER_RULE_DIR).join(format!("{id_tail}.md"));
    let content = format!(
        r#"---
id: {id}
title: {title:?}
level: warn
status: draft
tags: [local, ai-feedback]
fixable: false
---

# {title}

{feedback}

```grit
// TODO: Add GritQL once the matching shape is clear.
```

## Bad

```text
TODO: Add an example that should be flagged.
```

## Good

```text
TODO: Add an example that should be allowed.
```
"#
    );

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }
    fs::write(&path, &content).with_context(|| format!("failed to write {}", path.display()))?;

    Ok(RuleDraft {
        id,
        title,
        path,
        content,
    })
}

pub fn new_rule(root: &Path, id: &str, title: &str, language: Option<&str>) -> Result<RuleDraft> {
    let filename = id
        .strip_prefix("local.")
        .unwrap_or(id)
        .replace(['.', '/'], "-");
    let path = root.join(USER_RULE_DIR).join(format!("{filename}.md"));
    let language_line = language
        .map(|language| format!("language: {language}\n"))
        .unwrap_or_default();
    let id = if id.contains('.') {
        id.to_string()
    } else {
        format!("local.{id}")
    };
    let content = format!(
        r#"---
id: {id}
title: {title:?}
{language_line}level: warn
status: draft
tags: [local]
fixable: false
---

# {title}

TODO: Explain this rule.

```grit
// TODO: Add GritQL once the matching shape is clear.
```

## Bad

```text
TODO: Add an example that should be flagged.
```

## Good

```text
TODO: Add an example that should be allowed.
```
"#
    );
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }
    fs::write(&path, &content).with_context(|| format!("failed to write {}", path.display()))?;
    Ok(RuleDraft {
        id,
        title: title.to_string(),
        path,
        content,
    })
}

pub fn set_rule_status(rule: &RuleDefinition, status: RuleStatus) -> Result<()> {
    let content = fs::read_to_string(&rule.source_path)
        .with_context(|| format!("failed to read {}", rule.source_path.display()))?;
    let mut changed = false;
    let mut lines = Vec::new();
    for line in content.lines() {
        if line.trim_start().starts_with("status:") {
            lines.push(format!("status: {status}"));
            changed = true;
        } else {
            lines.push(line.to_string());
        }
    }
    let next = if changed {
        lines.join("\n") + "\n"
    } else if let Some(index) = content.find("\n---\n") {
        let mut next = content.clone();
        next.insert_str(index, &format!("\nstatus: {status}"));
        next
    } else {
        content
    };
    fs::write(&rule.source_path, next)
        .with_context(|| format!("failed to write {}", rule.source_path.display()))?;
    Ok(())
}

pub fn add_example(rule: &RuleDefinition, kind: &str, language: &str, code: &str) -> Result<()> {
    let heading = match kind {
        "bad" | "Bad" => "Bad",
        "good" | "Good" => "Good",
        _ => "Bad",
    };
    let mut content = fs::read_to_string(&rule.source_path)
        .with_context(|| format!("failed to read {}", rule.source_path.display()))?;
    if !content.ends_with('\n') {
        content.push('\n');
    }
    content.push_str(&format!("\n## {heading}\n\n```{language}\n{code}\n```\n"));
    fs::write(&rule.source_path, content)
        .with_context(|| format!("failed to write {}", rule.source_path.display()))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn suggest_rule_creates_draft_file() {
        let tempdir = tempfile::tempdir().unwrap();
        let draft = suggest_rule(tempdir.path(), "Prefer pydantic models").unwrap();
        assert_eq!(draft.id, "local.prefer-pydantic-models");
        assert!(draft.path.exists());
        assert!(draft.content.contains("status: draft"));
    }
}
