use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result, bail};
use serde::Deserialize;

use crate::model::{CompiledRules, Diagnostic, Severity};

#[derive(Debug, Clone, Copy)]
pub enum CheckMode {
    Check,
    Fix,
}

pub fn ensure_grit_available() -> Result<String> {
    let output = Command::new("grit")
        .arg("version")
        .output()
        .context("failed to run `grit version`; install Grit CLI before running checks")?;
    if !output.status.success() {
        bail!("`grit version` failed; install or repair the Grit CLI");
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

pub fn check_grit_compatibility(version_output: &str, _requirement: Option<&str>) -> Result<()> {
    if version_output.trim().is_empty() {
        bail!("`grit version` returned an empty version string");
    }
    Ok(())
}

pub fn run_grit(
    root: &Path,
    compiled: &CompiledRules,
    paths: &[PathBuf],
    mode: CheckMode,
) -> Result<Vec<Diagnostic>> {
    if compiled.grit_rules.is_empty() {
        return Ok(Vec::new());
    }

    let version = ensure_grit_available()?;
    check_grit_compatibility(&version, None)?;

    let mut command = Command::new("grit");
    command
        .current_dir(root)
        .arg("--grit-dir")
        .arg(&compiled.grit_dir)
        .arg("--jsonl")
        .arg("check");

    if matches!(mode, CheckMode::Fix) {
        command.arg("--fix");
    }

    if paths.is_empty() {
        command.arg(".");
    } else {
        command.args(paths);
    }

    let output = command.output().context("failed to run `grit check`")?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let diagnostics = parse_grit_jsonl(&stdout);

    if !output.status.success() && diagnostics.is_empty() {
        bail!("`grit check` failed: {}", stderr.trim());
    }

    Ok(diagnostics)
}

fn parse_grit_jsonl(output: &str) -> Vec<Diagnostic> {
    output
        .lines()
        .filter_map(|line| serde_json::from_str::<GritJsonLine>(line).ok())
        .filter_map(GritJsonLine::into_diagnostic)
        .collect()
}

#[derive(Debug, Deserialize)]
struct GritJsonLine {
    #[serde(default)]
    file: Option<PathBuf>,
    #[serde(default)]
    path: Option<PathBuf>,
    #[serde(default)]
    message: Option<String>,
    #[serde(default)]
    rule: Option<String>,
    #[serde(default)]
    rule_id: Option<String>,
    #[serde(default)]
    level: Option<String>,
    #[serde(default)]
    line: Option<u32>,
    #[serde(default)]
    column: Option<u32>,
    #[serde(default)]
    end_line: Option<u32>,
    #[serde(default)]
    end_column: Option<u32>,
}

impl GritJsonLine {
    fn into_diagnostic(self) -> Option<Diagnostic> {
        let path = self.path.or(self.file)?;
        Some(Diagnostic {
            rule_id: self
                .rule_id
                .or(self.rule)
                .unwrap_or_else(|| "grit.unknown".to_string()),
            level: parse_level(self.level.as_deref()),
            message: self
                .message
                .unwrap_or_else(|| "Grit diagnostic".to_string()),
            path,
            start_line: self.line.unwrap_or(1),
            start_column: self.column.unwrap_or(1),
            end_line: self.end_line,
            end_column: self.end_column,
            fix_available: false,
        })
    }
}

fn parse_level(level: Option<&str>) -> Severity {
    match level.unwrap_or("warn").to_ascii_lowercase().as_str() {
        "none" => Severity::None,
        "info" => Severity::Info,
        "error" => Severity::Error,
        _ => Severity::Warn,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_simple_jsonl_diagnostic() {
        let output = r#"{"path":"src/main.py","message":"No print","rule_id":"python.no-print","level":"error","line":2,"column":4}"#;
        let diagnostics = parse_grit_jsonl(output);
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].rule_id, "python.no-print");
        assert_eq!(diagnostics[0].level, Severity::Error);
    }

    #[test]
    fn accepts_non_empty_grit_version() {
        check_grit_compatibility("grit 0.1.0", Some(">=0.1.0")).unwrap();
        assert!(check_grit_compatibility("", None).is_err());
    }
}
