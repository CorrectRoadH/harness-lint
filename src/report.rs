use std::io::IsTerminal;
use std::path::{Path, PathBuf};

use anyhow::Result;

use crate::model::{Diagnostic, RuleDefinition, RulePack, Severity};

#[derive(Debug, Clone, Copy)]
pub enum ReportFormat {
    Human,
    Json,
}

#[derive(Debug, Clone, Copy)]
pub struct DiagnosticReportOptions<'a> {
    pub root: &'a Path,
    pub active_rules: usize,
    pub checked_files: usize,
}

pub fn report_diagnostics(
    diagnostics: &[Diagnostic],
    format: ReportFormat,
    options: DiagnosticReportOptions<'_>,
) -> Result<()> {
    match format {
        ReportFormat::Human => report_human(diagnostics, options),
        ReportFormat::Json => {
            println!("{}", serde_json::to_string_pretty(diagnostics)?);
        }
    }
    Ok(())
}

fn report_human(diagnostics: &[Diagnostic], options: DiagnosticReportOptions<'_>) {
    if diagnostics.is_empty() {
        if options.active_rules == 0 {
            println!("No diagnostics because no active rules were selected.");
        } else if options.checked_files == 0 {
            println!(
                "No diagnostics because no files matched the selected scope and active rules."
            );
        } else {
            println!(
                "No diagnostics across {} file(s) with {} active rule(s).",
                options.checked_files, options.active_rules
            );
        }
        return;
    }

    let colors = ColorMode::detect();
    let mut current_path: Option<&PathBuf> = None;
    // Diagnostics arrive sorted by path; cache the current file's lines so a
    // file with many diagnostics is read once, not once per diagnostic.
    let mut current_lines: Option<Vec<String>> = None;
    for diagnostic in diagnostics {
        if current_path != Some(&diagnostic.path) {
            if current_path.is_some() {
                println!();
            }
            println!(
                "{}",
                colors.paint(Color::Path, &diagnostic.path.display().to_string())
            );
            current_path = Some(&diagnostic.path);
            current_lines = read_source_lines(options.root, &diagnostic.path);
        }

        println!(
            "  {}  {}:{}  {}",
            colors.paint(
                severity_color(diagnostic.level),
                &severity_label(diagnostic.level).to_ascii_uppercase()
            ),
            diagnostic.start_line,
            diagnostic.start_column,
            colors.paint(Color::Rule, &diagnostic.rule_id),
        );
        println!("        {}", diagnostic.message);

        if let Some(source) = current_lines
            .as_ref()
            .and_then(|lines| lines.get(diagnostic.start_line.saturating_sub(1) as usize))
        {
            print_snippet(diagnostic, source, colors);
        }
    }
}

fn read_source_lines(root: &Path, path: &Path) -> Option<Vec<String>> {
    let path = if path.is_absolute() {
        path.to_path_buf()
    } else {
        root.join(path)
    };
    let content = std::fs::read_to_string(path).ok()?;
    Some(content.lines().map(ToOwned::to_owned).collect())
}

fn print_snippet(diagnostic: &Diagnostic, source: &str, colors: ColorMode) {
    let gutter_width = diagnostic.start_line.to_string().len().max(2);
    println!();
    println!(
        "  {:>gutter_width$} | {}",
        diagnostic.start_line,
        source,
        gutter_width = gutter_width
    );
    println!(
        "  {:>gutter_width$} | {}{}",
        "",
        " ".repeat(diagnostic.start_column.saturating_sub(1) as usize),
        colors.paint(Color::Caret, &"^".repeat(caret_width(diagnostic))),
        gutter_width = gutter_width
    );
}

fn caret_width(diagnostic: &Diagnostic) -> usize {
    if diagnostic.end_line == Some(diagnostic.start_line)
        && let Some(end_column) = diagnostic.end_column
    {
        return end_column.saturating_sub(diagnostic.start_column).max(1) as usize;
    }
    1
}

pub fn print_rule_packs(packs: &[RulePack], format: ReportFormat) -> Result<()> {
    match format {
        ReportFormat::Json => anyhow::bail!(
            "`harness-lint --json rule list` is not supported; `rule list` always prints Markdown"
        ),
        ReportFormat::Human => {
            let non_empty_packs = packs
                .iter()
                .filter(|pack| !pack.rules.is_empty())
                .collect::<Vec<_>>();
            if non_empty_packs.is_empty() {
                println!("No rules found.");
                return Ok(());
            }
            for (index, pack) in non_empty_packs.iter().enumerate() {
                if index > 0 {
                    println!();
                }
                println!("## {}", markdown_text(&pack.name));
                println!();
                println!("| Level | ID | Description |");
                println!("| --- | --- | --- |");
                for rule in &pack.rules {
                    let description = if rule.description.is_empty() {
                        &rule.title
                    } else {
                        &rule.description
                    };
                    println!(
                        "| {} | `{}` | {} |",
                        rule.level,
                        markdown_text(&rule.id),
                        markdown_text(description)
                    );
                }
            }
        }
    }
    Ok(())
}

pub fn print_rule_explain(rule: &RuleDefinition) {
    println!("# {}\n", rule.title);
    println!("id: `{}`", rule.id);
    println!("level: `{:?}`", rule.level);
    println!("source: `{}`\n", rule.source_path.display());
    if !rule.description.is_empty() {
        println!("{}", rule.description);
    }
}

fn markdown_text(value: &str) -> String {
    value
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
        .replace('\\', "\\\\")
        .replace('|', "\\|")
}

fn severity_label(severity: Severity) -> &'static str {
    match severity {
        Severity::None => "none",
        Severity::Info => "info",
        Severity::Warn => "warn",
        Severity::Error => "error",
    }
}

fn severity_color(severity: Severity) -> Color {
    match severity {
        Severity::None => Color::Muted,
        Severity::Info => Color::Info,
        Severity::Warn => Color::Warn,
        Severity::Error => Color::Error,
    }
}

#[derive(Debug, Clone, Copy)]
enum Color {
    Caret,
    Error,
    Info,
    Muted,
    Path,
    Rule,
    Warn,
}

#[derive(Debug, Clone, Copy)]
struct ColorMode {
    enabled: bool,
}

impl ColorMode {
    fn detect() -> Self {
        Self {
            enabled: std::io::stdout().is_terminal() && std::env::var_os("NO_COLOR").is_none(),
        }
    }

    fn paint(self, color: Color, text: &str) -> String {
        if !self.enabled {
            return text.to_string();
        }
        let code = match color {
            Color::Caret => "32",
            Color::Error => "31;1",
            Color::Info => "34;1",
            Color::Muted | Color::Path => "2",
            Color::Rule => "36",
            Color::Warn => "33;1",
        };
        format!("\x1b[{code}m{text}\x1b[0m")
    }
}
