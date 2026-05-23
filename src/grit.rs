use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result, bail};
use serde::Deserialize;

use crate::model::{CompiledRules, Diagnostic, Severity};

pub fn ensure_grit_available() -> Result<String> {
    let output = Command::new("grit")
        .arg("--version")
        .output()
        .context("failed to run `grit --version`; install Grit CLI before running checks")?;
    if !output.status.success() {
        bail!("`grit --version` failed; install or repair the Grit CLI");
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

pub fn check_grit_compatibility(version_output: &str, _requirement: Option<&str>) -> Result<()> {
    if version_output.trim().is_empty() {
        bail!("`grit --version` returned an empty version string");
    }
    Ok(())
}

pub fn run_grit(
    root: &Path,
    compiled: &CompiledRules,
    paths: &[PathBuf],
) -> Result<Vec<Diagnostic>> {
    if compiled.grit_rules.is_empty() {
        return Ok(Vec::new());
    }

    let version = ensure_grit_available()?;
    check_grit_compatibility(&version, None)?;
    let grit_cache_dir = root.join(".harness/cache/grit");
    std::fs::create_dir_all(&grit_cache_dir)
        .with_context(|| format!("failed to create {}", grit_cache_dir.display()))?;

    let mut command = Command::new("grit");
    let grit_work_dir = compiled.grit_dir.parent().unwrap_or(root);
    command
        .current_dir(grit_work_dir)
        .env("GRIT_CACHE_DIR", &grit_cache_dir)
        .arg("--json")
        .arg("check");

    if paths.is_empty() {
        command.arg(root);
    } else {
        command.args(paths.iter().map(|path| root.join(path)));
    }

    let output = command.output().context("failed to run `grit check`")?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let diagnostics = parse_grit_output(&stdout, &stderr);

    if !output.status.success() && diagnostics.is_empty() {
        bail!("`grit check` failed: {}", stderr.trim());
    }

    Ok(diagnostics)
}

fn parse_grit_output(stdout: &str, stderr: &str) -> Vec<Diagnostic> {
    let json = if stdout.trim_start().starts_with('{') {
        stdout
    } else if stderr.trim_start().starts_with('{') {
        stderr
    } else {
        return parse_grit_jsonl(stdout);
    };
    match serde_json::from_str::<GritJsonOutput>(json) {
        Ok(output) => output
            .results
            .into_iter()
            .map(GritJsonResult::into_diagnostic)
            .collect(),
        Err(_) => parse_grit_jsonl(json),
    }
}

#[derive(Debug, Deserialize)]
struct GritJsonOutput {
    results: Vec<GritJsonResult>,
}

#[derive(Debug, Deserialize)]
struct GritJsonResult {
    #[serde(default)]
    check_id: Option<String>,
    #[serde(default)]
    local_name: Option<String>,
    start: GritJsonPosition,
    #[serde(default)]
    end: Option<GritJsonPosition>,
    path: PathBuf,
    #[serde(default)]
    extra: Option<GritJsonExtra>,
}

#[derive(Debug, Deserialize)]
struct GritJsonPosition {
    line: u32,
    #[serde(alias = "column")]
    col: u32,
}

#[derive(Debug, Deserialize)]
struct GritJsonExtra {
    #[serde(default)]
    message: Option<String>,
    #[serde(default)]
    severity: Option<String>,
}

impl GritJsonResult {
    fn into_diagnostic(self) -> Diagnostic {
        let extra = self.extra.unwrap_or_default();
        Diagnostic {
            rule_id: self
                .local_name
                .or(self.check_id)
                .unwrap_or_else(|| "grit.unknown".to_string()),
            level: parse_level(extra.severity.as_deref()),
            message: extra
                .message
                .unwrap_or_else(|| "Grit diagnostic".to_string()),
            path: self.path,
            start_line: self.start.line,
            start_column: self.start.col,
            end_line: self.end.as_ref().map(|position| position.line),
            end_column: self.end.as_ref().map(|position| position.col),
            fix_available: false,
        }
    }
}

impl Default for GritJsonExtra {
    fn default() -> Self {
        Self {
            message: None,
            severity: None,
        }
    }
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
        let diagnostics = parse_grit_output(output, "");
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].rule_id, "python.no-print");
        assert_eq!(diagnostics[0].level, Severity::Error);
    }

    #[test]
    fn parses_grit_json_diagnostic() {
        let output = r##"{"paths":["a.md"],"results":[{"check_id":"#fence/markdown","local_name":"fence","start":{"line":1,"col":1,"offset":0},"end":{"line":1,"col":5,"offset":4},"path":"a.md","extra":{"message":"Bad fence","severity":"warn"}}]}"##;
        let diagnostics = parse_grit_output("", output);
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].rule_id, "fence");
        assert_eq!(diagnostics[0].message, "Bad fence");
        assert_eq!(diagnostics[0].path, PathBuf::from("a.md"));
    }

    #[test]
    fn accepts_non_empty_grit_version() {
        check_grit_compatibility("grit 0.1.0", Some(">=0.1.0")).unwrap();
        assert!(check_grit_compatibility("", None).is_err());
    }
}
