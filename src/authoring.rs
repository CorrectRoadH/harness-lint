use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};

use crate::config::USER_RULE_DIR;
use crate::model::CreatedRule;

pub fn create_rule(
    root: &Path,
    local_rule_dirs: &[PathBuf],
    feedback: &str,
) -> Result<CreatedRule> {
    let id_tail = rule_name_from_feedback(feedback)?;
    let id = format!("local.{id_tail}");
    let title = feedback.trim().to_string();
    let path = target_rule_dir(root, local_rule_dirs).join(format!("{id_tail}.md"));
    let content = format!(
        r#"---
id: {id}
title: {title:?}
level: warn
skill:
tags: [local, ai-feedback]
---

# {title}

{feedback}

TODO: Add GritQL once the matching shape is clear.

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

    Ok(CreatedRule {
        id,
        title,
        path,
        content,
    })
}

fn rule_name_from_feedback(feedback: &str) -> Result<String> {
    let trimmed = feedback.trim();
    if trimmed.is_empty() {
        bail!("rule feedback is empty; pass a short rule description");
    }

    for (char_index, (byte_index, ch)) in trimmed.char_indices().enumerate() {
        if is_forbidden_rule_name_char(ch) {
            bail!(
                "rule feedback contains unsupported character `{}` at character {}, byte {}; remove path separators, control characters, and cross-platform filename punctuation",
                ch,
                char_index + 1,
                byte_index
            );
        }
    }
    Ok(trimmed.to_string())
}

fn is_forbidden_rule_name_char(ch: char) -> bool {
    ch.is_control() || matches!(ch, '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|')
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
    fn create_rule_creates_rule_file() {
        let tempdir = tempfile::tempdir().unwrap();
        let created = create_rule(
            tempdir.path(),
            &[PathBuf::from("custom-rules")],
            "Prefer pydantic models",
        )
        .unwrap();
        assert_eq!(created.id, "local.Prefer pydantic models");
        assert_eq!(
            created.path,
            tempdir
                .path()
                .join("custom-rules/Prefer pydantic models.md")
        );
        assert!(!created.content.contains("status:"));
    }

    #[test]
    fn create_rule_preserves_unicode_feedback_in_rule_name() {
        let tempdir = tempfile::tempdir().unwrap();
        let created = create_rule(
            tempdir.path(),
            &[PathBuf::from("Rules")],
            "你好，不允许使用UI",
        )
        .unwrap();
        assert_eq!(created.id, "local.你好，不允许使用UI");
        assert_eq!(created.title, "你好，不允许使用UI");
        assert_eq!(
            created.path,
            tempdir.path().join("Rules/你好，不允许使用UI.md")
        );
    }

    #[test]
    fn create_rule_rejects_path_separators() {
        let tempdir = tempfile::tempdir().unwrap();
        let error = create_rule(tempdir.path(), &[PathBuf::from("Rules")], "不要用 UI/DOM")
            .unwrap_err()
            .to_string();
        assert!(error.contains("unsupported character `/`"));
    }
}
