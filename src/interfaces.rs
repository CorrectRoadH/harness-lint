use std::path::PathBuf;

use anyhow::Result;

use crate::model::{
    CompiledRules, CreatedRule, Diagnostic, LockEntry, PackSpec, ProjectContext, RegistryCandidate,
    RegistryQuery, ResolvedPack, RulePack,
};

pub trait RuleSource {
    fn resolve(&self, spec: PackSpec) -> Result<ResolvedPack>;
    fn update(&self, lock: LockEntry) -> Result<ResolvedPack>;
}

pub trait RuleCompiler {
    fn compile(&self, packs: Vec<RulePack>) -> Result<CompiledRules>;
}

pub trait GritRunner {
    fn check(&self, rules: CompiledRules, files: Vec<PathBuf>) -> Result<Vec<Diagnostic>>;
}

pub trait RuleAuthoring {
    fn create_rule(
        &self,
        feedback: String,
        language: String,
        grit: String,
        context: ProjectContext,
    ) -> Result<CreatedRule>;
}

pub trait Reporter {
    fn report(&self, diagnostics: Vec<Diagnostic>) -> Result<()>;
}

pub trait RuleRegistry {
    fn search(&self, query: RegistryQuery) -> Result<Vec<RegistryCandidate>>;
}
