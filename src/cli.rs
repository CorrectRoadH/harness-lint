use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use anyhow::{Context, Result, anyhow, bail};
use clap::{Args, Parser, Subcommand};

use crate::authoring;
use crate::cache;
use crate::compiler;
use crate::config::{self, ProjectConfig};
use crate::git;
use crate::grit;
use crate::init;
use crate::model::{PackSourceKind, RuleDefinition};
use crate::obsidian;
use crate::pack;
use crate::paths;
use crate::registry;
use crate::report::{self, DiagnosticReportOptions, ReportFormat};

const GRIT_BATCH_SIZE: usize = 256;

#[derive(Debug, Parser)]
#[command(name = "harness-lint")]
#[command(version)]
#[command(about = "GritQL rule ecosystem and AI feedback linter")]
pub struct Cli {
    #[arg(long)]
    config: Option<PathBuf>,
    #[arg(long)]
    cwd: Option<PathBuf>,
    #[arg(long)]
    json: bool,
    #[arg(long, short)]
    verbose: bool,
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    #[command(about = "Create harness.toml and local rule scaffolding")]
    Init(InitCommand),
    #[command(about = "Diagnose config, rules, grit, and git integration")]
    Doctor,
    #[command(about = "Run active rules against selected files")]
    Check(CheckCommand),
    #[command(about = "Search the rule-pack catalog")]
    Search { query: Vec<String> },
    #[command(about = "Show details for a catalog rule pack")]
    Inspect { id: String },
    #[command(about = "Install a rule pack")]
    Install { id: String, spec: Option<String> },
    #[command(about = "Refresh installed rule packs and rewrite the lock file")]
    Update,
    #[command(about = "Rebuild the local pack cache from harness.lock")]
    Restore,
    #[command(about = "Check installed rule packs for updates")]
    Outdated,
    #[command(about = "Remove an installed rule pack")]
    Remove { id: String },
    #[command(about = "List installed or available rule packs")]
    List {
        #[arg(long)]
        available: bool,
    },
    #[command(about = "List, explain, create, or suggest local rules")]
    Rule {
        #[command(subcommand)]
        command: RuleCommand,
    },
}

#[derive(Debug, Args)]
struct InitCommand {
    #[arg(long)]
    force: bool,
}

#[derive(Debug, Args)]
struct CheckCommand {
    #[arg(long)]
    all: bool,
    #[arg(long)]
    changed: bool,
    #[arg(long)]
    staged: bool,
    #[arg(long)]
    base: Option<String>,
    #[arg(long)]
    rule: Vec<String>,
    #[arg(long)]
    tag: Vec<String>,
    #[arg(value_name = "PATH")]
    paths: Vec<PathBuf>,
}

#[derive(Debug)]
enum CatalogCommand {
    Search { query: Vec<String> },
    Inspect { id: String },
    Install { id: String, spec: Option<String> },
    Update,
    Restore,
    Outdated,
    Remove { id: String },
    List { available: bool },
}

#[derive(Debug, Subcommand)]
enum RuleCommand {
    #[command(about = "List loaded rules")]
    List,
    #[command(about = "Explain a loaded rule")]
    Explain { rule_id: String },
    #[command(about = "Create a local rule draft")]
    New {
        id: String,
        title: String,
        #[arg(long)]
        language: Option<String>,
    },
    #[command(about = "Create a local rule draft from feedback")]
    Draft { feedback: String },
    #[command(about = "Find existing rule candidates from feedback")]
    Suggest { feedback: String },
}

