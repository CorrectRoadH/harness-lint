use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use globset::{Glob, GlobSet, GlobSetBuilder};
use ignore::WalkBuilder;

use crate::config::{CACHE_DIR, GENERATED_GRIT_DIR, PACKS_DIR, WORK_DIR};
use crate::model::RuleDefinition;

pub fn discover_all_files(
    root: &Path,
    ignore_patterns: &[String],
    rule_dirs: &[PathBuf],
) -> Result<Vec<PathBuf>> {
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
        if is_internal_path(relative, rule_dirs) || ignore_set.is_match(relative) {
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
    rule_dirs: &[PathBuf],
) -> Result<Vec<PathBuf>> {
    let ignore_set = build_ignore_set(ignore_patterns)?;
    let mut filtered = Vec::new();
    for path in paths {
        if is_internal_path(&path, rule_dirs) || ignore_set.is_match(&path) {
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
    match language.to_ascii_lowercase().as_str() {
        "python" | "py" => ext == "py",
        "javascript" | "ecmascript" | "node" | "nodejs" | "js" => {
            matches!(ext, "js" | "jsx" | "mjs" | "cjs")
        }
        "jsx" => ext == "jsx",
        "typescript" | "ts" => matches!(ext, "ts" | "tsx"),
        "tsx" => ext == "tsx",
        "rust" => ext == "rs",
        "go" | "golang" => ext == "go",
        "ruby" | "rb" => ext == "rb",
        "elixir" | "ex" | "exs" => matches!(ext, "ex" | "exs"),
        "csharp" | "c#" | "cs" => ext == "cs",
        "java" => ext == "java",
        "kotlin" | "kt" | "kts" => matches!(ext, "kt" | "kts"),
        "solidity" | "sol" => ext == "sol",
        "hcl" => ext == "hcl",
        "terraform" | "tf" => ext == "tf",
        "html" | "htm" => matches!(ext, "html" | "htm"),
        "css" => ext == "css",
        "markdown" | "md" => ext == "md",
        "yaml" | "yml" => matches!(ext, "yaml" | "yml"),
        "json" => ext == "json",
        "toml" => ext == "toml",
        "sql" => ext == "sql",
        "vue" => ext == "vue",
        "php" => ext == "php",
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

fn is_internal_path(path: &Path, rule_dirs: &[PathBuf]) -> bool {
    path.starts_with(".git")
        || path.starts_with(".obsidian")
        || path.starts_with(WORK_DIR)
        || path.starts_with(PACKS_DIR)
        || path.starts_with(GENERATED_GRIT_DIR)
        || path.starts_with(CACHE_DIR)
        || path.starts_with("rules")
        || path.starts_with("harness/rules")
        || path.starts_with("target")
        || path.starts_with("node_modules")
        || path.starts_with(".venv")
        || rule_dirs
            .iter()
            .filter(|dir| !dir.is_absolute())
            .any(|dir| path.starts_with(dir))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{RuleBody, Severity};

    #[test]
    fn filters_by_language_and_internal_paths() {
        let rule = RuleDefinition {
            id: "python.x".to_string(),
            title: "x".to_string(),
            language: Some("python".to_string()),
            level: Severity::Warn,
            skill: None,
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
                PathBuf::from(".harness/generated/.grit/grit.yaml"),
            ],
            &[],
            &[rule],
            &[],
        )
        .unwrap();
        assert_eq!(paths, vec![PathBuf::from("src/main.py")]);
    }

    #[test]
    fn matches_common_grit_languages_and_aliases() {
        let mut rule = RuleDefinition {
            id: "local.x".to_string(),
            title: "x".to_string(),
            language: None,
            level: Severity::Warn,
            skill: None,
            tags: vec![],
            description: String::new(),
            body: RuleBody::Grit(String::new()),
            examples: vec![],
            source_path: PathBuf::from("rule.md"),
            pack_id: None,
        };
        let cases = [
            ("typescript", "src/main.ts"),
            ("tsx", "src/main.tsx"),
            ("javascript", "src/main.jsx"),
            ("python", "src/main.py"),
            ("go", "src/main.go"),
            ("rust", "src/main.rs"),
            ("ruby", "src/main.rb"),
            ("elixir", "src/main.exs"),
            ("csharp", "src/main.cs"),
            ("java", "src/Main.java"),
            ("kotlin", "src/Main.kts"),
            ("solidity", "src/Main.sol"),
            ("hcl", "infra/main.hcl"),
            ("terraform", "infra/main.tf"),
            ("html", "src/index.htm"),
            ("css", "src/main.css"),
            ("markdown", "README.md"),
            ("yaml", "config.yml"),
            ("json", "package.json"),
            ("toml", "Cargo.toml"),
            ("sql", "query.sql"),
            ("vue", "App.vue"),
            ("php", "index.php"),
        ];
        for (language, path) in cases {
            rule.language = Some(language.to_string());
            assert!(
                rule_matches_path(&rule, Path::new(path)),
                "{language} should match {path}"
            );
        }
    }
}
