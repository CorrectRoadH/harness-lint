use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use globset::{Glob, GlobSet, GlobSetBuilder};
use ignore::WalkBuilder;

use crate::config::{CACHE_DIR, GENERATED_GRIT_DIR, PACKS_DIR, ProjectConfig, WORK_DIR};
use crate::model::RuleDefinition;

/// Precompiled view of `[file_sets.*]` used to resolve a rule's `runs_on`
/// against individual paths. Built once per run.
///
/// Membership is layered: structural exclusions and `[ignore]` are removed
/// first (those files never reach here). What remains is partitioned by file
/// set; a path is in the implicit `default` region exactly when no
/// `default_rules = false` set claims it. A rule's `runs_on` then selects which
/// regions it scans (empty `runs_on` = `default`).
#[derive(Debug, Default)]
pub struct FileSetIndex {
    /// File set name -> compiled globs.
    sets: BTreeMap<String, GlobSet>,
    /// Globs of `default_rules = false` sets, used to compute `default`.
    closed: Vec<GlobSet>,
    /// Concept name -> file set names that `provides` it.
    concepts: BTreeMap<String, Vec<String>>,
}

impl FileSetIndex {
    pub fn build(config: &ProjectConfig) -> Result<Self> {
        let mut sets = BTreeMap::new();
        let mut closed = Vec::new();
        let mut concepts: BTreeMap<String, Vec<String>> = BTreeMap::new();
        for (name, section) in &config.file_sets {
            let set = build_ignore_set(&section.paths)
                .with_context(|| format!("invalid glob in [file_sets.{name}]"))?;
            if !section.default_rules {
                closed.push(set.clone());
            }
            for concept in &section.provides {
                concepts
                    .entry(concept.clone())
                    .or_default()
                    .push(name.clone());
            }
            sets.insert(name.clone(), set);
        }
        Ok(Self {
            sets,
            closed,
            concepts,
        })
    }

    /// A path is in the implicit `default` region when no default-closed set
    /// claims it.
    fn in_default(&self, path: &Path) -> bool {
        !self.closed.iter().any(|set| set.is_match(path))
    }

    /// Whether one `runs_on` target (a file set name, a provided concept, or the
    /// literal `default`) matches a path. Unresolved targets match nothing; a
    /// separate health check reports them.
    fn target_matches(&self, target: &str, path: &Path) -> bool {
        if target == "default" {
            return self.in_default(path);
        }
        if let Some(set) = self.sets.get(target) {
            return set.is_match(path);
        }
        if let Some(names) = self.concepts.get(target) {
            return names
                .iter()
                .any(|name| self.sets.get(name).is_some_and(|set| set.is_match(path)));
        }
        false
    }

    /// Whether a rule with the given `runs_on` scans a path. Empty `runs_on`
    /// means the `default` region.
    pub fn rule_runs_on_path(&self, runs_on: &[String], path: &Path) -> bool {
        if runs_on.is_empty() {
            return self.in_default(path);
        }
        runs_on
            .iter()
            .any(|target| self.target_matches(target, path))
    }
}

