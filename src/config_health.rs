//! Repository-level integrity checks for harness-lint's own configuration.
//!
//! GritQL rules only see a single source file's AST, so they cannot tell when a
//! configured path stops existing. When an `[[exceptions]]` or `[ignore]` path
//! is renamed or deleted, the entry silently stops matching: suppressed
//! diagnostics quietly come back, or an ignore no longer covers anything. These
//! checks read harness.toml and flag the stale entries through the normal
//! diagnostic pipeline.

use std::path::{Path, PathBuf};

use crate::config::{CONFIG_FILE, ProjectConfig};
use crate::model::{Diagnostic, Severity};

pub const STALE_EXCEPTION_PATH_RULE: &str = "harness.stale-exception-path";
pub const STALE_IGNORE_PATH_RULE: &str = "harness.stale-ignore-path";

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

fn stale_diagnostic(config: &ProjectConfig, rule_id: &str, message: String) -> Diagnostic {
    let level = config
        .overrides
        .get(rule_id)
        .copied()
        .unwrap_or(Severity::Warn);
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

    diagnostics
}

#[cfg(test)]
mod tests {
    use std::fs;

    use crate::config::{IgnoreSection, RuleExceptionSection};

    use super::*;

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
}
