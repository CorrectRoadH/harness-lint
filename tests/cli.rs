use std::process::Command;

#[test]
fn cli_init_and_rule_suggest_local_work() {
    let tempdir = tempfile::tempdir().unwrap();
    let binary = env!("CARGO_BIN_EXE_harness-lint");
    let init = Command::new(binary)
        .arg("--cwd")
        .arg(tempdir.path())
        .arg("init")
        .output()
        .unwrap();
    assert!(init.status.success());

    let suggest = Command::new(binary)
        .arg("--cwd")
        .arg(tempdir.path())
        .args(["rule", "suggest", "--local", "Prefer pydantic models"])
        .output()
        .unwrap();
    assert!(suggest.status.success());
    assert!(
        tempdir
            .path()
            .join("rules/prefer-pydantic-models.md")
            .exists()
    );
}
