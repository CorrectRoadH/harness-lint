use std::path::Path;

use anyhow::{Result, bail};
use regex::Regex;

use crate::model::{RuleBody, RuleDefinition, RuleEngineKind, RuleExampleKind, RuleStatus};

pub fn test_rule(_root: &Path, rule: &RuleDefinition) -> Result<()> {
    if matches!(rule.body, RuleBody::Missing) {
        bail!("rule `{}` has no executable body", rule.id);
    }
    if rule.status == RuleStatus::Enforced {
        let has_bad = rule
            .examples
            .iter()
            .any(|example| example.kind == RuleExampleKind::Bad && !example.code.trim().is_empty());
        let has_good = rule.examples.iter().any(|example| {
            example.kind == RuleExampleKind::Good && !example.code.trim().is_empty()
        });
        if !has_bad || !has_good {
            bail!(
                "enforced rule `{}` must include Bad and Good examples",
                rule.id
            );
        }
    }
    match rule.engine {
        RuleEngineKind::Text => test_text_examples(rule)?,
        RuleEngineKind::Regex => test_regex_examples(rule)?,
        _ => {}
    }
    Ok(())
}

fn test_text_examples(rule: &RuleDefinition) -> Result<()> {
    let RuleBody::Text(needle) = &rule.body else {
        return Ok(());
    };
    let needle = needle.trim();
    if needle.is_empty() {
        bail!("text rule `{}` has an empty pattern", rule.id);
    }
    for example in &rule.examples {
        let matched = example.code.contains(needle);
        match example.kind {
            RuleExampleKind::Bad if !matched => {
                bail!("text rule `{}` did not match a Bad example", rule.id)
            }
            RuleExampleKind::Good if matched => {
                bail!("text rule `{}` matched a Good example", rule.id)
            }
            _ => {}
        }
    }
    Ok(())
}

fn test_regex_examples(rule: &RuleDefinition) -> Result<()> {
    let RuleBody::Regex(pattern) = &rule.body else {
        return Ok(());
    };
    let regex = Regex::new(pattern).map_err(|error| {
        anyhow::anyhow!("regex rule `{}` has an invalid pattern: {error}", rule.id)
    })?;
    for example in &rule.examples {
        let matched = regex.is_match(&example.code);
        match example.kind {
            RuleExampleKind::Bad if !matched => {
                bail!("regex rule `{}` did not match a Bad example", rule.id)
            }
            RuleExampleKind::Good if matched => {
                bail!("regex rule `{}` matched a Good example", rule.id)
            }
            _ => {}
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::model::{RuleExample, Severity};

    use super::*;

    #[test]
    fn text_rule_tests_bad_and_good_examples() {
        let rule = RuleDefinition {
            id: "text.no-todo".to_string(),
            title: "No TODO".to_string(),
            engine: RuleEngineKind::Text,
            language: Some("markdown".to_string()),
            level: Severity::Warn,
            status: RuleStatus::Enforced,
            tags: vec![],
            fixable: false,
            description: String::new(),
            body: RuleBody::Text("TODO".to_string()),
            examples: vec![
                RuleExample {
                    kind: RuleExampleKind::Bad,
                    language: Some("text".to_string()),
                    code: "TODO fix this".to_string(),
                },
                RuleExample {
                    kind: RuleExampleKind::Good,
                    language: Some("text".to_string()),
                    code: "Done".to_string(),
                },
            ],
            source_path: PathBuf::from("rule.md"),
            pack_id: None,
        };
        test_rule(std::path::Path::new("."), &rule).unwrap();
    }
}
