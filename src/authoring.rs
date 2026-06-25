use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};

use crate::config::USER_RULE_DIR;
use crate::model::CreatedRule;

pub fn create_rule(
    root: &Path,
    local_rule_dirs: &[PathBuf],
    feedback: &str,
    language: &str,
    grit: &str,
) -> Result<CreatedRule> {
    let id_tail = rule_name_from_feedback(feedback)?;
    let id = format!("local.{id_tail}");
    let title = feedback.trim().to_string();
    let language = language.trim();
    validate_language(language)?;
    let grit = normalize_grit(language, grit)?;
    crate::grit::validate_grit_pattern(&grit, language)?;
    let path = target_rule_dir(root, local_rule_dirs).join(format!("{id_tail}.md"));
    let content = format!(
        r#"---
id: {id}
title: {title:?}
language: {language}
level: warn
skill:
tags: [local, ai-feedback]
---

# {title}

{feedback}

```grit
{grit}
```

## Bad

```{language}
TODO: Add an example that should be flagged.
```

## Good

```{language}
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

fn validate_language(language: &str) -> Result<()> {
    if language.is_empty() {
        bail!("rule language is required; pass --language <language>");
    }
    if language
        .chars()
        .any(|ch| ch.is_control() || ch.is_whitespace())
    {
        bail!("rule language must be a single Grit language name without whitespace");
    }
    Ok(())
}

fn normalize_grit(language: &str, grit: &str) -> Result<String> {
    let grit = grit.trim();
    if !crate::rule::has_executable_grit(grit) {
        bail!(
            "rule GritQL is required; if the feedback cannot be expressed as GritQL, do not create a harness-lint rule"
        );
    }
    let expected_grit_language = grit_language_for_project_language(language);
    let mut replaced_language_line = false;
    let mut lines = Vec::new();
    for line in grit.lines() {
        let trimmed = line.trim_start();
        if !replaced_language_line
            && trimmed.to_ascii_lowercase().starts_with("language ")
            && let Some(actual_language) = trimmed.split_whitespace().nth(1)
        {
            let actual_grit_language = grit_language_for_project_language(
                actual_language
                    .split_once('(')
                    .map(|(base, _)| base)
                    .unwrap_or(actual_language),
            );
            if actual_grit_language != expected_grit_language {
                bail!(
                    "rule GritQL language `{actual_language}` does not match --language `{language}`"
                );
            }
            if actual_language == expected_grit_language || actual_language.starts_with("js(") {
                lines.push(line.to_string());
            } else {
                lines.push(format!("language {expected_grit_language}"));
            }
            replaced_language_line = true;
            continue;
        }
        lines.push(line.to_string());
    }
    if replaced_language_line {
        Ok(lines.join("\n"))
    } else {
        Ok(format!("language {expected_grit_language}\n{grit}"))
    }
}

fn grit_language_for_project_language(language: &str) -> String {
    match language.to_ascii_lowercase().as_str() {
        "typescript" | "ts" | "tsx" | "javascript" | "ecmascript" | "node" | "nodejs" | "js"
        | "jsx" | "mjs" | "cjs" => "js".to_string(),
        "python" | "py" => "python".to_string(),
        "golang" => "go".to_string(),
        "rust" | "rs" => "rust".to_string(),
        "ruby" | "rb" => "ruby".to_string(),
        "elixir" | "ex" | "exs" => "elixir".to_string(),
        "c#" | "cs" => "csharp".to_string(),
        "kotlin" | "kt" | "kts" => "kotlin".to_string(),
        "terraform" | "tf" => "hcl".to_string(),
        "solidity" | "sol" => "solidity".to_string(),
        "html" | "htm" => "html".to_string(),
        "markdown" | "md" => "markdown".to_string(),
        "yaml" | "yml" => "yaml".to_string(),
        other => other.to_string(),
    }
}

fn rule_name_from_feedback(feedback: &str) -> Result<String> {
    let trimmed = feedback.trim();
    if trimmed.is_empty() {
        bail!("rule feedback is empty; pass a short rule description");
    }

    let mut slug = String::new();
    let mut pending_dash = false;
    for (char_index, (byte_index, ch)) in trimmed.char_indices().enumerate() {
        if is_forbidden_rule_name_char(ch) {
            bail!(
                "rule feedback contains unsupported character `{}` at character {}, byte {}; remove path separators, control characters, and cross-platform filename punctuation",
                ch,
                char_index + 1,
                byte_index
            );
        }
        if ch.is_alphanumeric() {
            if pending_dash && !slug.is_empty() {
                slug.push('-');
            }
            for lower in ch.to_lowercase() {
                slug.push(lower);
            }
            pending_dash = false;
        } else {
            pending_dash = true;
        }
    }
    if slug.is_empty() {
        bail!("rule feedback does not contain any usable filename characters");
    }
    Ok(slug)
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

    fn grit_available() -> bool {
        crate::grit::ensure_grit_available().is_ok()
    }

    #[test]
    fn create_rule_creates_rule_file() {
        if !grit_available() {
            return;
        }
        let tempdir = tempfile::tempdir().unwrap();
        let created = create_rule(
            tempdir.path(),
            &[PathBuf::from("custom-rules")],
            "Prefer pydantic models",
            "python",
            "`print($value)`",
        )
        .unwrap();
        assert_eq!(created.id, "local.prefer-pydantic-models");
        assert_eq!(
            created.path,
            tempdir
                .path()
                .join("custom-rules/prefer-pydantic-models.md")
        );
        assert!(!created.content.contains("status:"));
        assert!(created.content.contains("language: python"));
        assert!(created.content.contains("language python\n`print($value)`"));
    }

    #[test]
    fn create_rule_maps_typescript_to_grit_js() {
        if !grit_available() {
            return;
        }
        let tempdir = tempfile::tempdir().unwrap();
        let created = create_rule(
            tempdir.path(),
            &[PathBuf::from("Rules")],
            "Avoid console logging",
            "typescript",
            "`console.log($value)`",
        )
        .unwrap();
        assert!(created.content.contains("language: typescript"));
        assert!(
            created
                .content
                .contains("language js\n`console.log($value)`")
        );
    }

    #[test]
    fn create_rule_preserves_supported_grit_language_variants() {
        if !grit_available() {
            return;
        }
        let tempdir = tempfile::tempdir().unwrap();
        let created = create_rule(
            tempdir.path(),
            &[PathBuf::from("Rules")],
            "Avoid console logging",
            "typescript",
            "language js(typescript)\n`console.log($value)`",
        )
        .unwrap();
        assert!(
            created
                .content
                .contains("language js(typescript)\n`console.log($value)`")
        );
    }

    #[test]
    fn create_rule_maps_common_language_aliases_to_grit_languages() {
        let cases = [
            ("tsx", "js"),
            ("nodejs", "js"),
            ("py", "python"),
            ("golang", "go"),
            ("rs", "rust"),
            ("rb", "ruby"),
            ("exs", "elixir"),
            ("cs", "csharp"),
            ("kt", "kotlin"),
            ("tf", "hcl"),
            ("sol", "solidity"),
            ("htm", "html"),
            ("md", "markdown"),
            ("yml", "yaml"),
        ];
        for (project_language, grit_language) in cases {
            assert_eq!(
                grit_language_for_project_language(project_language),
                grit_language
            );
        }
    }

    #[test]
    fn create_rule_slugifies_unicode_feedback_in_rule_name() {
        if !grit_available() {
            return;
        }
        let tempdir = tempfile::tempdir().unwrap();
        let created = create_rule(
            tempdir.path(),
            &[PathBuf::from("Rules")],
            "你好，不允许使用UI",
            "typescript",
            "language typescript\n`ReactDOM.render($value)`",
        )
        .unwrap();
        assert_eq!(created.id, "local.你好-不允许使用ui");
        assert_eq!(created.title, "你好，不允许使用UI");
        assert!(
            created
                .content
                .contains("language js\n`ReactDOM.render($value)`")
        );
        assert_eq!(
            created.path,
            tempdir.path().join("Rules/你好-不允许使用ui.md")
        );
    }

    #[test]
    fn create_rule_rejects_path_separators() {
        let tempdir = tempfile::tempdir().unwrap();
        let error = create_rule(
            tempdir.path(),
            &[PathBuf::from("Rules")],
            "不要用 UI/DOM",
            "typescript",
            "`console.log($value)`",
        )
        .unwrap_err()
        .to_string();
        assert!(error.contains("unsupported character `/`"));
    }

    #[test]
    fn create_rule_rejects_missing_gritql() {
        let tempdir = tempfile::tempdir().unwrap();
        let error = create_rule(
            tempdir.path(),
            &[PathBuf::from("Rules")],
            "Prefer pydantic models",
            "python",
            "// TODO",
        )
        .unwrap_err()
        .to_string();
        assert!(error.contains("rule GritQL is required"));
    }

    #[test]
    fn create_rule_rejects_invalid_gritql() {
        if !grit_available() {
            return;
        }
        let tempdir = tempfile::tempdir().unwrap();
        let error = create_rule(
            tempdir.path(),
            &[PathBuf::from("Rules")],
            "Avoid print debugging",
            "python",
            "`print($value)` where {\n  $filename <: r\".*src/.*\\.py\";\n}",
        )
        .unwrap_err()
        .to_string();
        assert!(error.contains("GritQL failed validation"));
    }

    #[test]
    fn create_rule_rejects_invalid_language() {
        let tempdir = tempfile::tempdir().unwrap();
        let error = create_rule(
            tempdir.path(),
            &[PathBuf::from("Rules")],
            "Prefer pydantic models",
            "python\nlevel: error",
            "`print($value)`",
        )
        .unwrap_err()
        .to_string();
        assert!(error.contains("single Grit language name"));
    }
}