pub fn run() -> Result<ExitCode> {
    let cli = Cli::parse();
    let format = if cli.json {
        ReportFormat::Json
    } else {
        ReportFormat::Human
    };

    let cwd = cli
        .cwd
        .clone()
        .unwrap_or(std::env::current_dir().context("failed to read current directory")?);

    match cli.command {
        Command::Init(command) => {
            let root = cwd.canonicalize().unwrap_or(cwd);
            init::init_project(&root, command.force)?;
            println!("Initialized harness-lint in {}", root.display());
            println!("\nAI agent instructions:\n{}", init::AI_AGENT_INSTRUCTIONS);
            Ok(())
        }
        Command::Doctor => run_doctor(&cwd, cli.config.as_deref(), format),
        Command::Check(command) => {
            run_check(&cwd, cli.config.as_deref(), command, format, cli.verbose)
        }
        Command::Search { query } => run_catalog(
            &cwd,
            cli.config.as_deref(),
            CatalogCommand::Search { query },
            format,
        ),
        Command::Inspect { id } => run_catalog(
            &cwd,
            cli.config.as_deref(),
            CatalogCommand::Inspect { id },
            format,
        ),
        Command::Install { id, spec } => run_catalog(
            &cwd,
            cli.config.as_deref(),
            CatalogCommand::Install { id, spec },
            format,
        ),
        Command::Update => run_catalog(&cwd, cli.config.as_deref(), CatalogCommand::Update, format),
        Command::Restore => {
            run_catalog(&cwd, cli.config.as_deref(), CatalogCommand::Restore, format)
        }
        Command::Outdated => run_catalog(
            &cwd,
            cli.config.as_deref(),
            CatalogCommand::Outdated,
            format,
        ),
        Command::Remove { id } => run_catalog(
            &cwd,
            cli.config.as_deref(),
            CatalogCommand::Remove { id },
            format,
        ),
        Command::List { available } => run_catalog(
            &cwd,
            cli.config.as_deref(),
            CatalogCommand::List { available },
            format,
        ),
        Command::Rule { command } => run_rule(&cwd, cli.config.as_deref(), command, format),
    }
    .map(|_| ExitCode::SUCCESS)
}

fn run_check(
    cwd: &PathBuf,
    config_path: Option<&std::path::Path>,
    command: CheckCommand,
    format: ReportFormat,
    verbose: bool,
) -> Result<()> {
    let root = config::find_project_root(cwd)?;
    let config = config::load_config(&root, config_path)?;
    let packs = load_rule_packs(&root, &config)?;
    let active_rules = collect_effective_rules(&packs, &config, &command);
    let selected_paths = select_paths(&root, &config, &command, &active_rules)?;
    if verbose {
        eprintln!(
            "harness-lint: {} active rule(s), {} GritQL path(s), {} Obsidian path(s)",
            active_rules.len(),
            selected_paths.grit.len(),
            selected_paths.obsidian.len()
        );
    }
    if selected_paths.grit.is_empty() && selected_paths.obsidian.is_empty() {
        report::report_diagnostics(&[], format, DiagnosticReportOptions { root: &root })?;
        return Ok(());
    }
    let mut diagnostics = if active_rules.is_empty() || selected_paths.grit.is_empty() {
        Vec::new()
    } else {
        let compiled = compiler::compile_rule_set(&root, active_rules.clone())?;
        if config.lint.cache {
            let rule_fingerprint = cache::fingerprint(&format!("{:?}", compiled.grit_rules));
            let config_fingerprint = cache::fingerprint(&format!(
                "{:?}{:?}{:?}{:?}{:?}",
                command.rule, command.tag, config.overrides, config.disabled, config.ignore
            ));
            run_cached_check(
                &root,
                &compiled,
                &selected_paths.grit,
                &rule_fingerprint,
                &config_fingerprint,
                verbose,
            )?
        } else {
            run_uncached_check(&root, &compiled, &selected_paths.grit)?
        }
    };
    diagnostics.extend(obsidian::run_checks(
        &root,
        &config.obsidian,
        &selected_paths.obsidian,
    )?);
    diagnostics.sort_by(|left, right| {
        left.path
            .cmp(&right.path)
            .then(left.start_line.cmp(&right.start_line))
            .then(left.start_column.cmp(&right.start_column))
            .then(left.rule_id.cmp(&right.rule_id))
    });
    report::report_diagnostics(
        &diagnostics,
        format,
        DiagnosticReportOptions { root: &root },
    )?;
    if diagnostics
        .iter()
        .any(|diagnostic| diagnostic.level.is_failing())
    {
        bail!("harness-lint found error-level diagnostics");
    }
    Ok(())
}

