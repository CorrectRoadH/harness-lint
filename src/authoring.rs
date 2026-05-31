use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use heck::{ToKebabCase, ToTitleCase};

use crate::config::USER_RULE_DIR;
use crate::model::RuleDraft;

pub fn suggest_rule(root: &Path, local_rule_dirs: &[PathBuf], feedback: &str) -> Result<RuleDraft> {
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
    let path = target_rule_dir(root, local_rule_dirs).join(format!("{id_tail}.md"));
    let content = format!(
        r#"---
id: {id}
title: {title:?}
level: warn
status: draft
skill:
tags: [local, ai-feedback]
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

pub fn new_rule(
    root: &Path,
    local_rule_dirs: &[PathBuf],
    id: &str,
    title: &str,
    language: Option<&str>,
) -> Result<RuleDraft> {
    let filename = id
        .strip_prefix("local.")
        .unwrap_or(id)
        .replace(['.', '/'], "-");
    let path = target_rule_dir(root, local_rule_dirs).join(format!("{filename}.md"));
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
skill:
tags: [local]
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

fn target_rule_dir(root: &Path, local_rule_dirs: &[PathBuf]) -> PathBuf {
    let dir = local_rule_dirs
        .first()
        .cloned()
        .unwrap_or_else(|| PathBuf::from(USER_RULE_DIR));
    if dir.is_absolute() {
        dir
    } else {
        root.join(dir)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn suggest_rule_creates_draft_file() {
        let tempdir = tempfile::tempdir().unwrap();
        let draft = suggest_rule(
            tempdir.path(),
            &[PathBuf::from("custom-rules")],
            "Prefer pydantic models",
        )
        .unwrap();
        assert_eq!(draft.id, "local.prefer-pydantic-models");
        assert_eq!(
            draft.path,
            tempdir
                .path()
                .join("custom-rules/prefer-pydantic-models.md")
        );
        assert!(draft.content.contains("status: draft"));
    }
}
