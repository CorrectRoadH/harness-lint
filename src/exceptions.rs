use anyhow::{Context, Result, bail};
use globset::{Glob, GlobSet, GlobSetBuilder};

use crate::config::RuleExceptionSection;
use crate::model::Diagnostic;

#[derive(Debug)]
pub struct ExceptionOutcome {
    pub diagnostics: Vec<Diagnostic>,
    pub hidden_count: usize,
}

#[derive(Debug)]
struct CompiledRuleException {
    rule: String,
    paths: GlobSet,
}

pub fn apply_diagnostic_exceptions(
    diagnostics: Vec<Diagnostic>,
    exceptions: &[RuleExceptionSection],
) -> Result<ExceptionOutcome> {
    if exceptions.is_empty() {
        return Ok(ExceptionOutcome {
            diagnostics,
            hidden_count: 0,
        });
    }

    let compiled = compile_exceptions(exceptions)?;
    let mut retained = Vec::with_capacity(diagnostics.len());
    let mut hidden_count = 0;

    for diagnostic in diagnostics {
        if compiled.iter().any(|exception| {
            exception.rule == diagnostic.rule_id && exception.paths.is_match(&diagnostic.path)
        }) {
            hidden_count += 1;
        } else {
            retained.push(diagnostic);
        }
    }

    Ok(ExceptionOutcome {
        diagnostics: retained,
        hidden_count,
    })
}

pub fn validate_exceptions(exceptions: &[RuleExceptionSection]) -> Result<()> {
    compile_exceptions(exceptions).map(|_| ())
}

fn compile_exceptions(exceptions: &[RuleExceptionSection]) -> Result<Vec<CompiledRuleException>> {
    let mut compiled = Vec::with_capacity(exceptions.len());
    for exception in exceptions {
        let rule = exception.rule.trim();
        if rule.is_empty() {
            bail!("rule exception must include a rule");
        }
        if exception.paths.is_empty() {
            bail!("rule exception for `{rule}` must include at least one path");
        }

        let mut builder = GlobSetBuilder::new();
        for pattern in &exception.paths {
            let pattern = pattern.trim();
            if pattern.is_empty() {
                bail!("rule exception for `{rule}` includes an empty path glob");
            }
            builder.add(
                Glob::new(pattern)
                    .with_context(|| format!("invalid path glob `{pattern}` in rule exception"))?,
            );
        }

        compiled.push(CompiledRuleException {
            rule: rule.to_string(),
            paths: builder.build().with_context(|| {
                format!("failed to compile path globs for rule exception `{rule}`")
            })?,
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
    fn hides_only_matching_rule_and_path() {
        let diagnostics = vec![
            diagnostic("go.no-placeholder", "src/generated/router.go"),
            diagnostic("go.no-placeholder", "src/app/service.go"),
            diagnostic("go.no-panic", "src/generated/router.go"),
        ];
        let exceptions = vec![RuleExceptionSection {
            rule: "go.no-placeholder".to_string(),
            paths: vec!["src/generated/**".to_string()],
            reason: Some("generated route glue".to_string()),
        }];

        let outcome = apply_diagnostic_exceptions(diagnostics, &exceptions).unwrap();

        assert_eq!(outcome.hidden_count, 1);
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
    fn rejects_exception_without_paths() {
        let exceptions = vec![RuleExceptionSection {
            rule: "go.no-placeholder".to_string(),
            paths: Vec::new(),
            reason: None,
        }];

        let error = validate_exceptions(&exceptions).unwrap_err().to_string();

        assert!(error.contains("must include at least one path"));
    }
}