#[derive(Debug, serde::Serialize)]
struct DoctorFinding {
    status: DoctorStatus,
    check: &'static str,
    message: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "lowercase")]
enum DoctorStatus {
    Ok,
    Warn,
    Error,
}

fn run_doctor(
    cwd: &PathBuf,
    config_path: Option<&std::path::Path>,
    format: ReportFormat,
) -> Result<()> {
    let mut findings = Vec::new();
    let root = match config::find_project_root(cwd) {
        Ok(root) => {
            findings.push(doctor_ok(
                "project-root",
                format!("found project root at {}", root.display()),
            ));
            root
        }
        Err(error) => {
            findings.push(doctor_error("project-root", error.to_string()));
            report_doctor(&findings, format)?;
            bail!("harness-lint doctor found error-level issues");
        }
    };

    let config_path_display = config_path
        .map(|path| path.display().to_string())
        .unwrap_or_else(|| root.join(config::CONFIG_FILE).display().to_string());
    let config = match config::load_config(&root, config_path) {
        Ok(config) => {
            findings.push(doctor_ok(
                "config",
                format!("loaded configuration from {config_path_display}"),
            ));
            config
        }
        Err(error) => {
            findings.push(doctor_error("config", error.to_string()));
            report_doctor(&findings, format)?;
            bail!("harness-lint doctor found error-level issues");
        }
    };

    if config.rules.local.is_empty() {
        findings.push(doctor_warn(
            "local-rules",
            "no local rule directories are configured".to_string(),
        ));
    }
    for dir in &config.rules.local {
        let path = if dir.is_absolute() {
            dir.clone()
        } else {
            root.join(dir)
        };
        if path.exists() {
            findings.push(doctor_ok(
                "local-rules",
                format!("local rule directory exists at {}", path.display()),
            ));
        } else {
            findings.push(doctor_warn(
                "local-rules",
                format!("local rule directory does not exist: {}", path.display()),
            ));
        }
    }

    match load_rule_packs(&root, &config) {
        Ok(packs) => {
            let rule_count: usize = packs.iter().map(|pack| pack.rules.len()).sum();
            findings.push(doctor_ok(
                "rules",
                format!("loaded {} pack(s) with {} rule(s)", packs.len(), rule_count),
            ));
        }
        Err(error) => findings.push(doctor_error("rules", error.to_string())),
    }

    match std::process::Command::new("grit").arg("--version").output() {
        Ok(output) if output.status.success() => {
            let version = String::from_utf8_lossy(&output.stdout);
            let version = version.trim();
            let message = if version.is_empty() {
                "grit is installed".to_string()
            } else {
                format!("grit is installed: {version}")
            };
            findings.push(doctor_ok("grit", message));
        }
        Ok(output) => {
            let stderr = String::from_utf8_lossy(&output.stderr);
            findings.push(doctor_error(
                "grit",
                format!("grit --version failed: {}", stderr.trim()),
            ));
        }
        Err(error) => {
            findings.push(doctor_error(
                "grit",
                format!("grit is required for `check` but was not found: {error}"),
            ));
        }
    }

    match std::process::Command::new("git")
        .current_dir(&root)
        .args(["rev-parse", "--is-inside-work-tree"])
        .output()
    {
        Ok(output) if output.status.success() => findings.push(doctor_ok(
            "git",
            "git repository is available for changed/staged checks".to_string(),
        )),
        Ok(_) => findings.push(doctor_warn(
            "git",
            "not inside a git worktree; changed/staged checks may be unavailable".to_string(),
        )),
        Err(error) => findings.push(doctor_warn(
            "git",
            format!("git command is unavailable: {error}"),
        )),
    }

    report_doctor(&findings, format)?;
    if findings
        .iter()
        .any(|finding| finding.status == DoctorStatus::Error)
    {
        bail!("harness-lint doctor found error-level issues");
    }
    Ok(())
}

fn doctor_ok(check: &'static str, message: String) -> DoctorFinding {
    DoctorFinding {
        status: DoctorStatus::Ok,
        check,
        message,
    }
}

fn doctor_warn(check: &'static str, message: String) -> DoctorFinding {
    DoctorFinding {
        status: DoctorStatus::Warn,
        check,
        message,
    }
}

