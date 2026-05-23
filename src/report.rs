use anyhow::Result;

use crate::model::{Diagnostic, RuleDefinition, Severity};

#[derive(Debug, Clone, Copy)]
pub enum ReportFormat {
    Human,
    Json,
}

pub fn report_diagnostics(diagnostics: &[Diagnostic], format: ReportFormat) -> Result<()> {
    match format {
        ReportFormat::Human => report_human(diagnostics),
        ReportFormat::Json => {
            println!("{}", serde_json::to_string_pretty(diagnostics)?);
        }
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

pub fn print_rules(rules: &[RuleDefinition], format: ReportFormat) -> Result<()> {
    match format {
        ReportFormat::Json => {
            println!("{}", serde_json::to_string_pretty(&rules_to_json(rules))?)
        }
        ReportFormat::Human => {
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

fn severity_label(severity: Severity) -> &'static str {
    match severity {
        Severity::None => "none",
        Severity::Info => "info",
        Severity::Warn => "warn",
        Severity::Error => "error",
    }
}
