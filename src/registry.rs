use std::fs;
use std::path::Path;
use std::time::Duration;

use anyhow::{Context, Result};

use crate::model::{RegistryCandidate, RegistryPack, RegistryQuery};

const EMBEDDED_CATALOG: &str = include_str!("../site/catalog.json");

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
    Ok(local_catalog_candidates(query, catalog(registry_url)))
}

pub fn inspect_pack(id: &str, registry_url: Option<&str>) -> Result<Option<RegistryPack>> {
    Ok(catalog(registry_url).into_iter().find(|pack| pack.id == id))
}

pub fn list_packs(registry_url: Option<&str>) -> Result<Vec<RegistryPack>> {
    let mut packs = catalog(registry_url);
    packs.sort_by(|left, right| left.id.cmp(&right.id));
    Ok(packs)
}

fn catalog(registry_url: Option<&str>) -> Vec<RegistryPack> {
    if let Some(url) = registry_url {
        match remote_catalog(url) {
            Ok(catalog) => return catalog,
            Err(error) => {
                eprintln!(
                    "harness-lint: warning: could not load registry {url} ({error:#}); \
                     falling back to the embedded catalog"
                );
            }
        }
    }
    embedded_catalog()
}

fn remote_catalog(registry_url: &str) -> Result<Vec<RegistryPack>> {
    if let Some(path) = registry_url.strip_prefix("file://") {
        return read_catalog_file(Path::new(path));
    }
    let path = Path::new(registry_url);
    if path.exists() {
        return read_catalog_file(path);
    }

    let url = catalog_url(registry_url);
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(5))
        .user_agent("harness-lint")
        .build()
        .context("failed to build registry client")?;
    let response = client
        .get(url)
        .send()
        .context("failed to fetch registry catalog")?
        .error_for_status()
        .context("registry catalog returned an error")?;
    response
        .json()
        .context("failed to parse registry catalog response")
}

fn read_catalog_file(path: &Path) -> Result<Vec<RegistryPack>> {
    let content =
        fs::read_to_string(path).with_context(|| format!("failed to read {}", path.display()))?;
    serde_json::from_str(&content).with_context(|| format!("failed to parse {}", path.display()))
}

fn catalog_url(registry_url: &str) -> String {
    let trimmed = registry_url.trim_end_matches('/');
    if trimmed.ends_with(".json") {
        trimmed.to_string()
    } else {
        format!("{trimmed}/catalog.json")
    }
}

fn embedded_catalog() -> Vec<RegistryPack> {
    serde_json::from_str(EMBEDDED_CATALOG).expect("embedded registry catalog is valid JSON")
}

fn local_catalog_candidates(
    query: &RegistryQuery,
    catalog: Vec<RegistryPack>,
) -> Vec<RegistryCandidate> {
    let mut candidates = Vec::new();
    let haystack = searchable_terms(query);
    for pack in catalog {
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
        let pack = inspect_pack("typescript", None).unwrap().unwrap();
        assert_eq!(pack.id, "typescript");
        assert!(!pack.rules.is_empty());
    }
}