fn doctor_error(check: &'static str, message: String) -> DoctorFinding {
    DoctorFinding {
        status: DoctorStatus::Error,
        check,
        message,
    }
}

fn report_doctor(findings: &[DoctorFinding], format: ReportFormat) -> Result<()> {
    match format {
        ReportFormat::Json => {
            println!("{}", serde_json::to_string_pretty(findings)?);
        }
        ReportFormat::Human => {
            for finding in findings {
                let status = match finding.status {
                    DoctorStatus::Ok => "ok",
                    DoctorStatus::Warn => "warn",
                    DoctorStatus::Error => "error",
                };
                println!("[{status}] {}: {}", finding.check, finding.message);
            }
        }
    }
    Ok(())
}

fn run_cached_check(
    root: &Path,
    compiled: &crate::model::CompiledRules,
    paths: &[PathBuf],
    rule_fingerprint: &str,
    config_fingerprint: &str,
    verbose: bool,
) -> Result<Vec<crate::model::Diagnostic>> {
    let mut diagnostics = Vec::new();
    let mut misses = Vec::new();
    let mut miss_keys = BTreeMap::new();

    for path in paths {
        let file_hash = cache::file_hash(root, path)?;
        let key = cache::file_cache_key(path, &file_hash, rule_fingerprint, config_fingerprint);
        if let Some(cached) = cache::load_file(root, &key)? {
            diagnostics.extend(cached);
        } else {
            misses.push(path.clone());
            miss_keys.insert(path.clone(), key);
        }
    }

    if verbose {
        eprintln!(
            "harness-lint: {} cache hit(s), {} cache miss(es)",
            paths.len().saturating_sub(misses.len()),
            misses.len()
        );
    }

    for (index, batch) in misses.chunks(GRIT_BATCH_SIZE).enumerate() {
        if verbose {
            eprintln!(
                "harness-lint: running GritQL batch {}/{} ({} file(s))",
                index + 1,
                misses.len().div_ceil(GRIT_BATCH_SIZE),
                batch.len()
            );
        }
        let batch_paths = batch.to_vec();
        let fresh = normalize_diagnostics(root, grit::run_grit(root, compiled, &batch_paths)?);
        let mut by_path = cache::group_by_path(root, fresh);
        for path in batch {
            let path_diagnostics = by_path.remove(path).unwrap_or_default();
            if let Some(key) = miss_keys.get(path) {
                cache::store_file(root, key, path_diagnostics.clone())?;
            }
            diagnostics.extend(path_diagnostics);
        }
    }

    diagnostics.sort_by(|left, right| {
        left.path
            .cmp(&right.path)
            .then(left.start_line.cmp(&right.start_line))
            .then(left.start_column.cmp(&right.start_column))
            .then(left.rule_id.cmp(&right.rule_id))
    });
    Ok(diagnostics)
}

fn run_uncached_check(
    root: &Path,
    compiled: &crate::model::CompiledRules,
    paths: &[PathBuf],
) -> Result<Vec<crate::model::Diagnostic>> {
    let mut diagnostics = Vec::new();
    for batch in paths.chunks(GRIT_BATCH_SIZE) {
        diagnostics.extend(normalize_diagnostics(
            root,
            grit::run_grit(root, compiled, batch)?,
        ));
    }
    Ok(diagnostics)
}

fn normalize_diagnostics(
    root: &Path,
    diagnostics: Vec<crate::model::Diagnostic>,
) -> Vec<crate::model::Diagnostic> {
    diagnostics
        .into_iter()
        .map(|mut diagnostic| {
            diagnostic.path = cache::normalize_path(root, &diagnostic.path);
            diagnostic
        })
        .collect()
}

