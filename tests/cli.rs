use std::fs;
use std::process::Command;

fn grit_available() -> bool {
    Command::new("grit")
        .arg("--version")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

fn run_git(repo: &std::path::Path, args: &[&str]) {
    let output = Command::new("git")
        .current_dir(repo)
        .args(args)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "git {:?} failed: {}",
        args,
        String::from_utf8_lossy(&output.stderr)
    );
}

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
    assert!(stdout.contains("Run active rules against the configured project file set"));
    assert!(stdout.contains("Rebuild the local pack cache from harness.lock"));
}

#[test]
fn cli_check_rejects_positional_paths() {
    let tempdir = tempfile::tempdir().unwrap();
    let binary = env!("CARGO_BIN_EXE_harness-lint");
    let output = Command::new(binary)
        .arg("--cwd")
        .arg(tempdir.path())
        .args(["check", "path/to/example.py"])
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("unexpected argument"),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn cli_check_applies_rule_path_exceptions() {
    if !grit_available() {
        return;
    }
    let tempdir = tempfile::tempdir().unwrap();
    fs::create_dir_all(tempdir.path().join("rules")).unwrap();
    fs::create_dir_all(tempdir.path().join("src/generated")).unwrap();
    fs::write(
        tempdir.path().join("harness.toml"),
        r#"
[project]
name = "demo"

[rules]
local = ["rules"]

[ignore]
paths = []

[[exceptions]]
rule = "local.no-print"
paths = ["src/generated/**"]
reason = "Generated fixtures intentionally use print."
"#,
    )
    .unwrap();
    fs::write(
        tempdir.path().join("rules/no-print.md"),
        r#"---
id: local.no-print
title: Avoid print debugging
language: python
level: warn
tags: [local, python]
---

# Avoid print debugging

Use logging instead.

```grit
language python
`print($value)`
```

## Bad

```python
print(user)
```

## Good

```python
logger.info("user=%s", user)
```
"#,
    )
    .unwrap();
    fs::write(tempdir.path().join("src/app.py"), "print('visible')\n").unwrap();
    fs::write(
        tempdir.path().join("src/generated/adapter.py"),
        "print('hidden')\n",
    )
    .unwrap();

    let binary = env!("CARGO_BIN_EXE_harness-lint");
    let output = Command::new(binary)
        .arg("--cwd")
        .arg(tempdir.path())
        .args(["check", "--all"])
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("src/app.py"), "{stdout}");
    assert!(!stdout.contains("src/generated/adapter.py"), "{stdout}");
}

#[test]
fn cli_warns_for_legacy_suppressions_key() {
    let tempdir = tempfile::tempdir().unwrap();
    fs::create_dir_all(tempdir.path().join("rules")).unwrap();
    fs::write(
        tempdir.path().join("harness.toml"),
        r#"
[rules]
local = ["rules"]

[[suppressions]]
rule = "local.no-print"
paths = ["src/generated/**"]
"#,
    )
    .unwrap();

    let binary = env!("CARGO_BIN_EXE_harness-lint");
    let output = Command::new(binary)
        .arg("--cwd")
        .arg(tempdir.path())
        .args(["rule", "list"])
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("`[[suppressions]]` is deprecated"),
        "{stderr}"
    );
    assert!(stderr.contains("[[exceptions]]"), "{stderr}");
}

