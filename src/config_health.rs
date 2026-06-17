//! Repository-level integrity checks for harness-lint's own configuration.
//!
//! GritQL rules only see a single source file's AST, so they cannot tell when a
//! configured path stops existing. When an `[[exceptions]]` or `[ignore]` path
//! is renamed or deleted, the entry silently stops matching: suppressed
//! diagnostics quietly come back, or an ignore no longer covers anything. These
//! checks read harness.toml and flag the stale entries through the normal
//! diagnostic pipeline.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use crate::config::{CONFIG_FILE, ProjectConfig};
use crate::model::{Diagnostic, RuleDefinition, Severity};

pub const STALE_EXCEPTION_PATH_RULE: &str = "harness.stale-exception-path";
pub const STALE_IGNORE_PATH_RULE: &str = "harness.stale-ignore-path";
pub const UNKNOWN_DISABLED_RULE: &str = "harness.unknown-disabled-rule";
pub const UNKNOWN_OVERRIDE_RULE: &str = "harness.unknown-override-rule";
pub const STALE_FILE_SET_PATH_RULE: &str = "harness.stale-file-set-path";
pub const EMPTY_FILE_SET_RULE: &str = "harness.empty-file-set";
pub const FILE_SET_IGNORE_OVERLAP_RULE: &str = "harness.file-set-ignore-overlap";
pub const UNKNOWN_RUN_TARGET_RULE: &str = "harness.unknown-run-target";

const GLOB_META: &[char] = &['*', '?', '[', ']', '{', '}'];

/// The literal path prefix of a glob pattern, taken up to (but excluding) the
/// first path component that contains a glob metacharacter.
///
/// `backend/internal/cache/**` -> `backend/internal/cache`
/// `backend/cmd/server/main.go` -> `backend/cmd/server/main.go`
/// `**/*_test.go` -> `None` (no anchorable prefix to verify)
fn literal_prefix(pattern: &str) -> Option<PathBuf> {
    let mut prefix = PathBuf::new();
    for component in pattern.split('/') {
        if component.is_empty() {
            continue;
        }
        if component.contains(GLOB_META) {
            break;
        }
        prefix.push(component);
    }
    if prefix.as_os_str().is_empty() {
        None
    } else {
        Some(prefix)
    }
}

/// True when the literal prefix of `pattern` does not exist under `root`.
/// Patterns without a literal prefix (a leading glob) are never stale because
/// there is nothing concrete to anchor against.
fn is_stale(root: &Path, pattern: &str) -> bool {
    match literal_prefix(pattern.trim()) {
        Some(prefix) => !root.join(prefix).exists(),
        None => false,
    }
}

/// A configuration diagnostic anchored at `harness.toml`. `default_level` is
/// used unless `[overrides]` re-levels the id.
fn config_diagnostic(
    config: &ProjectConfig,
    rule_id: &str,
    default_level: Severity,
    message: String,
) -> Diagnostic {
    let level = config
        .overrides
        .get(rule_id)
        .copied()
        .unwrap_or(default_level);
    Diagnostic {
        rule_id: rule_id.to_string(),
        level,
        message,
        path: PathBuf::from(CONFIG_FILE),
        start_line: 1,
        start_column: 1,
        end_line: None,
        end_column: None,
        fix_available: false,
    }
}

fn stale_diagnostic(config: &ProjectConfig, rule_id: &str, message: String) -> Diagnostic {
    config_diagnostic(config, rule_id, Severity::Warn, message)
}

