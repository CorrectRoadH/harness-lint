use std::fs;
use std::path::Path;

use anyhow::Result;

use crate::model::{RegistryCandidate, RegistryPack, RegistryQuery};

pub fn infer_project_context(root: &Path) -> RegistryQuery {
    RegistryQuery {
        feedback: String::new(),
        languages: infer_languages(root),
        libraries: infer_libraries(root),
    }
}

pub fn search_registry(
    query: &RegistryQuery,
    registry_url: Option<&str>,
) -> Result<Vec<RegistryCandidate>> {
    let _registry_url = registry_url;

    Ok(local_catalog_candidates(query))
}

pub fn inspect_pack(id: &str) -> Option<RegistryPack> {
    catalog().into_iter().find(|pack| pack.id == id)
}

fn local_catalog_candidates(query: &RegistryQuery) -> Vec<RegistryCandidate> {
    let mut candidates = Vec::new();
    let haystack = searchable_terms(query);
    for pack in catalog() {
        let language_match = pack
            .languages
            .iter()
            .any(|language| query.languages.iter().any(|actual| actual == language));
        let pack_keyword_match = pack
            .keywords
            .iter()
            .any(|keyword| haystack.contains(keyword));
        for mut rule in pack.rules {
            let rule_keyword_match = rule
                .reason
                .split(|ch: char| !ch.is_ascii_alphanumeric())
                .filter(|word| word.len() > 2)
                .any(|word| haystack.contains(&word.to_ascii_lowercase()));
            if language_match || pack_keyword_match || rule_keyword_match {
                rule.score += if language_match { 45 } else { 0 };
                rule.score += if pack_keyword_match { 25 } else { 0 };
                rule.score += if rule_keyword_match { 20 } else { 0 };
                candidates.push(rule);
            }
        }
    }
    candidates.sort_by(|left, right| {
        right
            .score
            .cmp(&left.score)
            .then(left.pack_id.cmp(&right.pack_id))
            .then(left.rule_id.cmp(&right.rule_id))
    });
    candidates.truncate(12);
    candidates
}

fn searchable_terms(query: &RegistryQuery) -> String {
    let mut terms = Vec::new();
    terms.push(query.feedback.to_ascii_lowercase());
    terms.extend(
        query
            .languages
            .iter()
            .map(|language| language.to_ascii_lowercase()),
    );
    terms.extend(
        query
            .libraries
            .iter()
            .map(|library| library.to_ascii_lowercase()),
    );
    terms.join(" ")
}