#[test]
fn cli_check_all_keeps_mixed_language_rules_on_matching_files() {
    if !grit_available() {
        return;
    }
    let tempdir = tempfile::tempdir().unwrap();
    fs::create_dir_all(tempdir.path().join("rules")).unwrap();
    fs::create_dir_all(tempdir.path().join("src")).unwrap();
    fs::write(
        tempdir.path().join("harness.toml"),
        r#"
[project]
name = "demo"

[lint]
default_level = "warn"
changed_base = "origin/main"
cache = false

[rules]
local = ["rules"]

[ignore]
paths = []
"#,
    )
    .unwrap();
    fs::write(
        tempdir.path().join("rules/no-var.md"),
        r#"---
id: local.no-var
title: Avoid var declarations
language: typescript
level: warn
tags: [typescript]
---

# Avoid var declarations

Use let or const.

```grit
language js
`var $name = $value`
```

## Bad

```ts
var total = 0
```

## Good

```ts
let total = 0
```
"#,
    )
    .unwrap();
    fs::write(
        tempdir.path().join("rules/no-panic.md"),
        r#"---
id: local.no-panic
title: Avoid panic
language: go
level: warn
tags: [go]
---

# Avoid panic

Return errors.

```grit
language go
`panic($value)`
```

## Bad

```go
panic(err)
```

## Good

```go
return err
```
"#,
    )
    .unwrap();
    fs::write(
        tempdir.path().join("src/main.go"),
        "package main\n\nfunc main() {\n\tvar total = 1\n\tpanic(total)\n}\n",
    )
    .unwrap();
    fs::write(tempdir.path().join("src/app.ts"), "var total = 0\n").unwrap();

    let binary = env!("CARGO_BIN_EXE_harness-lint");
    let output = Command::new(binary)
        .arg("--cwd")
        .arg(tempdir.path())
        .args(["check", "--all"])
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("src/app.ts"), "{stdout}");
    assert!(stdout.contains("src/main.go"), "{stdout}");
    assert!(stdout.contains("local.no-var"), "{stdout}");
    assert!(stdout.contains("local.no-panic"), "{stdout}");
    assert!(!stdout.contains("src/main.go: local.no-var"), "{stdout}");
}

#[test]
fn cli_init_and_rule_create_work() {
    if !grit_available() {
        return;
    }
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
        .args([
            "rule",
            "create",
            "Prefer pydantic models",
            "--language",
            "python",
            "--grit",
            "`print($value)`",
        ])
        .output()
        .unwrap();
    assert!(suggest.status.success());
    assert!(
        tempdir
            .path()
            .join("rules/Prefer pydantic models.md")
            .exists()
    );
}