/// Check that every non-glob path referenced by `[[exceptions]]` and `[ignore]`
/// still exists. Returns one diagnostic per stale entry, anchored at
/// `harness.toml`. Severity defaults to `warn` and honours `[overrides]`.
pub fn check_config_paths(root: &Path, config: &ProjectConfig) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();

    for exception in &config.exceptions {
        for pattern in &exception.paths {
            if is_stale(root, pattern) {
                diagnostics.push(stale_diagnostic(
                    config,
                    STALE_EXCEPTION_PATH_RULE,
                    format!(
                        "exception path `{pattern}` for rule `{}` no longer exists; \
                         the exception is silently dead and its diagnostics have returned",
                        exception.rule
                    ),
                ));
            }
        }
    }

    for pattern in &config.ignore.paths {
        if is_stale(root, pattern) {
            diagnostics.push(stale_diagnostic(
                config,
                STALE_IGNORE_PATH_RULE,
                format!("ignore path `{pattern}` no longer exists"),
            ));
        }
    }

    for (name, section) in &config.file_sets {
        if section.paths.is_empty() {
            diagnostics.push(config_diagnostic(
                config,
                EMPTY_FILE_SET_RULE,
                Severity::Error,
                format!(
                    "file set `{name}` has no `paths`; it matches nothing and any rule that \
                     `runs_on` it scans no files"
                ),
            ));
        }
        for pattern in &section.paths {
            if is_stale(root, pattern) {
                diagnostics.push(stale_diagnostic(
                    config,
                    STALE_FILE_SET_PATH_RULE,
                    format!(
                        "file set `{name}` path `{pattern}` no longer exists; \
                         rules that `runs_on` it silently scan nothing there"
                    ),
                ));
            }
            if let Some(overlap) = ignore_overlap(pattern, &config.ignore.paths) {
                diagnostics.push(config_diagnostic(
                    config,
                    FILE_SET_IGNORE_OVERLAP_RULE,
                    Severity::Error,
                    format!(
                        "file set `{name}` path `{pattern}` overlaps `[ignore]` path `{overlap}`; \
                         `[ignore]` is never scanned, so the file set can never be reached — \
                         move the path out of `[ignore]`"
                    ),
                ));
            }
        }
    }

    diagnostics
}

/// The first `[ignore]` pattern whose literal prefix is a path-prefix of (or is
/// prefixed by) `pattern`'s literal prefix — i.e. the two cover overlapping
/// directory trees. Leading-glob patterns on either side have no anchor and are
/// skipped.
fn ignore_overlap(pattern: &str, ignore_patterns: &[String]) -> Option<String> {
    let file_set_prefix = literal_prefix(pattern.trim())?;
    ignore_patterns.iter().find_map(|ignore| {
        let ignore_prefix = literal_prefix(ignore.trim())?;
        (file_set_prefix.starts_with(&ignore_prefix) || ignore_prefix.starts_with(&file_set_prefix))
            .then(|| ignore.clone())
    })
}

/// Flag rule `runs_on` targets that resolve to nothing: not the literal
/// `default`, not a defined `[file_sets.<name>]`, and not a concept any file set
/// `provides`. After a pack update introduces a rule that expects a concept the
/// project never wired up, the rule silently scans nothing. Anchored at
/// `harness.toml` (the fix is a `[file_sets.*]` entry) and defaults to `error`.
pub fn check_run_targets(config: &ProjectConfig, rules: &[&RuleDefinition]) -> Vec<Diagnostic> {
    let known = known_run_targets(config);
    let mut diagnostics = Vec::new();
    for rule in rules {
        for target in &rule.runs_on {
            if !known.contains(target.as_str()) {
                diagnostics.push(config_diagnostic(
                    config,
                    UNKNOWN_RUN_TARGET_RULE,
                    Severity::Error,
                    format!(
                        "rule `{}` runs_on `{target}`, but no file set is named `{target}` and \
                         none `provides` it; add `[file_sets.<name>]` with \
                         `provides = [\"{target}\"]` pointing at the right paths",
                        rule.id
                    ),
                ));
            }
        }
    }
    diagnostics
}

/// The set of valid `runs_on` targets: `default`, every file set name, and every
/// concept any file set `provides`.
fn known_run_targets(config: &ProjectConfig) -> BTreeSet<String> {
    let mut targets = BTreeSet::new();
    targets.insert("default".to_string());
    for (name, section) in &config.file_sets {
        targets.insert(name.clone());
        for concept in &section.provides {
            targets.insert(concept.clone());
        }
    }
    targets
}

