use std::path::PathBuf;
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
use crate::pack;
use crate::paths;
use crate::registry;
use crate::report::{self, ReportFormat};

#[derive(Debug, Parser)]
#[command(name = "harness-lint")]
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
    Init(InitCommand),
    Check(CheckCommand),
    Pack {
        #[command(subcommand)]
        command: PackCommand,
    },
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

#[derive(Debug, Subcommand)]
enum PackCommand {
    Add { id: String, spec: String },
    Update,
    List,
}

#[derive(Debug, Subcommand)]
enum RuleCommand {
    List,
    Explain {
        rule_id: String,
    },
    New {
        id: String,
        title: String,
        #[arg(long)]
        language: Option<String>,
    },
    Suggest {
        feedback: String,
        #[arg(long)]
        local: bool,
    },
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
        Command::Check(command) => {
            run_check(&cwd, cli.config.as_deref(), command, format, cli.verbose)
        }
        Command::Pack { command } => run_pack(&cwd, cli.config.as_deref(), command, format),
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
    let paths = select_paths(&root, &config, &command, &active_rules)?;
    if verbose {
        eprintln!(
            "harness-lint: {} active rule(s), {} selected path(s)",
            active_rules.len(),
            paths.len()
        );
    }
    if paths.is_empty() || active_rules.is_empty() {
        report::report_diagnostics(&[], format)?;
        return Ok(());
    }
    let compiled =
        compiler::compile_grit_rules(&root, packs, &config.overrides, &config.disabled.rules)?;
    let diagnostics = if config.lint.cache {
        let key = cache::cache_key(
            &root,
            &paths,
            &cache::fingerprint(&format!("{:?}", compiled.grit_rules)),
            &cache::fingerprint(&format!(
                "{:?}{:?}{:?}",
                config.overrides, config.disabled, config.ignore
            )),
        )?;
        if let Some(cached) = cache::load(&root, &key)? {
            cached
        } else {
            let diagnostics = grit::run_grit(&root, &compiled, &paths)?;
            cache::store(&root, &key, diagnostics.clone())?;
            diagnostics
        }
    } else {
        grit::run_grit(&root, &compiled, &paths)?
    };
    report::report_diagnostics(&diagnostics, format)?;
    if diagnostics
        .iter()
        .any(|diagnostic| diagnostic.level.is_failing())
    {
        bail!("harness-lint found error-level diagnostics");
    }
    Ok(())
}

fn run_pack(
    cwd: &PathBuf,
    config_path: Option<&std::path::Path>,
    command: PackCommand,
    _format: ReportFormat,
) -> Result<()> {
    let root = config::find_project_root(cwd)?;
    match command {
        PackCommand::Add { id, spec } => {
            let mut config = config::load_config(&root, config_path)?;
            let mut lock = config::load_lock(&root)?;
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
        PackCommand::Update => {
            let config = config::load_config(&root, config_path)?;
            let mut lock = config::load_lock(&root)?;
            for (id, spec) in &config.packs {
                let parsed = pack::parse_pack_spec(id, spec);
                match parsed.source {
                    PackSourceKind::Local => {
                        let resolved = pack::resolve_local_pack(&root, parsed)?;
                        lock.packs
                            .insert(id.clone(), pack::lock_entry(&resolved, &root));
                    }
                    PackSourceKind::Git => {
                        let resolved = if let Some(entry) = lock.packs.get(id) {
                            pack::update_git_pack(&root, entry)?
                        } else {
                            pack::install_git_pack(&root, parsed)?
                        };
                        lock.packs
                            .insert(id.clone(), pack::lock_entry(&resolved, &root));
                    }
                    _ => bail!("unsupported pack source for `{id}`"),
                }
            }
            config::write_lock(&root, &lock)?;
            println!("Updated {} pack(s).", config.packs.len());
        }
        PackCommand::List => {
            let config = config::load_config(&root, config_path)?;
            if config.packs.is_empty() {
                println!("No external packs configured.");
            }
            for (id, spec) in config.packs {
                println!("{id}\t{spec}");
            }
        }
    }
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
            let draft = authoring::new_rule(&root, &id, &title, language.as_deref())?;
            println!(
                "Created rule draft `{}` at {}",
                draft.id,
                draft.path.display()
            );
        }
        RuleCommand::Suggest { feedback, local } => {
            let config = config::load_config(&root, config_path)?;
            if !local {
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
                        "To install the best match, run:\n  harness-lint pack add {} {}",
                        best.pack_id, best.pack_spec
                    );
                    println!(
                        "To create a local draft instead, run:\n  harness-lint rule suggest --local {:?}",
                        feedback
                    );
                    return Ok(());
                }
            }
            let draft = authoring::suggest_rule(&root, &feedback)?;
            println!(
                "Created rule draft `{}` at {}",
                draft.id,
                draft.path.display()
            );
        }
    }
    Ok(())
}

fn select_paths(
    root: &std::path::Path,
    config: &ProjectConfig,
    command: &CheckCommand,
    rules: &[RuleDefinition],
) -> Result<Vec<PathBuf>> {
    let is_implicit_full_scan =
        command.paths.is_empty() && !command.changed && !command.staged && !command.all;
    let paths = if !command.paths.is_empty() {
        command.paths.clone()
    } else if command.staged {
        git::staged_files(root)?
    } else if command.changed {
        let base = command.base.as_deref().unwrap_or(&config.lint.changed_base);
        git::changed_files(root, base)?
    } else {
        paths::discover_all_files(root, &config.ignore.paths)?
    };
    let paths = paths::filter_paths(paths, &config.ignore.paths, rules)?;
    if is_implicit_full_scan && paths.len() > 1000 {
        bail!(
            "refusing implicit full scan of {} files; use `harness-lint check --changed`, pass paths, or run `harness-lint check --all` to force it",
            paths.len()
        );
    }
    Ok(paths)
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
                    anyhow!("pack `{id}` is not installed; run `harness pack update`")
                })?;
                let local_path = if entry.local_path.is_absolute() {
                    entry.local_path.clone()
                } else {
                    root.join(&entry.local_path)
                };
                let resolved = crate::model::ResolvedPack {
                    spec,
                    local_path,
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