#[test]
fn cli_rule_verify_checks_bad_examples() {
    if !grit_available() {
        return;
    }
    let tempdir = tempfile::tempdir().unwrap();
    let binary = env!("CARGO_BIN_EXE_harness-lint");
    let init = Command::new(binary)
        .arg("--cwd")
        .arg(tempdir.path())
        .arg("init")
        .output()
        .unwrap();
    assert!(init.status.success());

    fs::write(
        tempdir.path().join("rules/no-console.md"),
        r#"---
id: local.no-console
title: Avoid console logging
language: typescript
level: warn
tags: [local, typescript]
---

# Avoid console logging

Use structured logging.

```grit
language js
`console.log($value)`
```

## Bad

```typescript
console.log(user);
```

## Good

```typescript
logger.info("user=%s", user);
```
"#,
    )
    .unwrap();

    let verify = Command::new(binary)
        .arg("--cwd")
        .arg(tempdir.path())
        .args(["rule", "verify", "local.no-console"])
        .output()
        .unwrap();
    assert!(
        verify.status.success(),
        "{}",
        String::from_utf8_lossy(&verify.stderr)
    );
    assert!(
        String::from_utf8_lossy(&verify.stdout).contains("Verified 1 rule(s), 1 Bad example(s).")
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

#[cfg(unix)]
#[test]
fn cli_git_pack_install_retries_reports_and_cleans_failed_tmp() {
    use std::os::unix::fs::PermissionsExt;

    let tempdir = tempfile::tempdir().unwrap();
    let binary = env!("CARGO_BIN_EXE_harness-lint");
    let init = Command::new(binary)
        .arg("--cwd")
        .arg(tempdir.path())
        .arg("init")
        .output()
        .unwrap();
    assert!(init.status.success());

    let bin_dir = tempdir.path().join("bin");
    fs::create_dir_all(&bin_dir).unwrap();
    let fake_git = bin_dir.join("git");
    fs::write(
        &fake_git,
        r#"#!/bin/sh
target=""
for arg in "$@"; do
  target="$arg"
done
mkdir -p "$target"
echo "fake git clone failed" >&2
exit 1
"#,
    )
    .unwrap();
    let mut permissions = fs::metadata(&fake_git).unwrap().permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(&fake_git, permissions).unwrap();

    let path = format!(
        "{}:{}",
        bin_dir.display(),
        std::env::var("PATH").unwrap_or_default()
    );
    let install = Command::new(binary)
        .arg("--cwd")
        .arg(tempdir.path())
        .args([
            "install",
            "broken",
            "github:CorrectRoadH/harness-lint@main#packs/broken",
        ])
        .env("PATH", path)
        .output()
        .unwrap();

    assert!(!install.status.success());
    let stderr = String::from_utf8_lossy(&install.stderr);
    assert!(stderr.contains("attempt 1/3"), "{stderr}");
    assert!(stderr.contains("attempt 3/3"), "{stderr}");
    assert!(stderr.contains("after 3 attempts"), "{stderr}");
    assert!(stderr.contains("temporary checkout"), "{stderr}");
    assert!(!tempdir.path().join(".harness/packs/broken.tmp").exists());

    let config = fs::read_to_string(tempdir.path().join("harness.toml")).unwrap();
    assert!(!config.contains("broken ="));
    let lock = fs::read_to_string(tempdir.path().join("harness.lock")).unwrap_or_default();
    assert!(!lock.contains("[packs.broken]"));
    let repos_dir = tempdir.path().join(".harness/repos");
    if repos_dir.exists() {
        let leftovers: Vec<_> = fs::read_dir(&repos_dir)
            .unwrap()
            .map(|entry| entry.unwrap().path())
            .filter(|path| path.extension().and_then(|extension| extension.to_str()) == Some("tmp"))
            .collect();
        assert!(leftovers.is_empty(), "{leftovers:?}");
    }
}

#[test]
fn cli_git_pack_install_reuses_repo_cache_for_same_repo_ref() {
    let tempdir = tempfile::tempdir().unwrap();
    let repo_dir = tempdir.path().join("pack-repo");
    for id in ["one", "two"] {
        let rules_dir = repo_dir.join("packs").join(id).join("rules");
        fs::create_dir_all(&rules_dir).unwrap();
        fs::write(
            repo_dir.join("packs").join(id).join("harness-pack.toml"),
            format!(
                r#"[pack]
id = "{id}"
name = "{id}"
version = "0.1.0"

[compat]
languages = ["go"]
"#
            ),
        )
        .unwrap();
        fs::write(
            rules_dir.join("no-fmt-print.md"),
            format!(
                r#"---
id: {id}.no-fmt-print
title: Avoid fmt print
language: go
level: warn
tags: [go]
---

# Avoid fmt print

Use structured logging.

```grit
language go
`fmt.Println($value)`
```
"#
            ),
        )
        .unwrap();
    }

    for args in [
        vec!["init", "-b", "main"],
        vec!["config", "user.email", "test@example.com"],
        vec!["config", "user.name", "Harness Test"],
        vec!["add", "."],
        vec!["commit", "-m", "add packs"],
    ] {
        let output = Command::new("git")
            .current_dir(&repo_dir)
            .args(args)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    let binary = env!("CARGO_BIN_EXE_harness-lint");
    let init = Command::new(binary)
        .arg("--cwd")
        .arg(tempdir.path())
        .arg("init")
        .output()
        .unwrap();
    assert!(init.status.success());

    for id in ["one", "two"] {
        let spec = format!("git:{}@main#packs/{id}", repo_dir.display());
        let install = Command::new(binary)
            .arg("--cwd")
            .arg(tempdir.path())
            .args(["install", id, &spec])
            .output()
            .unwrap();
        assert!(
            install.status.success(),
            "{}",
            String::from_utf8_lossy(&install.stderr)
        );
    }

    let repo_cache_entries: Vec<_> = fs::read_dir(tempdir.path().join(".harness/repos"))
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.is_dir())
        .collect();
    assert_eq!(repo_cache_entries.len(), 1, "{repo_cache_entries:?}");
    assert!(
        tempdir
            .path()
            .join(".harness/packs/one/harness-pack.toml")
            .exists()
    );
    assert!(
        tempdir
            .path()
            .join(".harness/packs/two/harness-pack.toml")
            .exists()
    );
}

#[test]
fn cli_git_pack_update_refreshes_existing_repo_cache() {
    let tempdir = tempfile::tempdir().unwrap();
    let repo_dir = tempdir.path().join("pack-repo");
    let pack_dir = repo_dir.join("packs/demo");
    let rules_dir = pack_dir.join("rules");
    fs::create_dir_all(&rules_dir).unwrap();
    fs::write(
        pack_dir.join("harness-pack.toml"),
        r#"[pack]
id = "demo"
name = "Demo"
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

    run_git(&repo_dir, &["init", "-b", "main"]);
    run_git(&repo_dir, &["config", "user.email", "test@example.com"]);
    run_git(&repo_dir, &["config", "user.name", "Harness Test"]);
    run_git(&repo_dir, &["add", "."]);
    run_git(&repo_dir, &["commit", "-m", "initial pack"]);
    let old_commit = Command::new("git")
        .current_dir(&repo_dir)
        .args(["rev-parse", "HEAD"])
        .output()
        .unwrap();
    assert!(old_commit.status.success());
    let old_commit = String::from_utf8_lossy(&old_commit.stdout)
        .trim()
        .to_string();

    let binary = env!("CARGO_BIN_EXE_harness-lint");
    let init = Command::new(binary)
        .arg("--cwd")
        .arg(tempdir.path())
        .arg("init")
        .output()
        .unwrap();
    assert!(init.status.success());

    let spec = format!("git:{}@main#packs/demo", repo_dir.display());
    let install = Command::new(binary)
        .arg("--cwd")
        .arg(tempdir.path())
        .args(["install", "demo", &spec])
        .output()
        .unwrap();
    assert!(
        install.status.success(),
        "{}",
        String::from_utf8_lossy(&install.stderr)
    );
    let installed =
        fs::read_to_string(tempdir.path().join(".harness/packs/demo/rules/no-print.md")).unwrap();
    assert!(installed.contains("Use logging."));

    fs::write(
        rules_dir.join("no-print.md"),
        r#"---
id: demo.no-print
title: Avoid print
language: python
level: warn
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
    run_git(&repo_dir, &["add", "."]);
    run_git(&repo_dir, &["commit", "-m", "update pack"]);
    let new_commit = Command::new("git")
        .current_dir(&repo_dir)
        .args(["rev-parse", "HEAD"])
        .output()
        .unwrap();
    assert!(new_commit.status.success());
    let new_commit = String::from_utf8_lossy(&new_commit.stdout)
        .trim()
        .to_string();
    assert_ne!(old_commit, new_commit);

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
    let stdout = String::from_utf8_lossy(&update.stdout);
    assert!(stdout.contains(&format!("Updated pack `demo` to {new_commit}.")));

    let installed =
        fs::read_to_string(tempdir.path().join(".harness/packs/demo/rules/no-print.md")).unwrap();
    assert!(installed.contains("Use structured logging instead."));
    let lock = fs::read_to_string(tempdir.path().join("harness.lock")).unwrap();
    assert!(lock.contains(&format!("version = \"{new_commit}\"")));
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
        "rust",
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
    assert!(stdout.contains("python-pep8.no-lambda-assignment"));
    assert!(stdout.contains("go-effective-go.no-empty-interface-api"));
    assert!(stdout.contains("rust.no-dbg-macro"));
    assert!(stdout.contains("typescript-react.no-children-prop"));
    assert!(stdout.contains("typescript-react.no-index-key"));
}
