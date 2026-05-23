use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result, bail};
use regex::Regex;
use serde::Deserialize;

use crate::model::{CompiledRules, Diagnostic, RuleBody, RuleDefinition, RuleEngineKind, Severity};
use crate::paths;

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

pub fn run_builtin(
    root: &Path,
    rules: &[RuleDefinition],
    paths: &[PathBuf],
) -> Result<Vec<Diagnostic>> {
    let mut diagnostics = Vec::new();
    let builtin_rules = rules
        .iter()
        .filter(|rule| matches!(rule.engine, RuleEngineKind::Text | RuleEngineKind::Regex))
        .collect::<Vec<_>>();
    if builtin_rules.is_empty() {
        return run_external(root, rules, paths);
    }

    for path in paths {
        let full_path = root.join(path);
        let content = match std::fs::read_to_string(&full_path) {
            Ok(content) => content,
            Err(_) => continue,
        };
        for rule in &builtin_rules {
            if !paths::rule_matches_path(rule, path) {
                continue;
            }
            match &rule.body {
                RuleBody::Text(needle) => {
                    if let Some((line, column)) = find_text(&content, needle) {
                        diagnostics.push(Diagnostic {
                            rule_id: rule.id.clone(),
                            level: rule.level,
                            message: rule.title.clone(),
                            path: path.clone(),
                            start_line: line,
                            start_column: column,
                            end_line: None,
                            end_column: None,
                            fix_available: false,
                        });
                    }
                }
                RuleBody::Regex(pattern) => {
                    let regex = Regex::new(pattern)
                        .with_context(|| format!("invalid regex in rule `{}`", rule.id))?;
                    if let Some(match_) = regex.find(&content) {
                        let (line, column) = offset_to_line_column(&content, match_.start());
                        diagnostics.push(Diagnostic {
                            rule_id: rule.id.clone(),
                            level: rule.level,
                            message: rule.title.clone(),
                            path: path.clone(),
                            start_line: line,
                            start_column: column,
                            end_line: None,
                            end_column: None,
                            fix_available: false,
                        });
                    }
                }
                _ => {}
            }
        }
    }

    diagnostics.extend(run_external(root, rules, paths)?);
    Ok(diagnostics)
}

fn run_external(
    root: &Path,
    rules: &[RuleDefinition],
    paths: &[PathBuf],
) -> Result<Vec<Diagnostic>> {
    let mut diagnostics = Vec::new();
    for rule in rules
        .iter()
        .filter(|rule| rule.engine == RuleEngineKind::External)
    {
        let command = match &rule.body {
            RuleBody::Text(command) if !command.trim().is_empty() => command.trim(),
            _ => continue,
        };
        let selected_paths = paths
            .iter()
            .filter(|path| paths::rule_matches_path(rule, path))
            .map(|path| path.to_string_lossy().to_string())
            .collect::<Vec<_>>();
        if selected_paths.is_empty() {
            continue;
        }
        let output = Command::new("sh")
            .current_dir(root)
            .arg("-c")
            .arg(command)
            .env("HARNESS_RULE_ID", &rule.id)
            .env("HARNESS_PATHS", selected_paths.join("\n"))
            .output()
            .with_context(|| format!("failed to run external rule `{}`", rule.id))?;
        let stdout = String::from_utf8_lossy(&output.stdout);
        diagnostics.extend(parse_external_jsonl(&stdout, rule));
        if !output.status.success() && diagnostics.is_empty() {
            bail!(
                "external rule `{}` failed: {}",
                rule.id,
                String::from_utf8_lossy(&output.stderr).trim()
            );
        }
    }
    Ok(diagnostics)
}

fn parse_external_jsonl(output: &str, rule: &RuleDefinition) -> Vec<Diagnostic> {
    output
        .lines()
        .filter_map(|line| serde_json::from_str::<ExternalJsonLine>(line).ok())
        .map(|line| Diagnostic {
            rule_id: line.rule_id.unwrap_or_else(|| rule.id.clone()),
            level: line
                .level
                .as_deref()
                .map(parse_level_value)
                .unwrap_or(rule.level),
            message: line.message.unwrap_or_else(|| rule.title.clone()),
            path: line.path,
            start_line: line.line.unwrap_or(1),
            start_column: line.column.unwrap_or(1),
            end_line: line.end_line,
            end_column: line.end_column,
            fix_available: line.fix_available.unwrap_or(false),
        })
        .collect()
}

#[derive(Debug, Deserialize)]
struct ExternalJsonLine {
    path: PathBuf,
    #[serde(default)]
    rule_id: Option<String>,
    #[serde(default)]
    level: Option<String>,
    #[serde(default)]
    message: Option<String>,
    #[serde(default)]
    line: Option<u32>,
    #[serde(default)]
    column: Option<u32>,
    #[serde(default)]
    end_line: Option<u32>,
    #[serde(default)]
    end_column: Option<u32>,
    #[serde(default)]
    fix_available: Option<bool>,
}

fn find_text(content: &str, needle: &str) -> Option<(u32, u32)> {
    let needle = needle.trim();
    if needle.is_empty() {
        return None;
    }
    content
        .find(needle)
        .map(|offset| offset_to_line_column(content, offset))
}

fn offset_to_line_column(content: &str, offset: usize) -> (u32, u32) {
    let mut line = 1;
    let mut column = 1;
    for (index, ch) in content.char_indices() {
        if index >= offset {
            break;
        }
        if ch == '\n' {
            line += 1;
            column = 1;
        } else {
            column += 1;
        }
    }
    (line, column)
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

fn parse_level_value(level: &str) -> Severity {
    match level.to_ascii_lowercase().as_str() {
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
    fn builtin_text_rule_reports_match() {
        let tempdir = tempfile::tempdir().unwrap();
        std::fs::write(tempdir.path().join("note.md"), "hello TODO world").unwrap();
        let rule = RuleDefinition {
            id: "text.no-todo".to_string(),
            title: "No TODO".to_string(),
            engine: RuleEngineKind::Text,
            language: Some("markdown".to_string()),
            level: Severity::Warn,
            status: crate::model::RuleStatus::Warn,
            tags: vec![],
            fixable: false,
            description: String::new(),
            body: RuleBody::Text("TODO".to_string()),
            examples: vec![],
            source_path: PathBuf::from("rule.md"),
            pack_id: None,
        };
        let diagnostics =
            run_builtin(tempdir.path(), &[rule], &[PathBuf::from("note.md")]).unwrap();
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].start_column, 7);
    }

    #[test]
    fn accepts_non_empty_grit_version() {
        check_grit_compatibility("grit 0.1.0", Some(">=0.1.0")).unwrap();
        assert!(check_grit_compatibility("", None).is_err());
    }
}