fn run_catalog(
    cwd: &PathBuf,
    config_path: Option<&std::path::Path>,
    command: CatalogCommand,
    _format: ReportFormat,
) -> Result<()> {
    let root = config::find_project_root(cwd)?;
    match command {
        CatalogCommand::Install { id, spec } => {
            let mut config = config::load_config(&root, config_path)?;
            let mut lock = config::load_lock(&root)?;
            let spec = match spec {
                Some(spec) => spec,
                None => {
                    registry::inspect_pack(&id, config.registry.url.as_deref())?
                        .ok_or_else(|| anyhow!("pack `{id}` was not found in the catalog"))?
                        .pack_spec
                }
            };
            let parsed = pack::parse_pack_spec(&id, &spec);
            let resolved = match parsed.source {
                PackSourceKind::Local => pack::resolve_local_pack(&root, parsed)?,
                PackSourceKind::Git => pack::install_git_pack(&root, parsed)?,
                _ => bail!("unsupported pack source for `{id}`"),
            };
            let loaded = pack::load_rule_pack(&resolved)?;
            lock.packs
                .insert(id.clone(), pack::lock_entry(&resolved, &root));
            config::write_lock(&root, &lock)?;
            config.packs.insert(id.clone(), spec.clone());
            config::write_config(&root, &config)?;
            println!(
                "Added pack `{id}` = `{spec}` ({} rules)",
                loaded.rules.len()
            );
        }
        CatalogCommand::Search { query } => {
            let config = config::load_config(&root, config_path)?;
            if query.is_empty() {
                print_available_packs(config.registry.url.as_deref())?;
                return Ok(());
            }
            let mut registry_query = registry::infer_project_context(&root);
            registry_query.feedback = query.join(" ");
            let candidates =
                registry::search_registry(&registry_query, config.registry.url.as_deref())?;
            if candidates.is_empty() {
                println!("No matching packs found.");
                return Ok(());
            }
            println!("Matching rule packs:");
            for candidate in candidates {
                println!(
                    "- {}: {} ({}) score={}",
                    candidate.pack_id, candidate.title, candidate.rule_id, candidate.score
                );
                println!("  {}", candidate.reason);
                println!("  inspect: harness-lint inspect {}", candidate.pack_id);
                println!("  install: harness-lint install {}", candidate.pack_id);
            }
        }
        CatalogCommand::Inspect { id } => {
            let config = config::load_config(&root, config_path)?;
            let pack = registry::inspect_pack(&id, config.registry.url.as_deref())?
                .ok_or_else(|| anyhow!("pack `{id}` was not found in the catalog"))?;
            println!("{} ({})", pack.title, pack.id);
            println!("{}", pack.description);
            println!("languages: {}", pack.languages.join(", "));
            println!("rules:");
            for rule in &pack.rules {
                println!("- {}: {}", rule.rule_id, rule.title);
                println!("  {}", rule.reason);
            }
            println!();
            println!("source: {}", pack.pack_spec);
            println!("install: harness-lint install {}", pack.id);
        }
        CatalogCommand::Update => update_configured_packs(&root, config_path)?,
        CatalogCommand::Restore => restore_locked_packs(&root, config_path)?,
        CatalogCommand::Outdated => {
            let config = config::load_config(&root, config_path)?;
            let lock = config::load_lock(&root)?;
            let mut found = 0usize;
            for (id, spec) in &config.packs {
                let parsed = pack::parse_pack_spec(id, spec);
                match parsed.source {
                    PackSourceKind::Git => {
                        let Some(entry) = lock.packs.get(id) else {
                            found += 1;
                            println!("{id}\tnot installed; run harness-lint update");
                            continue;
                        };
                        let status = pack::check_git_pack_update(&root, parsed, Some(entry))?;
                        if status.update_available {
                            found += 1;
                            println!(
                                "{id}\t{} -> {}",
                                status.installed_checksum.as_deref().unwrap_or("-"),
                                status.latest_checksum.as_deref().unwrap_or("-")
                            );
                        }
                    }
                    PackSourceKind::Local => {
                        let resolved = pack::resolve_local_pack(&root, parsed)?;
                        let Some(installed) =
                            lock.packs.get(id).and_then(|entry| entry.checksum.clone())
                        else {
                            found += 1;
                            println!("{id}\tlock entry missing; run harness-lint update");
                            continue;
                        };
                        if Some(installed) != resolved.checksum {
                            found += 1;
                            println!("{id}\tlocal changes detected");
                        }
                    }
                    _ => {}
                }
            }
            if found == 0 {
                println!("All packs are up to date.");
            }
        }
        CatalogCommand::List { available } => {
            let config = config::load_config(&root, config_path)?;
            if available {
                print_available_packs(config.registry.url.as_deref())?;
                return Ok(());
            }
            let lock = config::load_lock(&root)?;
            if config.packs.is_empty() {
                println!("No external packs configured.");
            }
            for (id, spec) in &config.packs {
                if let Some(entry) = lock.packs.get(id) {
                    println!(
                        "{id}\t{spec}\t{}",
                        entry.version.as_deref().unwrap_or("unversioned")
                    );
                } else {
                    println!("{id}\t{spec}\tnot installed");
                }
            }
        }
        CatalogCommand::Remove { id } => {
            let mut config = config::load_config(&root, config_path)?;
            let mut lock = config::load_lock(&root)?;
            let removed_config = config.packs.remove(&id).is_some();
            let removed_lock = lock.packs.remove(&id).is_some();
            let cached = root.join(config::PACKS_DIR).join(&id);
            if cached.exists() {
                fs::remove_dir_all(&cached)
                    .with_context(|| format!("failed to remove {}", cached.display()))?;
            }
            config::write_config(&root, &config)?;
            config::write_lock(&root, &lock)?;
            if removed_config || removed_lock {
                println!("Removed pack `{id}`.");
            } else {
                println!("Pack `{id}` was not installed.");
            }
        }
    }
    Ok(())
}