/// Flag `[disabled]` and `[overrides]` entries that reference a rule id no
/// loaded pack or local rule provides. After a pack update drops or renames a
/// rule, such an entry stays in harness.toml but becomes silently inert: a
/// disabled rule that no longer disables anything, or an override that no
/// longer adjusts any severity. `known_rule_ids` is the set of ids across all
/// loaded packs and local rules.
pub fn check_unknown_rule_refs(
    config: &ProjectConfig,
    known_rule_ids: &BTreeSet<&str>,
) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();

    for rule_id in &config.disabled.rules {
        if !known_rule_ids.contains(rule_id.as_str()) {
            diagnostics.push(stale_diagnostic(
                config,
                UNKNOWN_DISABLED_RULE,
                format!(
                    "disabled rule `{rule_id}` is not provided by any loaded pack or local rule; \
                     the entry has no effect (was it dropped or renamed by a pack update?)"
                ),
            ));
        }
    }

    for rule_id in config.overrides.keys() {
        if !known_rule_ids.contains(rule_id.as_str()) {
            diagnostics.push(stale_diagnostic(
                config,
                UNKNOWN_OVERRIDE_RULE,
                format!(
                    "override for rule `{rule_id}` is not provided by any loaded pack or local rule; \
                     the entry has no effect (was it dropped or renamed by a pack update?)"
                ),
            ));
        }
    }

    diagnostics
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;

    use crate::config::{FileSetSection, IgnoreSection, RuleExceptionSection};
    use crate::model::{RuleBody, RuleDefinition};

    use super::*;

    fn rule_running_on(id: &str, runs_on: Vec<&str>) -> RuleDefinition {
        RuleDefinition {
            id: id.to_string(),
            title: id.to_string(),
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

    #[test]
    fn flags_empty_and_stale_and_overlapping_file_sets() {
        let tempdir = tempfile::tempdir().unwrap();
        let root = tempdir.path();
        fs::create_dir_all(root.join("apps/backend/gen")).unwrap();

        let mut config = ProjectConfig::default();
        config.ignore = IgnoreSection {
            paths: vec!["dist/**".to_string()],
        };
        config.file_sets.insert(
            "generated".to_string(),
            FileSetSection {
                paths: vec![
                    "apps/backend/gen/**".to_string(), // present
                    "services/foo/gen/**".to_string(), // stale
                ],
                default_rules: false,
                provides: vec!["generated".to_string()],
            },
        );
        config.file_sets.insert(
            "empty".to_string(),
            FileSetSection {
                paths: vec![],
                default_rules: false,
                provides: vec![],
            },
        );
        config.file_sets.insert(
            "build".to_string(),
            FileSetSection {
                paths: vec!["dist/generated/**".to_string()], // overlaps [ignore] dist/**
                default_rules: false,
                provides: vec![],
            },
        );

        let diagnostics = check_config_paths(root, &config);

        assert!(diagnostics.iter().any(|d| {
            d.rule_id == STALE_FILE_SET_PATH_RULE && d.message.contains("services/foo/gen/**")
        }));
        assert!(diagnostics.iter().any(|d| {
            d.rule_id == EMPTY_FILE_SET_RULE
                && d.message.contains("empty")
                && d.level == Severity::Error
        }));
        assert!(diagnostics.iter().any(|d| {
            d.rule_id == FILE_SET_IGNORE_OVERLAP_RULE
                && d.message.contains("dist/**")
                && d.level == Severity::Error
        }));
    }

    #[test]
    fn flags_unknown_run_targets() {
        let mut config = ProjectConfig::default();
        config.file_sets.insert(
            "codegen".to_string(),
            FileSetSection {
                paths: vec!["gen/**".to_string()],
                default_rules: false,
                provides: vec!["generated".to_string()],
            },
        );

        let known = rule_running_on("a.known-set", vec!["codegen"]);
        let concept = rule_running_on("a.known-concept", vec!["generated"]);
        let default = rule_running_on("a.default", vec!["default"]);
        let unknown = rule_running_on("a.unknown", vec!["migrations"]);
        let rules = vec![&known, &concept, &default, &unknown];

        let diagnostics = check_run_targets(&config, &rules);

        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].rule_id, UNKNOWN_RUN_TARGET_RULE);
        assert_eq!(diagnostics[0].level, Severity::Error);
        assert!(diagnostics[0].message.contains("a.unknown"));
        assert!(diagnostics[0].message.contains("migrations"));
    }

    #[test]
    fn literal_prefix_stops_at_first_glob() {
        assert_eq!(
            literal_prefix("backend/internal/cache/**"),
            Some(PathBuf::from("backend/internal/cache"))
        );
        assert_eq!(
            literal_prefix("backend/cmd/server/main.go"),
            Some(PathBuf::from("backend/cmd/server/main.go"))
        );
        assert_eq!(literal_prefix("**/*_test.go"), None);
        assert_eq!(
            literal_prefix("dist/build-*/out"),
            Some(PathBuf::from("dist"))
        );
    }

    #[test]
    fn flags_missing_exception_and_ignore_paths() {
        let tempdir = tempfile::tempdir().unwrap();
        let root = tempdir.path();
        fs::create_dir_all(root.join("backend/internal/cache")).unwrap();

        let mut config = ProjectConfig::default();
        config.exceptions = vec![
            RuleExceptionSection {
                rule: "go.no-panic".to_string(),
                // Present directory glob: not stale.
                paths: vec!["backend/internal/cache/**".to_string()],
                reason: None,
            },
            RuleExceptionSection {
                rule: "go.no-panic".to_string(),
                // Literal file that was moved away: stale.
                paths: vec!["backend/cmd/server/main.go".to_string()],
                reason: None,
            },
        ];
        config.ignore = IgnoreSection {
            paths: vec![
                "backend/internal/cache/**".to_string(), // present
                "frontend/src/lib/gen/**".to_string(),   // missing
                "**/*.snap".to_string(),                 // leading glob, skipped
            ],
        };

        let diagnostics = check_config_paths(root, &config);

        assert_eq!(diagnostics.len(), 2);
        assert!(diagnostics.iter().any(|d| {
            d.rule_id == STALE_EXCEPTION_PATH_RULE
                && d.message.contains("backend/cmd/server/main.go")
        }));
        assert!(diagnostics.iter().any(|d| {
            d.rule_id == STALE_IGNORE_PATH_RULE && d.message.contains("frontend/src/lib/gen/**")
        }));
        assert!(diagnostics.iter().all(|d| d.level == Severity::Warn));
    }

    #[test]
    fn overrides_can_escalate_severity() {
        let tempdir = tempfile::tempdir().unwrap();
        let root = tempdir.path();

        let mut config = ProjectConfig::default();
        config
            .overrides
            .insert(STALE_EXCEPTION_PATH_RULE.to_string(), Severity::Error);
        config.exceptions = vec![RuleExceptionSection {
            rule: "go.no-panic".to_string(),
            paths: vec!["does/not/exist.go".to_string()],
            reason: None,
        }];

        let diagnostics = check_config_paths(root, &config);

        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].level, Severity::Error);
    }

    #[test]
    fn flags_unknown_disabled_and_override_rules() {
        let mut config = ProjectConfig::default();
        config.disabled.rules = vec!["go.known".to_string(), "go.dropped".to_string()];
        config
            .overrides
            .insert("go.known".to_string(), Severity::Error);
        config
            .overrides
            .insert("go.gone".to_string(), Severity::Warn);

        let known: BTreeSet<&str> = ["go.known"].into_iter().collect();
        let diagnostics = check_unknown_rule_refs(&config, &known);

        assert_eq!(diagnostics.len(), 2);
        assert!(
            diagnostics.iter().any(|d| {
                d.rule_id == UNKNOWN_DISABLED_RULE && d.message.contains("go.dropped")
            })
        );
        assert!(
            diagnostics
                .iter()
                .any(|d| { d.rule_id == UNKNOWN_OVERRIDE_RULE && d.message.contains("go.gone") })
        );
    }
}
