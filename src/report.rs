use anyhow::Result;

use crate::model::{Diagnostic, RuleDefinition, Severity};

#[derive(Debug, Clone, Copy)]
pub enum ReportFormat {
    Human,
    Json,
    Jsonl,
    Markdown,
    Github,
    Sarif,
}

pub fn report_diagnostics(diagnostics: &[Diagnostic], format: ReportFormat) -> Result<()> {
    match format {
        ReportFormat::Human => report_human(diagnostics),
        ReportFormat::Json => {
            println!("{}", serde_json::to_string_pretty(diagnostics)?);
        }
        ReportFormat::Jsonl => {
            for diagnostic in diagnostics {
                println!("{}", serde_json::to_string(diagnostic)?);
            }
        }
        ReportFormat::Markdown => report_markdown(diagnostics),
        ReportFormat::Github => report_github(diagnostics),
        ReportFormat::Sarif => report_sarif(diagnostics)?,
    }
    Ok(())
}

fn report_human(diagnostics: &[Diagnostic]) {
    if diagnostics.is_empty() {
        println!("No diagnostics.");
        return;
    }

    for diagnostic in diagnostics {
        println!(
            "{}:{}:{} [{}] {}: {}",
            diagnostic.path.display(),
            diagnostic.start_line,
            diagnostic.start_column,
            severity_label(diagnostic.level),
            diagnostic.rule_id,
            diagnostic.message
        );
    }
}

fn report_markdown(diagnostics: &[Diagnostic]) {
    println!("# harness-lint report\n");
    if diagnostics.is_empty() {
        println!("No diagnostics.");
        return;
    }
    for diagnostic in diagnostics {
        println!(
            "- `{}` at `{}:{}:{}`: {}",
            diagnostic.rule_id,
            diagnostic.path.display(),
            diagnostic.start_line,
            diagnostic.start_column,
            diagnostic.message
        );
    }
}

pub fn print_rules(rules: &[RuleDefinition], format: ReportFormat) -> Result<()> {
    match format {
        ReportFormat::Json | ReportFormat::Sarif => {
            println!("{}", serde_json::to_string_pretty(&rules_to_json(rules))?)
        }
        ReportFormat::Jsonl => {
            for rule in rules_to_json(rules) {
                println!("{}", serde_json::to_string(&rule)?);
            }
        }
        ReportFormat::Human | ReportFormat::Markdown | ReportFormat::Github => {
            if rules.is_empty() {
                println!("No rules found.");
            }
            for rule in rules {
                println!(
                    "{}\t{:?}\t{:?}\t{}",
                    rule.id,
                    rule.status,
                    rule.level,
                    rule.source_path.display()
                );
            }
        }
    }
    Ok(())
}

pub fn print_rule_explain(rule: &RuleDefinition) {
    println!("# {}\n", rule.title);
    println!("id: `{}`", rule.id);
    println!("status: `{:?}`", rule.status);
    println!("level: `{:?}`", rule.level);
    println!("source: `{}`\n", rule.source_path.display());
    if !rule.description.is_empty() {
        println!("{}", rule.description);
    }
}

fn rules_to_json(rules: &[RuleDefinition]) -> Vec<serde_json::Value> {
    rules
        .iter()
        .map(|rule| {
            serde_json::json!({
                "id": rule.id,
                "title": rule.title,
                "language": rule.language,
                "level": format!("{:?}", rule.level),
                "status": format!("{:?}", rule.status),
                "source_path": rule.source_path,
                "pack_id": rule.pack_id,
            })
        })
        .collect()
}

fn report_github(diagnostics: &[Diagnostic]) {
    for diagnostic in diagnostics {
        let level = match diagnostic.level {
            Severity::Error => "error",
            _ => "warning",
        };
        println!(
            "::{level} file={},line={},col={},title={}::{}",
            escape_github(&diagnostic.path.display().to_string()),
            diagnostic.start_line,
            diagnostic.start_column,
            escape_github(&diagnostic.rule_id),
            escape_github(&diagnostic.message)
        );
    }
}

fn report_sarif(diagnostics: &[Diagnostic]) -> Result<()> {
    let results = diagnostics
        .iter()
        .map(|diagnostic| {
            serde_json::json!({
                "ruleId": diagnostic.rule_id,
                "level": sarif_level(diagnostic.level),
                "message": { "text": diagnostic.message },
                "locations": [{
                    "physicalLocation": {
                        "artifactLocation": { "uri": diagnostic.path },
                        "region": {
                            "startLine": diagnostic.start_line,
                            "startColumn": diagnostic.start_column,
                            "endLine": diagnostic.end_line,
                            "endColumn": diagnostic.end_column,
                        }
                    }
                }]
            })
        })
        .collect::<Vec<_>>();
    let sarif = serde_json::json!({
        "$schema": "https://json.schemastore.org/sarif-2.1.0.json",
        "version": "2.1.0",
        "runs": [{
            "tool": {
                "driver": {
                    "name": "harness-lint",
                    "informationUri": "https://github.com/harness-lint/harness-lint"
                }
            },
            "results": results
        }]
    });
    println!("{}", serde_json::to_string_pretty(&sarif)?);
    Ok(())
}

fn sarif_level(severity: Severity) -> &'static str {
    match severity {
        Severity::Error => "error",
        Severity::Warn => "warning",
        Severity::Info => "note",
        Severity::None => "none",
    }
}

fn escape_github(value: &str) -> String {
    value
        .replace('%', "%25")
        .replace('\r', "%0D")
        .replace('\n', "%0A")
        .replace(':', "%3A")
        .replace(',', "%2C")
}

fn severity_label(severity: Severity) -> &'static str {
    match severity {
        Severity::None => "none",
        Severity::Info => "info",
        Severity::Warn => "warn",
        Severity::Error => "error",
    }
}