fn print_available_packs(registry_url: Option<&str>) -> Result<()> {
    let packs = registry::list_packs(registry_url)?;
    if packs.is_empty() {
        println!("No packs are available in the catalog.");
        return Ok(());
    }
    println!("Available rule packs:");
    for pack in packs {
        println!(
            "- {}: {} [{}]",
            pack.id,
            pack.title,
            pack.languages.join(", ")
        );
        println!("  {}", pack.description);
        println!("  install: harness-lint install {}", pack.id);
    }
    Ok(())
}

fn update_configured_packs(root: &Path, config_path: Option<&std::path::Path>) -> Result<()> {
    let config = config::load_config(root, config_path)?;
    let mut lock = config::load_lock(root)?;
    let mut updated = 0usize;
    for (id, spec) in &config.packs {
        let parsed = pack::parse_pack_spec(id, spec);
        let previous_checksum = lock.packs.get(id).and_then(|entry| entry.checksum.clone());
        match parsed.source {
            PackSourceKind::Local => {
                let resolved = pack::resolve_local_pack(root, parsed)?;
                let changed = previous_checksum != resolved.checksum;
                lock.packs
                    .insert(id.clone(), pack::lock_entry(&resolved, root));
                if changed {
                    updated += 1;
                    println!("Refreshed local pack `{id}`.");
                }
            }
            PackSourceKind::Git => {
                let resolved = pack::install_git_pack(root, parsed)?;
                let changed = previous_checksum != resolved.checksum;
                lock.packs
                    .insert(id.clone(), pack::lock_entry(&resolved, root));
                if changed {
                    updated += 1;
                    println!(
                        "Updated pack `{id}` to {}.",
                        resolved.version.as_deref().unwrap_or("latest")
                    );
                } else {
                    println!("Pack `{id}` is already up to date.");
                }
            }
            _ => bail!("unsupported pack source for `{id}`"),
        }
    }
    config::write_lock(root, &lock)?;
    println!("Updated {updated} of {} pack(s).", config.packs.len());
    Ok(())
}

