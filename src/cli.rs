use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use anyhow::{Context, Result, anyhow, bail};
use clap::{Args, Parser, Subcommand};

use crate::authoring;
use crate::cache;
use crate::compiler;
use crate::config::{self, ProjectConfig};
use crate::config_health;
use crate::exceptions;
use crate::git;
use crate::grit;
use crate::init;
use crate::model::{PackSourceKind, RuleBody, RuleDefinition, RuleExampleKind};
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
    #[command(about = "Show recent feature highlights and when to adopt them")]
    Whatsnew,
    #[command(about = "Run active rules against the configured project file set")]
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
    #[command(about = "Create a local rule from feedback and executable GritQL")]
    Create {
        feedback: String,
        #[arg(long)]
        language: String,
        #[arg(long)]
        grit: String,
    },
    #[command(about = "Verify that rule Bad examples trigger their GritQL")]
    Verify { rule_id: Option<String> },
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
        Command::Whatsnew => {
            run_whatsnew();
            Ok(())
        }
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

fn run_whatsnew() {
    println!("harness-lint — what's new\n");
    println!("0.5.x  Agent plugins (Claude Code & Codex)");
    println!(
        "  Inject Lint Driven Development guidance and live `check --changed`\n  \
         diagnostics through SessionStart/UserPromptSubmit hooks instead of a\n  \
         static AGENTS.md block, plus a `/harness-lint-capture` command that\n  \
         turns a session's feedback into rules."
    );
    println!(
        "  Adopt when: agents keep ignoring the harness-lint guidance you put in\n  \
         AGENTS.md, or you want violations surfaced before the agent writes code."
    );
    println!("  Install: see plugins/ — `/plugin install harness-lint@harness-lint`.");
    println!();
    println!("0.4.x  File sets & runs_on");
    println!(
        "  Scope a rule to a named region, or reach a default-closed region such as\n  \
         committed generated code, with `runs_on` + `[file_sets.*]`."
    );
    println!(
        "  Adopt when: two or more rules share a directory region, or a rule must scan\n  \
         generated code that ordinary rules should skip."
    );
    println!(
        "  Keep `$filename` for: single-file scope, or \"a region minus a few files\"\n  \
         (runs_on is include-only and cannot express exclusions)."
    );
    println!();
    println!(
        "Full guide (with per-feature adoption advice): {}",
        config::WHATS_NEW_URL
    );
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
    exceptions::validate_exceptions(&config.exceptions)?;

    // Repository-level integrity checks for harness-lint's own configuration.
    // These run independently of GritQL (even when no source paths are
    // selected) and honour the same `--rule`/`--tag` filtering.
    let config_diagnostics: Vec<_> = config_health::check_config_paths(&root, &config)
        .into_iter()
        .filter(|diagnostic| config_check_selected(&command, &diagnostic.rule_id))
        .collect();

    let packs = load_rule_packs(&root, &config)?;
    let known_rule_ids: BTreeSet<&str> = packs
        .iter()
        .flat_map(|pack| {
            pack.rules
                .iter()
                .map(|rule| rule.id.as_str())
                .chain(pack.default_disabled.iter().map(|id| id.as_str()))
        })
        .collect();
    let ref_diagnostics: Vec<_> = config_health::check_unknown_rule_refs(&config, &known_rule_ids)
        .into_iter()
        .filter(|diagnostic| config_check_selected(&command, &diagnostic.rule_id))
        .collect();
    let all_rules: Vec<&RuleDefinition> = packs.iter().flat_map(|pack| pack.rules.iter()).collect();
    let run_target_diagnostics: Vec<_> = config_health::check_run_targets(&config, &all_rules)
        .into_iter()
        .filter(|diagnostic| config_check_selected(&command, &diagnostic.rule_id))
        .collect();
    let runs_on_filename_diagnostics: Vec<_> =
        config_health::check_runs_on_filename(&config, &all_rules)
            .into_iter()
            .filter(|diagnostic| config_check_selected(&command, &diagnostic.rule_id))
            .collect();
    let file_set_index = paths::FileSetIndex::build(&config)?;
    let active_rules = collect_effective_rules(&packs, &config, &command);
    let selected_paths = select_paths(&root, &config, &command, &active_rules, &file_set_index)?;
    if verbose {
        eprintln!(
            "harness-lint: {} active rule(s), {} GritQL path(s)",
            active_rules.len(),
            selected_paths.grit.len()
        );
    }
    let grit_diagnostics = if active_rules.is_empty() || selected_paths.grit.is_empty() {
        Vec::new()
    } else {
        let _lock = acquire_grit_run_lock(&root)?;
        run_grit_checks(
            &root,
            &active_rules,
            &selected_paths.grit,
            &config,
            &command,
            &file_set_index,
            verbose,
        )?
    };
    let exception_outcome =
        exceptions::apply_diagnostic_exceptions(grit_diagnostics, &config.exceptions)?;
    if verbose && exception_outcome.hidden_count > 0 {
        eprintln!(
            "harness-lint: hid {} result(s) by rule exception configuration",
            exception_outcome.hidden_count
        );
    }
    let mut diagnostics = exception_outcome.diagnostics;
    diagnostics.extend(config_diagnostics);
    diagnostics.extend(ref_diagnostics);
    diagnostics.extend(run_target_diagnostics);
    diagnostics.extend(runs_on_filename_diagnostics);
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

/// Whether a synthetic configuration diagnostic survives the active
/// `--rule`/`--tag` selection. Config checks carry no tags, so any `--tag`
/// filter excludes them; a `--rule` filter keeps only matching rule ids.
fn config_check_selected(command: &CheckCommand, rule_id: &str) -> bool {
    if !command.tag.is_empty() {
        return false;
    }
    command.rule.is_empty() || command.rule.iter().any(|rule| rule == rule_id)
}

fn run_grit_checks(
    root: &Path,
    active_rules: &[RuleDefinition],
    selected_paths: &[PathBuf],
    config: &ProjectConfig,
    command: &CheckCommand,
    file_set_index: &paths::FileSetIndex,
    verbose: bool,
) -> Result<Vec<crate::model::Diagnostic>> {
    let mut diagnostics = Vec::new();
    for rules in group_rules_by_language(active_rules) {
        let paths: Vec<_> = selected_paths
            .iter()
            .filter(|path| {
                rules
                    .iter()
                    .any(|rule| paths::rule_scans_path(rule, path.as_path(), file_set_index))
            })
            .cloned()
            .collect();
        if paths.is_empty() {
            continue;
        }
        let compiled = compiler::compile_rule_set(root, rules)?;
        if config.lint.cache {
            let rule_fingerprint = cache::fingerprint(&format!("{:?}", compiled.grit_rules));
            let config_fingerprint = cache::fingerprint(&format!(
                "{:?}{:?}{:?}{:?}{:?}{:?}",
                command.rule,
                command.tag,
                config.overrides,
                config.disabled,
                config.ignore,
                config.file_sets
            ));
            diagnostics.extend(run_cached_check(
                root,
                &compiled,
                &paths,
                &rule_fingerprint,
                &config_fingerprint,
                verbose,
            )?);
        } else {
            diagnostics.extend(run_uncached_check(root, &compiled, &paths)?);
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

fn group_rules_by_language(active_rules: &[RuleDefinition]) -> Vec<Vec<RuleDefinition>> {
    let mut groups: BTreeMap<Option<String>, Vec<RuleDefinition>> = BTreeMap::new();
    for rule in active_rules {
        groups
            .entry(
                rule.language
                    .as_ref()
                    .map(|language| language.trim().to_ascii_lowercase()),
            )
            .or_default()
            .push(rule.clone());
    }
    groups.into_values().collect()
}

#[derive(Debug)]
struct GritRunLock {
    path: PathBuf,
}

impl Drop for GritRunLock {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}

fn acquire_grit_run_lock(root: &Path) -> Result<GritRunLock> {
    let lock_dir = root.join(".harness");
    fs::create_dir_all(&lock_dir)
        .with_context(|| format!("failed to create {}", lock_dir.display()))?;
    let path = lock_dir.join("grit-run.lock");
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
    loop {
        match fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
        {
            Ok(mut file) => {
                writeln!(file, "pid={}", std::process::id())
                    .with_context(|| format!("failed to write {}", path.display()))?;
                return Ok(GritRunLock { path });
            }
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
                if std::time::Instant::now() >= deadline {
                    bail!(
                        "timed out waiting for another harness-lint check to release {}; remove the lock if no harness-lint process is running",
                        path.display()
                    );
                }
                std::thread::sleep(std::time::Duration::from_millis(100));
            }
            Err(error) => {
                return Err(error).with_context(|| format!("failed to acquire {}", path.display()));
            }
        }
    }
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

    match exceptions::validate_exceptions(&config.exceptions) {
        Ok(()) if !config.exceptions.is_empty() => findings.push(doctor_ok(
            "exceptions",
            format!("validated {} rule exception(s)", config.exceptions.len()),
        )),
        Ok(()) => {}
        Err(error) => findings.push(doctor_error("exceptions", error.to_string())),
    }

    let stale_paths = config_health::check_config_paths(&root, &config);
    if stale_paths.is_empty() {
        findings.push(doctor_ok(
            "config-paths",
            "all exception, ignore, and file-set paths exist".to_string(),
        ));
    } else {
        for diagnostic in stale_paths {
            if diagnostic.level.is_failing() {
                findings.push(doctor_error("config-paths", diagnostic.message));
            } else {
                findings.push(doctor_warn("config-paths", diagnostic.message));
            }
        }
    }

    if !config.file_sets.is_empty() {
        findings.push(doctor_ok(
            "file-sets",
            format!(
                "defined {} file set(s): {}",
                config.file_sets.len(),
                config
                    .file_sets
                    .keys()
                    .cloned()
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
        ));
    }

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
    match validate_local_rules(&root, &config) {
        Ok(validation) => {
            findings.push(doctor_ok(
                "local-rules",
                format!(
                    "validated {} local rule(s), all with executable GritQL",
                    validation.rule_count
                ),
            ));
        }
        Err(error) => findings.push(doctor_error("local-rules", format!("{error:#}"))),
    }

    match load_rule_packs(&root, &config) {
        Ok(packs) => {
            let rule_count: usize = packs.iter().map(|pack| pack.rules.len()).sum();
            findings.push(doctor_ok(
                "rules",
                format!("loaded {} pack(s) with {} rule(s)", packs.len(), rule_count),
            ));
            let rule_ids: BTreeSet<&str> = packs
                .iter()
                .flat_map(|pack| {
                    pack.rules
                        .iter()
                        .map(|rule| rule.id.as_str())
                        .chain(pack.default_disabled.iter().map(|id| id.as_str()))
                })
                .collect();
            for exception in &config.exceptions {
                if !rule_ids.contains(exception.rule.as_str()) {
                    findings.push(doctor_warn(
                        "exceptions",
                        format!(
                            "rule exception references unknown rule `{}`",
                            exception.rule
                        ),
                    ));
                }
            }
            for diagnostic in config_health::check_unknown_rule_refs(&config, &rule_ids) {
                findings.push(doctor_warn("config-refs", diagnostic.message));
            }
            let all_rules: Vec<&RuleDefinition> =
                packs.iter().flat_map(|pack| pack.rules.iter()).collect();
            let run_target_issues = config_health::check_run_targets(&config, &all_rules);
            if run_target_issues.is_empty() {
                findings.push(doctor_ok(
                    "run-targets",
                    "every rule `runs_on` a defined file set or provided concept".to_string(),
                ));
            } else {
                for diagnostic in run_target_issues {
                    findings.push(doctor_error("run-targets", diagnostic.message));
                }
            }
            let scope_conflicts = config_health::check_runs_on_filename(&config, &all_rules);
            if scope_conflicts.is_empty() {
                findings.push(doctor_ok(
                    "runs-on-scope",
                    "no rule has a `runs_on` region disjoint from its `$filename` scope"
                        .to_string(),
                ));
            } else {
                for diagnostic in scope_conflicts {
                    findings.push(doctor_warn("runs-on-scope", diagnostic.message));
                }
            }
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

#[derive(Debug)]
struct LocalRuleValidation {
    rule_count: usize,
}

fn validate_local_rules(root: &Path, config: &ProjectConfig) -> Result<LocalRuleValidation> {
    let mut rules = Vec::new();
    for dir in &config.rules.local {
        let path = if dir.is_absolute() {
            dir.clone()
        } else {
            root.join(dir)
        };
        if !path.exists() {
            continue;
        }
        let discovered = crate::rule::discover_rules(&path, None)
            .with_context(|| format!("invalid local rules in {}", path.display()))?;
        rules.extend(discovered);
    }
    let rule_count = rules.len();
    let executable_count = rules
        .iter()
        .filter(|rule| matches!(rule.body, RuleBody::Grit(_)))
        .count();
    if rule_count != executable_count {
        bail!(
            "validated {rule_count} local rule(s), but only {executable_count} include executable GritQL"
        );
    }
    validate_local_gritql(root, rules)
        .context("local rule GritQL failed validation during doctor")?;
    Ok(LocalRuleValidation { rule_count })
}

fn validate_local_gritql(root: &Path, rules: Vec<RuleDefinition>) -> Result<()> {
    let compiled = compiler::compile_rule_set(root, rules)?;
    if compiled.grit_rules.is_empty() {
        return Ok(());
    }

    let probe_path = std::env::temp_dir().join(format!(
        "harness-lint-doctor-probe-{}.ts",
        std::process::id()
    ));
    fs::write(&probe_path, "const harnessLintDoctorProbe = 1;\n")
        .with_context(|| format!("failed to write {}", probe_path.display()))?;
    let result = grit::run_grit(root, &compiled, std::slice::from_ref(&probe_path)).map(|_| ());
    let _ = fs::remove_file(&probe_path);
    if let Err(error) = result {
        bail!("{}", local_gritql_validation_message(&compiled, &error));
    }
    Ok(())
}

fn local_gritql_validation_message(
    compiled: &crate::model::CompiledRules,
    error: &anyhow::Error,
) -> String {
    let message = format!("{error:#}");
    let Some(pattern_name) = extract_grit_pattern_name(&message) else {
        return message;
    };
    let Some(rule) = compiled
        .grit_rules
        .iter()
        .find(|rule| compiler::safe_pattern_filename(&rule.id) == pattern_name)
    else {
        return message;
    };

    format!(
        "{message}\n\nGrit pattern `{pattern_name}` was generated from rule `{}` ({}) at {}",
        rule.id,
        rule.title,
        rule.source_path.display()
    )
}

fn extract_grit_pattern_name(message: &str) -> Option<String> {
    for marker in [
        "Unable to compile pattern ",
        "pattern definition not found: ",
    ] {
        let Some((_, rest)) = message.split_once(marker) else {
            continue;
        };
        let pattern: String = rest
            .chars()
            .take_while(|ch| ch.is_ascii_alphanumeric() || *ch == '_' || *ch == '-')
            .collect();
        if !pattern.is_empty() {
            return Some(pattern);
        }
    }
    None
}

fn run_cached_check(
    root: &Path,
    compiled: &crate::model::CompiledRules,
    paths: &[PathBuf],
    rule_fingerprint: &str,
    config_fingerprint: &str,
    verbose: bool,
) -> Result<Vec<crate::model::Diagnostic>> {
    let normalizer = DiagnosticNormalizer::new(root, compiled);
    let mut diagnostics = Vec::new();
    let mut misses = Vec::new();
    let mut miss_keys = BTreeMap::new();

    for path in paths {
        let file_hash = cache::file_hash(root, path)?;
        let key = cache::file_cache_key(path, &file_hash, rule_fingerprint, config_fingerprint);
        if let Some(cached) = cache::load_file(root, &key)? {
            diagnostics.extend(normalizer.normalize(cached));
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
        let fresh = normalizer.normalize(grit::run_grit(root, compiled, &batch_paths)?);
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
    let normalizer = DiagnosticNormalizer::new(root, compiled);
    let mut diagnostics = Vec::new();
    for batch in paths.chunks(GRIT_BATCH_SIZE) {
        diagnostics.extend(normalizer.normalize(grit::run_grit(root, compiled, batch)?));
    }
    Ok(diagnostics)
}

struct DiagnosticNormalizer<'a> {
    root: &'a Path,
    rule_ids_by_grit_name: BTreeMap<String, String>,
}

impl<'a> DiagnosticNormalizer<'a> {
    fn new(root: &'a Path, compiled: &crate::model::CompiledRules) -> Self {
        let rule_ids_by_grit_name = compiled
            .grit_rules
            .iter()
            .map(|rule| (compiler::safe_pattern_filename(&rule.id), rule.id.clone()))
            .collect();
        Self {
            root,
            rule_ids_by_grit_name,
        }
    }

    fn normalize(
        &self,
        diagnostics: Vec<crate::model::Diagnostic>,
    ) -> Vec<crate::model::Diagnostic> {
        diagnostics
            .into_iter()
            .map(|mut diagnostic| {
                diagnostic.path = cache::normalize_path(self.root, &diagnostic.path);
                if let Some(rule_id) = self.rule_ids_by_grit_name.get(&diagnostic.rule_id) {
                    diagnostic.rule_id = rule_id.clone();
                }
                diagnostic
            })
            .collect()
    }
}

fn normalize_diagnostics(
    root: &Path,
    compiled: &crate::model::CompiledRules,
    diagnostics: Vec<crate::model::Diagnostic>,
) -> Vec<crate::model::Diagnostic> {
    DiagnosticNormalizer::new(root, compiled).normalize(diagnostics)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{CompiledRules, Diagnostic, RuleBody, RuleDefinition, Severity};

    #[test]
    fn normalize_diagnostics_reports_canonical_rule_ids() {
        let tempdir = tempfile::tempdir().unwrap();
        let root = tempdir.path();
        let compiled = CompiledRules {
            grit_dir: root.join(".harness/generated/.grit"),
            grit_rules: vec![RuleDefinition {
                id: "local.playwright-no-inflated-assertion-timeouts".to_string(),
                title: "Avoid inflated Playwright assertion timeouts".to_string(),
                language: Some("typescript".to_string()),
                level: Severity::Warn,
                skill: None,
                tags: vec![],
                runs_on: vec![],
                description: String::new(),
                body: RuleBody::Grit(String::new()),
                examples: vec![],
                source_path: root.join("Rules/rule.md"),
                pack_id: Some("local".to_string()),
            }],
        };
        let diagnostics = normalize_diagnostics(
            root,
            &compiled,
            vec![Diagnostic {
                rule_id: "local_playwright_no_inflated_assertion_timeouts".to_string(),
                level: Severity::Warn,
                message: "message".to_string(),
                path: root.join("apps/website/e2e/user-preferences-display.spec.ts"),
                start_line: 14,
                start_column: 5,
                end_line: None,
                end_column: None,
                fix_available: false,
            }],
        );

        assert_eq!(
            diagnostics[0].rule_id,
            "local.playwright-no-inflated-assertion-timeouts"
        );
        assert_eq!(
            diagnostics[0].path,
            PathBuf::from("apps/website/e2e/user-preferences-display.spec.ts")
        );
    }

    #[test]
    fn local_gritql_validation_message_reports_source_rule() {
        let root = tempfile::tempdir().unwrap();
        let source_path = root.path().join("Rules/playwright-ready.md");
        let compiled = CompiledRules {
            grit_dir: root.path().join(".harness/generated/.grit"),
            grit_rules: vec![RuleDefinition {
                id: "local.playwright-page-ready-gate".to_string(),
                title: "Wait for page ready gate".to_string(),
                language: Some("typescript".to_string()),
                level: Severity::Warn,
                skill: None,
                tags: vec![],
                runs_on: vec![],
                description: String::new(),
                body: RuleBody::Grit("language js\n`page.goto($url)`".to_string()),
                examples: vec![],
                source_path: source_path.clone(),
                pack_id: Some("local".to_string()),
            }],
        };
        let error = anyhow!(
            "`grit check` failed: Error: Unable to compile pattern local_playwright_page_ready_gate:\npattern definition not found: local_playwright_page_ready_gate. Try running grit init."
        );

        let message = local_gritql_validation_message(&compiled, &error);

        assert!(message.contains("rule `local.playwright-page-ready-gate`"));
        assert!(message.contains("Wait for page ready gate"));
        assert!(message.contains(&source_path.display().to_string()));
    }

    #[test]
    fn local_rule_validation_rejects_non_executable_grit_block() {
        let root = tempfile::tempdir().unwrap();
        let rules_dir = root.path().join("Rules");
        fs::create_dir_all(&rules_dir).unwrap();
        let rule_path = rules_dir.join("draft.md");
        fs::write(
            &rule_path,
            r#"---
id: local.draft-rule
title: Draft Rule
---

# Draft Rule

```grit
// TODO: add a real pattern.
```
"#,
        )
        .unwrap();
        let mut config = ProjectConfig::default();
        config.rules.local = vec![PathBuf::from("Rules")];

        let error = validate_local_rules(root.path(), &config)
            .unwrap_err()
            .to_string();
        let expanded_error = format!(
            "{:#}",
            validate_local_rules(root.path(), &config).unwrap_err()
        );
        assert!(error.contains("invalid local rules"));
        assert!(expanded_error.contains(&rule_path.display().to_string()));
        assert!(expanded_error.contains("has a ```grit block but no executable GritQL"));
    }
}

fn run_catalog(
    cwd: &PathBuf,
    config_path: Option<&std::path::Path>,
    command: CatalogCommand,
    _format: ReportFormat,
) -> Result<()> {
    let root = config::find_project_root(cwd)?;
    let _pack_operation_lock = if requires_pack_operation_lock(&command) {
        Some(acquire_pack_operation_lock(&root)?)
    } else {
        None
    };
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

fn requires_pack_operation_lock(command: &CatalogCommand) -> bool {
    matches!(
        command,
        CatalogCommand::Install { .. }
            | CatalogCommand::Update
            | CatalogCommand::Restore
            | CatalogCommand::Outdated
            | CatalogCommand::Remove { .. }
    )
}

#[derive(Debug)]
struct PackOperationLock {
    path: PathBuf,
}

impl Drop for PackOperationLock {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}

fn acquire_pack_operation_lock(root: &Path) -> Result<PackOperationLock> {
    let lock_dir = root.join(config::WORK_DIR);
    fs::create_dir_all(&lock_dir)
        .with_context(|| format!("failed to create {}", lock_dir.display()))?;
    let path = lock_dir.join("pack-operation.lock");
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
    let mut reported_wait = false;
    loop {
        match fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
        {
            Ok(mut file) => {
                writeln!(file, "pid={}", std::process::id())
                    .with_context(|| format!("failed to write {}", path.display()))?;
                return Ok(PackOperationLock { path });
            }
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
                if !reported_wait {
                    eprintln!(
                        "harness-lint: waiting for another pack operation to release {}",
                        path.display()
                    );
                    reported_wait = true;
                }
                if std::time::Instant::now() >= deadline {
                    bail!(
                        "timed out waiting for another harness-lint pack operation to release {}; remove the lock if no harness-lint process is running",
                        path.display()
                    );
                }
                std::thread::sleep(std::time::Duration::from_millis(100));
            }
            Err(error) => {
                return Err(error).with_context(|| format!("failed to acquire {}", path.display()));
            }
        }
    }
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
            let packs = load_rule_packs(&root, &config)?;
            report::print_rule_packs(&packs, format)?;
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
        RuleCommand::Create {
            feedback,
            language,
            grit,
        } => {
            let config = config::load_config(&root, config_path)?;
            let created =
                authoring::create_rule(&root, &config.rules.local, &feedback, &language, &grit)?;
            println!(
                "Created rule `{}` at {}",
                created.id,
                created.path.display()
            );
        }
        RuleCommand::Verify { rule_id } => {
            let config = config::load_config(&root, config_path)?;
            let rules = load_rules(&root, &config)?;
            let selected_rules: Vec<_> = rules
                .into_iter()
                .filter(|rule| rule_id.as_ref().is_none_or(|id| &rule.id == id))
                .collect();
            if selected_rules.is_empty() {
                if let Some(rule_id) = rule_id {
                    bail!("rule `{rule_id}` was not found");
                }
                bail!("no rules found");
            }
            let verification = verify_rule_examples(&selected_rules)?;
            match format {
                ReportFormat::Json => {
                    println!(
                        "{}",
                        serde_json::json!({
                            "rules": selected_rules.len(),
                            "bad_examples": verification.bad_examples,
                            "good_examples": verification.good_examples
                        })
                    );
                }
                ReportFormat::Human => {
                    println!(
                        "Verified {} rule(s), {} Bad example(s), {} Good example(s).",
                        selected_rules.len(),
                        verification.bad_examples,
                        verification.good_examples
                    );
                }
            }
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
                    "To create a local rule instead, first confirm the feedback can be expressed as GritQL, then run:\n  harness-lint rule create {:?} --language <language> --grit <gritql>",
                    feedback
                );
            } else {
                println!("No existing rule candidates found.");
                println!(
                    "To create a local rule, first confirm the feedback can be expressed as GritQL, then run:\n  harness-lint rule create {:?} --language <language> --grit <gritql>",
                    feedback
                );
            }
        }
    }
    Ok(())
}

#[derive(Debug, Default)]
struct RuleVerificationSummary {
    bad_examples: usize,
    good_examples: usize,
}

fn verify_rule_examples(rules: &[RuleDefinition]) -> Result<RuleVerificationSummary> {
    let mut summary = RuleVerificationSummary::default();
    for rule in rules {
        let bad_examples: Vec<_> = rule
            .examples
            .iter()
            .filter(|example| example.kind == RuleExampleKind::Bad)
            .collect();
        let good_examples: Vec<_> = rule
            .examples
            .iter()
            .filter(|example| example.kind == RuleExampleKind::Good)
            .collect();
        if bad_examples.is_empty() {
            bail!(
                "rule `{}` has no Bad examples; add one before verification",
                rule.id
            );
        }
        let scratch = crate::scratch::ScratchDir::new("harness-lint-rule-verify")?;
        let compiled = compiler::compile_rule_set(scratch.path(), vec![rule.clone()])?;
        for (index, example) in bad_examples.iter().enumerate() {
            verify_rule_example(
                rule,
                &compiled,
                scratch.path(),
                RuleExampleKind::Bad,
                index,
                example,
            )?;
            summary.bad_examples += 1;
        }
        for (index, example) in good_examples.iter().enumerate() {
            verify_rule_example(
                rule,
                &compiled,
                scratch.path(),
                RuleExampleKind::Good,
                index,
                example,
            )?;
            summary.good_examples += 1;
        }
    }
    Ok(summary)
}

fn verify_rule_example(
    rule: &RuleDefinition,
    compiled: &crate::model::CompiledRules,
    scratch_root: &Path,
    kind: RuleExampleKind,
    index: usize,
    example: &crate::model::RuleExample,
) -> Result<()> {
    if !has_concrete_example(&example.code) {
        bail!(
            "{} example {} for rule `{}` is empty or TODO-only",
            example_kind_label(kind),
            index + 1,
            rule.id
        );
    }
    let language = example
        .language
        .as_deref()
        .or(rule.language.as_deref())
        .unwrap_or("text");
    let relative_path = example_verify_path(kind, index, language);
    let source_path = scratch_root.join(&relative_path);
    if let Some(parent) = source_path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }
    fs::write(&source_path, &example.code)
        .with_context(|| format!("failed to write {}", source_path.display()))?;
    let diagnostics = normalize_diagnostics(
        scratch_root,
        compiled,
        grit::run_grit(scratch_root, compiled, &[relative_path])?,
    );
    match kind {
        RuleExampleKind::Bad if diagnostics.is_empty() => {
            bail!(
                "Bad example {} for rule `{}` did not trigger; adjust the GritQL, Bad example, or `$filename` scope",
                index + 1,
                rule.id
            );
        }
        RuleExampleKind::Good if !diagnostics.is_empty() => {
            bail!(
                "Good example {} for rule `{}` triggered {} diagnostic(s); narrow the GritQL or replace the Good example",
                index + 1,
                rule.id,
                diagnostics.len()
            );
        }
        _ => {}
    }
    Ok(())
}

fn example_verify_path(kind: RuleExampleKind, index: usize, language: &str) -> PathBuf {
    let stem = match (kind, index) {
        (RuleExampleKind::Bad, 0) => "bad-example".to_string(),
        (RuleExampleKind::Bad, _) => format!("bad-example-{}", index + 1),
        (RuleExampleKind::Good, 0) => "good-example".to_string(),
        (RuleExampleKind::Good, _) => format!("good-example-{}", index + 1),
    };
    PathBuf::from("src").join(format!("{}.{}", stem, grit::sample_extension(language)))
}

fn example_kind_label(kind: RuleExampleKind) -> &'static str {
    match kind {
        RuleExampleKind::Bad => "Bad",
        RuleExampleKind::Good => "Good",
    }
}

fn has_concrete_example(code: &str) -> bool {
    code.lines().any(|line| {
        let trimmed = line.trim();
        !trimmed.is_empty() && !trimmed.to_ascii_uppercase().contains("TODO")
    })
}

fn select_paths(
    root: &std::path::Path,
    config: &ProjectConfig,
    command: &CheckCommand,
    rules: &[RuleDefinition],
    file_set_index: &paths::FileSetIndex,
) -> Result<SelectedPaths> {
    let is_implicit_full_scan = !command.changed && !command.staged && !command.all;
    let raw_paths = if command.staged {
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
        file_set_index,
    )?;
    if is_implicit_full_scan && grit_paths.len() > 1000 {
        bail!(
            "refusing implicit full scan of {} files; use `harness-lint check --changed` or run `harness-lint check --all` to force it",
            grit_paths.len()
        );
    }
    Ok(SelectedPaths { grit: grit_paths })
}

#[derive(Debug)]
struct SelectedPaths {
    grit: Vec<PathBuf>,
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
    packs.extend(pack::load_local_rule_packs(root, &config.rules.local)?);
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