/// Whether a rule scans a path: its `language` must match the file type *and*
/// its `runs_on` region must contain the path.
pub fn rule_scans_path(rule: &RuleDefinition, path: &Path, index: &FileSetIndex) -> bool {
    rule_matches_path(rule, path) && index.rule_runs_on_path(&rule.runs_on, path)
}

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
    index: &FileSetIndex,
) -> Result<Vec<PathBuf>> {
    let ignore_set = build_ignore_set(ignore_patterns)?;
    let mut filtered = Vec::new();
    for path in paths {
        if is_internal_path(&path, rule_dirs) || ignore_set.is_match(&path) {
            continue;
        }
        if rules.is_empty() || rules.iter().any(|rule| rule_scans_path(rule, &path, index)) {
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
            runs_on: vec![],
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
            &FileSetIndex::default(),
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
            runs_on: vec![],
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

    fn rule_with(runs_on: Vec<&str>) -> RuleDefinition {
        RuleDefinition {
            id: "local.x".to_string(),
            title: "x".to_string(),
            language: Some("go".to_string()),
            level: Severity::Warn,
            skill: None,
            tags: vec![],
            runs_on: runs_on.into_iter().map(str::to_string).collect(),
            description: String::new(),
            body: RuleBody::Grit(String::new()),
            examples: vec![],
            source_path: PathBuf::from("rule.md"),
            pack_id: None,
        }
    }

    fn config_with_generated() -> ProjectConfig {
        use crate::config::FileSetSection;
        let mut config = ProjectConfig::default();
        config.file_sets.insert(
            "codegen".to_string(),
            FileSetSection {
                paths: vec![
                    "apps/**/gen/**".to_string(),
                    "packages/proto/gen/**".to_string(),
                ],
                default_rules: false,
                provides: vec!["generated".to_string()],
            },
        );
        config
    }

    #[test]
    fn default_region_excludes_closed_file_sets() {
        let index = FileSetIndex::build(&config_with_generated()).unwrap();
        let default_rule = rule_with(vec![]);
        // Ordinary source: in default.
        assert!(rule_scans_path(
            &default_rule,
            Path::new("apps/backend/server.go"),
            &index
        ));
        // Generated code: removed from default, so a default rule skips it.
        assert!(!rule_scans_path(
            &default_rule,
            Path::new("apps/backend/gen/user.pb.go"),
            &index
        ));
    }

    #[test]
    fn runs_on_by_set_name_and_concept_reach_generated() {
        let index = FileSetIndex::build(&config_with_generated()).unwrap();
        let gen_path = Path::new("packages/proto/gen/user.pb.go");
        let src_path = Path::new("apps/backend/server.go");

        // By local file-set name.
        let by_name = rule_with(vec!["codegen"]);
        assert!(rule_scans_path(&by_name, gen_path, &index));
        assert!(!rule_scans_path(&by_name, src_path, &index));

        // By portable concept the set `provides`.
        let by_concept = rule_with(vec!["generated"]);
        assert!(rule_scans_path(&by_concept, gen_path, &index));
        assert!(!rule_scans_path(&by_concept, src_path, &index));
    }

    #[test]
    fn runs_on_default_plus_generated_is_additive() {
        let index = FileSetIndex::build(&config_with_generated()).unwrap();
        let rule = rule_with(vec!["default", "generated"]);
        assert!(rule_scans_path(
            &rule,
            Path::new("apps/backend/server.go"),
            &index
        ));
        assert!(rule_scans_path(
            &rule,
            Path::new("apps/backend/gen/user.pb.go"),
            &index
        ));
    }

    #[test]
    fn unresolved_run_target_matches_nothing() {
        let index = FileSetIndex::build(&config_with_generated()).unwrap();
        let rule = rule_with(vec!["migrations"]);
        assert!(!rule_scans_path(
            &rule,
            Path::new("apps/backend/gen/user.pb.go"),
            &index
        ));
        assert!(!rule_scans_path(
            &rule,
            Path::new("db/migrations/001.go"),
            &index
        ));
    }

    #[test]
    fn open_file_set_keeps_default_membership() {
        use crate::config::FileSetSection;
        let mut config = ProjectConfig::default();
        config.file_sets.insert(
            "api".to_string(),
            FileSetSection {
                paths: vec!["apps/api/**".to_string()],
                default_rules: true,
                provides: vec![],
            },
        );
        let index = FileSetIndex::build(&config).unwrap();
        // default_rules = true: still part of default for ordinary rules.
        assert!(rule_scans_path(
            &rule_with(vec![]),
            Path::new("apps/api/handler.go"),
            &index
        ));
        // And reachable by name for rules that want just this region.
        assert!(rule_scans_path(
            &rule_with(vec!["api"]),
            Path::new("apps/api/handler.go"),
            &index
        ));
    }
}
