use std::fs;
use std::path::Path;

use anyhow::{Context, Result};

use crate::config::{
    CACHE_DIR, CONFIG_FILE, GENERATED_GRIT_DIR, PACKS_DIR, USER_RULE_DIR, WORK_DIR, default_config,
};

pub const AI_AGENT_INSTRUCTIONS: &str = r#"When the user expresses a recurring coding preference, create or update a
harness-lint rule instead of only changing the current code.
Run `harness-lint check --changed` before finishing."#;

pub fn init_project(root: &Path, force: bool) -> Result<()> {
    fs::create_dir_all(root.join(USER_RULE_DIR))
        .with_context(|| format!("failed to create {}", root.join(USER_RULE_DIR).display()))?;
    fs::create_dir_all(root.join(PACKS_DIR))
        .with_context(|| format!("failed to create {}", root.join(PACKS_DIR).display()))?;
    fs::create_dir_all(root.join(GENERATED_GRIT_DIR)).with_context(|| {
        format!(
            "failed to create {}",
            root.join(GENERATED_GRIT_DIR).display()
        )
    })?;
    fs::create_dir_all(root.join(CACHE_DIR))
        .with_context(|| format!("failed to create {}", root.join(CACHE_DIR).display()))?;

    let config_path = root.join(CONFIG_FILE);
    if !config_path.exists() || force {
        let project_name = root
            .file_name()
            .and_then(|name| name.to_str())
            .map(ToOwned::to_owned);
        let config = default_config(project_name);
        let content = toml::to_string_pretty(&config).context("failed to serialize config")?;
        fs::write(&config_path, content)
            .with_context(|| format!("failed to write {}", config_path.display()))?;
    }

    if root.join(".grit").exists() {
        eprintln!(
            "harness-lint: existing .grit directory detected; harness generated files will stay under .harness/generated/.grit"
        );
    }

    let gitignore_path = root.join(".gitignore");
    let ignore_line = format!("{WORK_DIR}/");
    if gitignore_path.exists() {
        let content = fs::read_to_string(&gitignore_path)
            .with_context(|| format!("failed to read {}", gitignore_path.display()))?;
        if !content.lines().any(|line| line.trim() == ignore_line) {
            let mut next = content;
            if !next.ends_with('\n') {
                next.push('\n');
            }
            next.push_str(&ignore_line);
            next.push('\n');
            fs::write(&gitignore_path, next)
                .with_context(|| format!("failed to write {}", gitignore_path.display()))?;
        }
    } else {
        fs::write(&gitignore_path, format!("{ignore_line}\n"))
            .with_context(|| format!("failed to write {}", gitignore_path.display()))?;
    }

    Ok(())
}
