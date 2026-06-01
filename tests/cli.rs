use std::fs;
use std::process::Command;

#[test]
fn cli_exposes_version_and_command_descriptions() {
    let binary = env!("CARGO_BIN_EXE_harness-lint");
    let version = Command::new(binary).arg("--version").output().unwrap();
    assert!(
        version.status.success(),
        "{}",
        String::from_utf8_lossy(&version.stderr)
    );
    assert!(String::from_utf8_lossy(&version.stdout).contains(env!("CARGO_PKG_VERSION")));

    let help = Command::new(binary).arg("--help").output().unwrap();
    assert!(
        help.status.success(),
        "{}",
        String::from_utf8_lossy(&help.stderr)
    );
    let stdout = String::from_utf8_lossy(&help.stdout);
    assert!(stdout.contains("Run active rules against selected files"));
    assert!(stdout.contains("Rebuild the local pack cache from harness.lock"));
}

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

#[test]
fn cli_pack_catalog_add_outdated_update_and_remove_work() {
    let tempdir = tempfile::tempdir().unwrap();
    let pack_dir = tempdir.path().join("demo-pack");
    let rules_dir = pack_dir.join("rules");
    fs::create_dir_all(&rules_dir).unwrap();
    fs::write(
        pack_dir.join("harness-pack.toml"),
        r#"[pack]
id = "demo"
name = "Demo Pack"
version = "0.1.0"

[compat]
languages = ["python"]
"#,
    )
    .unwrap();
    fs::write(
        rules_dir.join("no-print.md"),
        r#"---
id: demo.no-print
title: Avoid print
language: python
level: warn
status: warn
tags: [python]
---

# Avoid print

Use logging.

```grit
language python
`print($value)`
```
"#,
    )
    .unwrap();

    let catalog = serde_json::json!([
      {
        "id": "demo",
        "title": "Demo Pack",
        "description": "Rules for tests.",
        "pack_spec": format!("local:{}", pack_dir.display()),
        "languages": ["python"],
        "keywords": ["python"],
        "rules": [
          {
            "rule_id": "demo.no-print",
            "title": "Avoid print",
            "pack_id": "demo",
            "pack_spec": format!("local:{}", pack_dir.display()),
            "score": 10,
            "reason": "Flag print."
          }
        ]
      }
    ]);
    let catalog_path = tempdir.path().join("catalog.json");
    fs::write(
        &catalog_path,
        serde_json::to_string_pretty(&catalog).unwrap(),
    )
    .unwrap();

    let binary = env!("CARGO_BIN_EXE_harness-lint");
    let init = Command::new(binary)
        .arg("--cwd")
        .arg(tempdir.path())
        .arg("init")
        .output()
        .unwrap();
    assert!(init.status.success());

    let config_path = tempdir.path().join("harness.toml");
    let config = fs::read_to_string(&config_path).unwrap();
    let config = config.replace(
        "https://raw.githubusercontent.com/CorrectRoadH/harness-lint/main/site/catalog.json",
        &catalog_path.to_string_lossy(),
    );
    fs::write(&config_path, config).unwrap();

    let available = Command::new(binary)
        .arg("--cwd")
        .arg(tempdir.path())
        .args(["list", "--available"])
        .output()
        .unwrap();
    assert!(
        available.status.success(),
        "{}",
        String::from_utf8_lossy(&available.stderr)
    );
    assert!(String::from_utf8_lossy(&available.stdout).contains("demo"));

    let add = Command::new(binary)
        .arg("--cwd")
        .arg(tempdir.path())
        .args(["install", "demo"])
        .output()
        .unwrap();
    assert!(
        add.status.success(),
        "{}",
        String::from_utf8_lossy(&add.stderr)
    );
    let lock = fs::read_to_string(tempdir.path().join("harness.lock")).unwrap();
    assert!(lock.contains("[packs.demo]"));
    assert!(lock.contains("checksum"));

    fs::write(
        rules_dir.join("no-print.md"),
        r#"---
id: demo.no-print
title: Avoid print
language: python
level: warn
status: warn
tags: [python]
---

# Avoid print

Use logging instead.

```grit
language python
`print($value)`
```
"#,
    )
    .unwrap();

    let outdated = Command::new(binary)
        .arg("--cwd")
        .arg(tempdir.path())
        .arg("outdated")
        .output()
        .unwrap();
    assert!(
        outdated.status.success(),
        "{}",
        String::from_utf8_lossy(&outdated.stderr)
    );
    assert!(String::from_utf8_lossy(&outdated.stdout).contains("local changes detected"));

    let update = Command::new(binary)
        .arg("--cwd")
        .arg(tempdir.path())
        .arg("update")
        .output()
        .unwrap();
    assert!(
        update.status.success(),
        "{}",
        String::from_utf8_lossy(&update.stderr)
    );

    let restore = Command::new(binary)
        .arg("--cwd")
        .arg(tempdir.path())
        .arg("restore")
        .output()
        .unwrap();
    assert!(
        restore.status.success(),
        "{}",
        String::from_utf8_lossy(&restore.stderr)
    );

    fs::write(
        rules_dir.join("no-print.md"),
        r#"---
id: demo.no-print
title: Avoid print
language: python
level: warn
status: warn
tags: [python]
---

# Avoid print

Use structured logging instead.

```grit
language python
`print($value)`
```
"#,
    )
    .unwrap();

    let restore_changed_local = Command::new(binary)
        .arg("--cwd")
        .arg(tempdir.path())
        .arg("restore")
        .output()
        .unwrap();
    assert!(!restore_changed_local.status.success());
    assert!(
        String::from_utf8_lossy(&restore_changed_local.stderr)
            .contains("differs from harness.lock")
    );

    let remove = Command::new(binary)
        .arg("--cwd")
        .arg(tempdir.path())
        .args(["remove", "demo"])
        .output()
        .unwrap();
    assert!(
        remove.status.success(),
        "{}",
        String::from_utf8_lossy(&remove.stderr)
    );
    let config = fs::read_to_string(tempdir.path().join("harness.toml")).unwrap();
    let lock = fs::read_to_string(tempdir.path().join("harness.lock")).unwrap();
    assert!(!config.contains("demo ="));
    assert!(!lock.contains("[packs.demo]"));
}