fn catalog() -> Vec<RegistryPack> {
    vec![
        RegistryPack {
            id: "python".to_string(),
            title: "Python application conventions".to_string(),
            description: "Typed Python runtime rules for explicit boundaries, safer dynamic access, and common debug hazards.".to_string(),
            pack_spec: "github:CorrectRoadH/harness-lint@main#packs/python".to_string(),
            languages: vec!["python".to_string()],
            keywords: vec![
                "python".to_string(),
                "pydantic".to_string(),
                "typed".to_string(),
                "getattr".to_string(),
                "object".to_string(),
            ],
            rules: vec![
                candidate("python", "python.no-getattr-flow", "Avoid getattr in normal application flow", "Flag dynamic getattr calls so agents prefer explicit fields, typed adapters, or real boundary objects.", 20),
                candidate("python", "python.no-broad-object-type", "Avoid broad object types", "Flag broad object type usage so code moves toward concrete models, unions, Protocols, or TypedDict-style structures.", 20),
                candidate("python", "python.no-print-debug", "Avoid committed print debugging", "Flag print calls in Python application code where logging or structured diagnostics should be used.", 15),
                candidate("python", "python.no-eval-exec", "Avoid eval and exec", "Flag eval and exec calls because they are rarely appropriate in normal application code.", 20),
                candidate("python", "python.no-runtime-assert", "Avoid assert for runtime validation", "Flag assert statements in runtime code because optimized Python execution can remove them.", 15),
            ],
        },
        RegistryPack {
            id: "go".to_string(),
            title: "Go service conventions".to_string(),
            description: "Small Go rules for production service hygiene: context flow, panics, and debug output.".to_string(),
            pack_spec: "github:CorrectRoadH/harness-lint@main#packs/go".to_string(),
            languages: vec!["go".to_string()],
            keywords: vec![
                "go".to_string(),
                "golang".to_string(),
                "context".to_string(),
                "panic".to_string(),
            ],
            rules: vec![
                candidate("go", "go.no-context-todo", "Avoid context.TODO in application flow", "Flag context.TODO calls so code accepts or derives an explicit lifecycle context.", 20),
                candidate("go", "go.no-panic-flow", "Avoid panic in normal service flow", "Flag panic calls in service code where errors should usually be returned or handled.", 20),
                candidate("go", "go.no-fmt-print-debug", "Avoid fmt print debugging", "Flag fmt.Print-style calls that should usually become structured logging.", 15),
                candidate("go", "go.no-process-exit-flow", "Avoid process exits in service flow", "Flag log.Fatal and os.Exit calls so reusable code returns errors to the application boundary.", 15),
            ],
        },
        RegistryPack {
            id: "typescript".to_string(),
            title: "TypeScript application conventions".to_string(),
            description: "TypeScript rules for safer application code: no committed console debugging, no var, and less untyped escape hatches.".to_string(),
            pack_spec: "github:CorrectRoadH/harness-lint@main#packs/typescript".to_string(),
            languages: vec![
                "typescript".to_string(),
                "javascript".to_string(),
            ],
            keywords: vec![
                "typescript".to_string(),
                "javascript".to_string(),
                "react".to_string(),
                "next".to_string(),
                "console".to_string(),
            ],
            rules: vec![
                candidate("typescript", "typescript.no-console-log", "Avoid committed console.log", "Flag console.log calls in application code where logging, telemetry, or UI state should be explicit.", 20),
                candidate("typescript", "typescript.no-var", "Avoid var declarations", "Flag var declarations so code uses block-scoped let or const.", 15),
                candidate("typescript", "typescript.no-explicit-any", "Avoid explicit any", "Flag explicit any annotations so code moves toward unknown, generics, discriminated unions, or domain types.", 15),
                candidate("typescript", "typescript.no-debugger", "Avoid committed debugger statements", "Flag debugger statements before they land in application code.", 15),
            ],
        },
    ]
}

fn candidate(
    pack_id: &str,
    rule_id: &str,
    title: &str,
    reason: &str,
    score: u32,
) -> RegistryCandidate {
    let pack_spec = format!("github:CorrectRoadH/harness-lint@main#packs/{pack_id}");
    RegistryCandidate {
        rule_id: rule_id.to_string(),
        title: title.to_string(),
        pack_id: pack_id.to_string(),
        pack_spec,
        score,
        reason: reason.to_string(),
    }
}

fn infer_languages(root: &Path) -> Vec<String> {
    let mut languages = Vec::new();
    let markers = [
        ("pyproject.toml", "python"),
        ("requirements.txt", "python"),
        ("package.json", "javascript"),
        ("tsconfig.json", "typescript"),
        ("Cargo.toml", "rust"),
        ("go.mod", "go"),
    ];
    for (marker, language) in markers {
        if root.join(marker).exists() {
            languages.push(language.to_string());
        }
    }
    languages.sort();
    languages.dedup();
    languages
}

fn infer_libraries(root: &Path) -> Vec<String> {
    let mut libraries = Vec::new();
    for file in [
        "pyproject.toml",
        "requirements.txt",
        "package.json",
        "Cargo.toml",
        "go.mod",
    ] {
        let path = root.join(file);
        let Ok(content) = fs::read_to_string(path) else {
            continue;
        };
        for library in [
            "pydantic", "fastapi", "django", "react", "next", "vue", "svelte", "serde", "tokio",
            "ruff", "gin", "echo",
        ] {
            if content.to_ascii_lowercase().contains(library) {
                libraries.push(library.to_string());
            }
        }
    }
    libraries.sort();
    libraries.dedup();
    libraries
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catalog_finds_pydantic_candidate() {
        let query = RegistryQuery {
            feedback: "prefer pydantic for validation".to_string(),
            languages: vec!["python".to_string()],
            libraries: vec![],
        };
        let candidates = search_registry(&query, None).unwrap();
        assert!(
            candidates
                .iter()
                .any(|candidate| candidate.pack_id == "python")
        );
    }

    #[test]
    fn inspect_returns_pack_details() {
        let pack = inspect_pack("typescript").unwrap();
        assert_eq!(pack.id, "typescript");
        assert!(!pack.rules.is_empty());
    }
}
