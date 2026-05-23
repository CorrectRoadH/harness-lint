use std::path::Path;

use anyhow::{Result, bail};

use crate::model::{RuleBody, RuleDefinition, RuleExampleKind, RuleStatus};

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
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::model::{RuleExample, Severity};

    use super::*;

    #[test]
    fn enforced_rule_requires_bad_and_good_examples() {
        let rule = RuleDefinition {
            id: "local.no-todo".to_string(),
            title: "No TODO".to_string(),
            language: Some("markdown".to_string()),
            level: Severity::Warn,
            status: RuleStatus::Enforced,
            tags: vec![],
            fixable: false,
            description: String::new(),
            body: RuleBody::Grit("language markdown\n`TODO`".to_string()),
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