fn restore_locked_packs(root: &Path, config_path: Option<&std::path::Path>) -> Result<()> {
    let config = config::load_config(root, config_path)?;
    let lock = config::load_lock(root)?;
    let mut restored = 0usize;
    for (id, spec) in &config.packs {
        let Some(entry) = lock.packs.get(id) else {
            bail!("pack `{id}` is missing from harness.lock; run `harness-lint update` first");
        };
        let parsed = pack::parse_pack_spec(id, spec);
        if parsed.source != entry.source || parsed.spec != entry.spec {
            bail!(
                "pack `{id}` differs between harness.toml and harness.lock; run `harness-lint update` to refresh the lock"
            );
        }
        match entry.source {
            PackSourceKind::Local => {
                let resolved = pack::resolve_local_pack(root, parsed)?;
                if entry.checksum.is_some() && entry.checksum != resolved.checksum {
                    bail!(
                        "local pack `{id}` differs from harness.lock; run `harness-lint update` if this change is intentional"
                    );
                }
            }
            PackSourceKind::Git => {
                let resolved = pack::restore_git_pack(root, entry)?;
                if entry.checksum.is_some() && entry.checksum != resolved.checksum {
                    bail!("restored pack `{id}` checksum differs from harness.lock");
                }
            }
            _ => bail!("unsupported pack source for `{id}`"),
        }
        restored += 1;
        println!("Restored pack `{id}`.");
    }
    println!("Restored {restored} of {} pack(s).", config.packs.len());
    Ok(())
}

fn run_rule(
    cwd: &PathBuf,
    config_path: Option<&std::path::Path>,
    command: RuleCommand,
    format: ReportFormat,
) -> Result<()> {
    let root = config::find_project_root(cwd)?;
    match command {
        RuleCommand::List => {
            let config = config::load_config(&root, config_path)?;
            let rules = load_rules(&root, &config)?;
            report::print_rules(&rules, format)?;
        }
        RuleCommand::Explain { rule_id } => {
            let config = config::load_config(&root, config_path)?;
            let rules = load_rules(&root, &config)?;
            let rule = rules
                .iter()
                .find(|rule| rule.id == rule_id)
                .ok_or_else(|| anyhow!("rule `{rule_id}` was not found"))?;
            report::print_rule_explain(rule);
        }
        RuleCommand::New {
            id,
            title,
            language,
        } => {
            let config = config::load_config(&root, config_path)?;
            let draft =
                authoring::new_rule(&root, &config.rules.local, &id, &title, language.as_deref())?;
            println!(
                "Created rule draft `{}` at {}",
                draft.id,
                draft.path.display()
            );
        }
        RuleCommand::Draft { feedback } => {
            let config = config::load_config(&root, config_path)?;
            let draft = authoring::suggest_rule(&root, &config.rules.local, &feedback)?;
            println!(
                "Created rule draft `{}` at {}",
                draft.id,
                draft.path.display()
            );
        }
        RuleCommand::Suggest { feedback } => {
            let config = config::load_config(&root, config_path)?;
            let mut query = registry::infer_project_context(&root);
            query.feedback = feedback.clone();
            let candidates = registry::search_registry(&query, config.registry.url.as_deref())?;
            if !candidates.is_empty() {
                println!("Found existing rule candidates:");
                for (index, candidate) in candidates.iter().enumerate() {
                    println!(
                        "{}. {} ({}) score={} pack={}",
                        index + 1,
                        candidate.title,
                        candidate.rule_id,
                        candidate.score,
                        candidate.pack_spec
                    );
                    println!("   {}", candidate.reason);
                }
                let best = &candidates[0];
                println!();
                println!(
                    "To install the best match, run:\n  harness-lint install {} {}",
                    best.pack_id, best.pack_spec
                );
                println!(
                    "To create a local draft instead, run:\n  harness-lint rule draft {:?}",
                    feedback
                );
            } else {
                println!("No existing rule candidates found.");
                println!(
                    "To create a local draft, run:\n  harness-lint rule draft {:?}",
                    feedback
                );
            }
        }
    }
    Ok(())
}

