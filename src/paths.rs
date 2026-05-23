use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use globset::{Glob, GlobSet, GlobSetBuilder};
use ignore::WalkBuilder;

use crate::config::{CACHE_DIR, GENERATED_GRIT_DIR, PACKS_DIR, WORK_DIR};
use crate::model::RuleDefinition;

pub fn discover_all_files(root: &Path, ignore_patterns: &[String]) -> Result<Vec<PathBuf>> {
    let ignore_set = build_ignore_set(ignore_patterns)?;
    let mut files = Vec::new();
    for entry in WalkBuilder::new(root)
        .hidden(false)
        .git_ignore(true)
        .git_exclude(true)
        .build()
    {
        let entry = entry.context("failed to walk project files")?;
        if !entry
            .file_type()
            .map(|kind| kind.is_file())
            .unwrap_or(false)
        {
            continue;
        }
        let path = entry.path();
        let relative = path.strip_prefix(root).unwrap_or(path);
        if is_internal_path(relative) || ignore_set.is_match(relative) {
            continue;
        }
        files.push(relative.to_path_buf());
    }
    files.sort();
    Ok(files)
}

pub fn filter_paths(
    paths: Vec<PathBuf>,
    ignore_patterns: &[String],
    rules: &[RuleDefinition],
) -> Result<Vec<PathBuf>> {
    let ignore_set = build_ignore_set(ignore_patterns)?;
    let mut filtered = Vec::new();
    for path in paths {
        if is_internal_path(&path) || ignore_set.is_match(&path) {
            continue;
        }
        if rules.is_empty() || rules.iter().any(|rule| rule_matches_path(rule, &path)) {
            filtered.push(path);
        }
    }
    filtered.sort();
    filtered.dedup();
    Ok(filtered)
}

pub fn rule_matches_path(rule: &RuleDefinition, path: &Path) -> bool {
    let Some(language) = &rule.language else {
        return true;
    };
    let ext = path.extension().and_then(|ext| ext.to_str()).unwrap_or("");
    match language.as_str() {
        "python" => ext == "py",
        "javascript" | "js" => matches!(ext, "js" | "jsx" | "mjs" | "cjs"),
        "typescript" | "ts" => matches!(ext, "ts" | "tsx"),
        "rust" => ext == "rs",
        "go" => ext == "go",
        "markdown" | "md" => ext == "md",
        "yaml" => matches!(ext, "yaml" | "yml"),
        "json" => ext == "json",
        "toml" => ext == "toml",
        "svg" => ext == "svg",
        "text" => true,
        _ => true,
    }
}

fn build_ignore_set(patterns: &[String]) -> Result<GlobSet> {
    let mut builder = GlobSetBuilder::new();
    for pattern in patterns {
        builder.add(Glob::new(pattern).with_context(|| format!("invalid ignore glob {pattern}"))?);
    }
    builder.build().context("failed to compile ignore patterns")
}

fn is_internal_path(path: &Path) -> bool {
    path.starts_with(".git")
        || path.starts_with(WORK_DIR)
        || path.starts_with(PACKS_DIR)
        || path.starts_with(GENERATED_GRIT_DIR)
        || path.starts_with(CACHE_DIR)
        || path.starts_with("harness/rules")
        || path.starts_with("target")
        || path.starts_with("node_modules")
        || path.starts_with(".venv")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{RuleBody, RuleStatus, Severity};

    #[test]
    fn filters_by_language_and_internal_paths() {
        let rule = RuleDefinition {
            id: "python.x".to_string(),
            title: "x".to_string(),
            language: Some("python".to_string()),
            level: Severity::Warn,
            status: RuleStatus::Warn,
            tags: vec![],
            description: String::new(),
            body: RuleBody::Grit(String::new()),
            examples: vec![],
            source_path: PathBuf::from("rule.md"),
            pack_id: None,
        };
        let paths = filter_paths(
            vec![
                PathBuf::from("src/main.py"),
                PathBuf::from("src/main.rs"),
                PathBuf::from(".harness/generated/grit/grit.yaml"),
            ],
            &[],
            &[rule],
        )
        .unwrap();
        assert_eq!(paths, vec![PathBuf::from("src/main.py")]);
    }
}
