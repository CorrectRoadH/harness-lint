use std::path::PathBuf;

use anyhow::Result;

use crate::model::{
    CompiledRules, Diagnostic, FixResult, LockEntry, PackSpec, ProjectContext, RegistryCandidate,
    RegistryQuery, ResolvedPack, RuleDraft, RulePack,
};

pub trait RuleSource {
    fn resolve(&self, spec: PackSpec) -> Result<ResolvedPack>;
    fn update(&self, lock: LockEntry) -> Result<ResolvedPack>;
}

pub trait RuleCompiler {
    fn compile(&self, packs: Vec<RulePack>) -> Result<CompiledRules>;
}

pub trait RuleEngine {
    fn check(&self, rules: CompiledRules, files: Vec<PathBuf>) -> Result<Vec<Diagnostic>>;
    fn fix(&self, rules: CompiledRules, files: Vec<PathBuf>) -> Result<FixResult>;
}

pub trait RuleAuthoring {
    fn suggest_rule(&self, feedback: String, context: ProjectContext) -> Result<RuleDraft>;
}

pub trait Reporter {
    fn report(&self, diagnostics: Vec<Diagnostic>) -> Result<()>;
}

pub trait RuleRegistry {
    fn search(&self, query: RegistryQuery) -> Result<Vec<RegistryCandidate>>;
}
