use anyhow::{Context, Result, bail};
use globset::{Glob, GlobSet, GlobSetBuilder};

use crate::config::SuppressionSection;
use crate::model::Diagnostic;

#[derive(Debug)]
pub struct SuppressionOutcome {
    pub diagnostics: Vec<Diagnostic>,
    pub suppressed_count: usize,
}

#[derive(Debug)]
struct CompiledSuppression {
    rule: String,
    paths: GlobSet,
}

pub fn apply_diagnostic_suppressions(
    diagnostics: Vec<Diagnostic>,
    suppressions: &[SuppressionSection],
) -> Result<SuppressionOutcome> {
    if suppressions.is_empty() {
        return Ok(SuppressionOutcome {
            diagnostics,
            suppressed_count: 0,
        });
    }

    let compiled = compile_suppressions(suppressions)?;
    let mut retained = Vec::with_capacity(diagnostics.len());
    let mut suppressed_count = 0;

    for diagnostic in diagnostics {
        if compiled.iter().any(|suppression| {
            suppression.rule == diagnostic.rule_id && suppression.paths.is_match(&diagnostic.path)
        }) {
            suppressed_count += 1;
        } else {
            retained.push(diagnostic);
        }
    }

    Ok(SuppressionOutcome {
        diagnostics: retained,
        suppressed_count,
    })
}

pub fn validate_suppressions(suppressions: &[SuppressionSection]) -> Result<()> {
    compile_suppressions(suppressions).map(|_| ())
}

fn compile_suppressions(suppressions: &[SuppressionSection]) -> Result<Vec<CompiledSuppression>> {
    let mut compiled = Vec::with_capacity(suppressions.len());
    for suppression in suppressions {
        let rule = suppression.rule.trim();
        if rule.is_empty() {
            bail!("suppression rule must not be empty");
        }
        if suppression.paths.is_empty() {
            bail!("suppression for rule `{rule}` must include at least one path");
        }

        let mut builder = GlobSetBuilder::new();
        for pattern in &suppression.paths {
            let pattern = pattern.trim();
            if pattern.is_empty() {
                bail!("suppression for rule `{rule}` includes an empty path glob");
            }
            builder.add(
                Glob::new(pattern)
                    .with_context(|| format!("invalid suppression glob `{pattern}`"))?,
            );
        }

        compiled.push(CompiledSuppression {
            rule: rule.to_string(),
            paths: builder
                .build()
                .with_context(|| format!("failed to compile suppression globs for `{rule}`"))?,
        });
    }
    Ok(compiled)
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use crate::model::Severity;

    use super::*;

    fn diagnostic(rule_id: &str, path: &str) -> Diagnostic {
        Diagnostic {
            rule_id: rule_id.to_string(),
            level: Severity::Warn,
            message: "message".to_string(),
            path: PathBuf::from(path),
            start_line: 1,
            start_column: 1,
            end_line: None,
            end_column: None,
            fix_available: false,
        }
    }

    #[test]
    fn suppresses_only_matching_rule_and_path() {
        let diagnostics = vec![
            diagnostic("go.no-placeholder", "src/generated/router.go"),
            diagnostic("go.no-placeholder", "src/app/service.go"),
            diagnostic("go.no-panic", "src/generated/router.go"),
        ];
        let suppressions = vec![SuppressionSection {
            rule: "go.no-placeholder".to_string(),
            paths: vec!["src/generated/**".to_string()],
            reason: Some("generated route glue".to_string()),
        }];

        let outcome = apply_diagnostic_suppressions(diagnostics, &suppressions).unwrap();

        assert_eq!(outcome.suppressed_count, 1);
        assert_eq!(
            outcome
                .diagnostics
                .iter()
                .map(|diagnostic| (diagnostic.rule_id.as_str(), diagnostic.path.as_path()))
                .collect::<Vec<_>>(),
            vec![
                ("go.no-placeholder", Path::new("src/app/service.go")),
                ("go.no-panic", Path::new("src/generated/router.go")),
            ]
        );
    }

    #[test]
    fn rejects_suppression_without_paths() {
        let suppressions = vec![SuppressionSection {
            rule: "go.no-placeholder".to_string(),
            paths: Vec::new(),
            reason: None,
        }];

        let error = validate_suppressions(&suppressions)
            .unwrap_err()
            .to_string();

        assert!(error.contains("must include at least one path"));
    }
}