#[test]
fn built_in_pack_directories_load_as_local_packs() {
    let tempdir = tempfile::tempdir().unwrap();
    let binary = env!("CARGO_BIN_EXE_harness-lint");
    let init = Command::new(binary)
        .arg("--cwd")
        .arg(tempdir.path())
        .arg("init")
        .output()
        .unwrap();
    assert!(init.status.success());

    let manifest_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    for id in [
        "python",
        "python-pep8",
        "python-typing",
        "python-async",
        "go",
        "go-effective-go",
        "go-concurrency",
        "typescript",
        "typescript-react",
    ] {
        let spec = format!("local:{}", manifest_dir.join("packs").join(id).display());
        let add = Command::new(binary)
            .arg("--cwd")
            .arg(tempdir.path())
            .args(["install", id, &spec])
            .output()
            .unwrap();
        assert!(
            add.status.success(),
            "{}: {}",
            id,
            String::from_utf8_lossy(&add.stderr)
        );
    }

    let rules = Command::new(binary)
        .arg("--cwd")
        .arg(tempdir.path())
        .args(["rule", "list"])
        .output()
        .unwrap();
    assert!(
        rules.status.success(),
        "{}",
        String::from_utf8_lossy(&rules.stderr)
    );
    let stdout = String::from_utf8_lossy(&rules.stdout);
    assert!(stdout.contains("python-pep8.no-wildcard-import"));
    assert!(stdout.contains("go-effective-go.no-empty-interface-api"));
    assert!(stdout.contains("typescript-react.no-index-key"));
}