fn select_paths(
    root: &std::path::Path,
    config: &ProjectConfig,
    command: &CheckCommand,
    rules: &[RuleDefinition],
) -> Result<SelectedPaths> {
    let is_implicit_full_scan =
        command.paths.is_empty() && !command.changed && !command.staged && !command.all;
    let raw_paths = if !command.paths.is_empty() {
        command.paths.clone()
    } else if command.staged {
        git::staged_files(root)?
    } else if command.changed {
        let base = command.base.as_deref().unwrap_or(&config.lint.changed_base);
        git::changed_files(root, base)?
    } else {
        paths::discover_all_files(root, &[], &config.rules.local)?
    };
    let grit_paths = paths::filter_paths(
        raw_paths.clone(),
        &config.ignore.paths,
        rules,
        &config.rules.local,
    )?;
    let mut obsidian_paths = paths::filter_paths(
        raw_paths.clone(),
        &config.ignore.paths,
        &[],
        &config.rules.local,
    )?;
    let mut obsidian_extra_roots = config.obsidian.content_roots.clone();
    obsidian_extra_roots.extend(config.obsidian.note_roots.clone());
    if let Some(root) = &config.obsidian.flat_attachment_dir {
        obsidian_extra_roots.push(root.clone());
    }
    if !obsidian_extra_roots.is_empty() {
        let mut extra_paths: Vec<_> = raw_paths
            .into_iter()
            .filter(|path| {
                obsidian_extra_roots
                    .iter()
                    .any(|root| path.starts_with(root))
            })
            .collect();
        obsidian_paths.append(&mut extra_paths);
        obsidian_paths.sort();
        obsidian_paths.dedup();
    }
    if is_implicit_full_scan && grit_paths.len() > 1000 {
        bail!(
            "refusing implicit full scan of {} files; use `harness-lint check --changed`, pass paths, or run `harness-lint check --all` to force it",
            grit_paths.len()
        );
    }
    Ok(SelectedPaths {
        grit: grit_paths,
        obsidian: obsidian_paths,
    })
}

#[derive(Debug)]
struct SelectedPaths {
    grit: Vec<PathBuf>,
    obsidian: Vec<PathBuf>,
}

fn load_rule_packs(
    root: &std::path::Path,
    config: &ProjectConfig,
) -> Result<Vec<crate::model::RulePack>> {
    let mut packs = Vec::new();
    for (id, spec) in &config.packs {
        let spec = pack::parse_pack_spec(id, spec);
        match spec.source {
            PackSourceKind::Local => {
                let resolved = pack::resolve_local_pack(root, spec)?;
                packs.push(pack::load_rule_pack(&resolved)?);
            }
            PackSourceKind::Git => {
                let lock = config::load_lock(root)?;
                let entry = lock.packs.get(id).ok_or_else(|| {
                    anyhow!("pack `{id}` is not installed; run `harness-lint update`")
                })?;
                let local_path = if entry.local_path.is_absolute() {
                    entry.local_path.clone()
                } else {
                    root.join(&entry.local_path)
                };
                let resolved = crate::model::ResolvedPack {
                    spec,
                    local_path,
                    pack_path: entry.pack_path.clone(),
                    version: entry.version.clone(),
                    checksum: entry.checksum.clone(),
                };
                packs.push(pack::load_rule_pack(&resolved)?);
            }
            _ => {
                bail!("unsupported pack source for `{}`", spec.id);
            }
        }
    }
    packs.push(pack::load_local_rules_pack(root, &config.rules.local)?);
    Ok(packs)
}

fn load_rules(root: &std::path::Path, config: &ProjectConfig) -> Result<Vec<RuleDefinition>> {
    let mut rules = Vec::new();
    for pack in load_rule_packs(root, config)? {
        rules.extend(pack.rules);
    }
    rules.sort_by(|left, right| left.id.cmp(&right.id));
    Ok(rules)
}

fn collect_effective_rules(
    packs: &[crate::model::RulePack],
    config: &ProjectConfig,
    command: &CheckCommand,
) -> Vec<RuleDefinition> {
    let mut rules = Vec::new();
    for pack in packs {
        for mut rule in pack.rules.clone() {
            if rule.status == crate::model::RuleStatus::Draft {
                continue;
            }
            if config.disabled.rules.iter().any(|id| id == &rule.id) {
                continue;
            }
            if !command.rule.is_empty() && !command.rule.iter().any(|id| id == &rule.id) {
                continue;
            }
            if !command.tag.is_empty()
                && !command
                    .tag
                    .iter()
                    .any(|tag| rule.tags.iter().any(|rule_tag| rule_tag == tag))
            {
                continue;
            }
            if let Some(level) = config.overrides.get(&rule.id) {
                rule.level = *level;
            }
            rules.push(rule);
        }
    }
    rules
}
