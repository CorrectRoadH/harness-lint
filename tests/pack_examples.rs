use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use harness_lint::compiler;
use harness_lint::grit;
use harness_lint::model::{
    PackSourceKind, PackSpec, ResolvedPack, RuleDefinition, RuleExampleKind,
};
use harness_lint::pack::{PACK_MANIFEST, load_rule_pack};

fn grit_available() -> bool {
    Command::new("grit")
        .arg("--version")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

#[test]
fn pack_bad_examples_trigger_and_good_examples_pass() {
    if !grit_available() {
        return;
    }

    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    for pack_dir in pack_dirs(repo_root) {
        let pack = load_pack(&pack_dir);
        for rule in pack.rules {
            verify_rule_examples(&rule);
        }
    }
}

fn pack_dirs(repo_root: &Path) -> Vec<PathBuf> {
    let mut dirs = fs::read_dir(repo_root.join("packs"))
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.join(PACK_MANIFEST).exists())
        .collect::<Vec<_>>();
    dirs.sort();
    dirs
}

fn load_pack(pack_dir: &Path) -> harness_lint::model::RulePack {
    let id = pack_dir.file_name().unwrap().to_string_lossy().to_string();
    let resolved = ResolvedPack {
        spec: PackSpec {
            id,
            source: PackSourceKind::Local,
            spec: pack_dir.display().to_string(),
            version_req: None,
        },
        local_path: pack_dir.to_path_buf(),
        pack_path: None,
        version: None,
        checksum: None,
    };
    load_rule_pack(&resolved).unwrap()
}

fn verify_rule_examples(rule: &RuleDefinition) {
    let bad_examples = rule
        .examples
        .iter()
        .filter(|example| example.kind == RuleExampleKind::Bad)
        .collect::<Vec<_>>();
    let good_examples = rule
        .examples
        .iter()
        .filter(|example| example.kind == RuleExampleKind::Good)
        .collect::<Vec<_>>();

    assert!(!bad_examples.is_empty(), "{} has no Bad examples", rule.id);
    assert!(
        !good_examples.is_empty(),
        "{} has no Good examples",
        rule.id
    );

    let tempdir = tempfile::tempdir().unwrap();
    let compiled = compiler::compile_rule_set(tempdir.path(), vec![rule.clone()]).unwrap();

    for (index, example) in bad_examples.iter().enumerate() {
        let relative_path = write_example(tempdir.path(), rule, "bad", index, example);
        let diagnostics = grit::run_grit(tempdir.path(), &compiled, &[relative_path]).unwrap();
        assert!(
            !diagnostics.is_empty(),
            "{} Bad example {} did not trigger",
            rule.id,
            index + 1
        );
    }

    for (index, example) in good_examples.iter().enumerate() {
        let relative_path = write_example(tempdir.path(), rule, "good", index, example);
        let diagnostics = grit::run_grit(tempdir.path(), &compiled, &[relative_path]).unwrap();
        assert!(
            diagnostics.is_empty(),
            "{} Good example {} triggered diagnostics: {:?}",
            rule.id,
            index + 1,
            diagnostics
        );
    }
}

fn write_example(
    root: &Path,
    rule: &RuleDefinition,
    kind: &str,
    index: usize,
    example: &harness_lint::model::RuleExample,
) -> PathBuf {
    let language = example
        .language
        .as_deref()
        .or(rule.language.as_deref())
        .unwrap_or("text");
    let relative_path = PathBuf::from("src").join(format!(
        "{}-{}-{}.{}",
        safe_filename(&rule.id),
        kind,
        index + 1,
        grit::sample_extension(language)
    ));
    let source_path = root.join(&relative_path);
    fs::create_dir_all(source_path.parent().unwrap()).unwrap();
    fs::write(&source_path, source_for(language, &example.code)).unwrap();
    relative_path
}

fn source_for(language: &str, code: &str) -> String {
    if matches!(language.to_ascii_lowercase().as_str(), "go" | "golang")
        && !code.trim_start().starts_with("package ")
    {
        format!("package main\n\n{code}\n")
    } else {
        code.to_string()
    }
}

fn safe_filename(value: &str) -> String {
    value
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || ch == '_' || ch == '-' {
                ch
            } else {
                '-'
            }
        })
        .collect()
}
