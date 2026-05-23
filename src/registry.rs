use std::fs;
use std::path::Path;

use anyhow::Result;

use crate::model::{RegistryCandidate, RegistryQuery};

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
    let Some(_registry_url) = registry_url else {
        return Ok(Vec::new());
    };

    Ok(local_stub_candidates(query))
}

fn local_stub_candidates(query: &RegistryQuery) -> Vec<RegistryCandidate> {
    let mut candidates = Vec::new();
    let feedback = query.feedback.to_ascii_lowercase();
    if query.languages.iter().any(|language| language == "python")
        && (feedback.contains("pydantic")
            || feedback.contains("validation")
            || query.libraries.iter().any(|library| library == "pydantic"))
    {
        candidates.push(RegistryCandidate {
            rule_id: "python.prefer-pydantic".to_string(),
            title: "Prefer Pydantic for structured validation".to_string(),
            pack_id: "python".to_string(),
            pack_spec: "github:harness-lint/rules-python@latest".to_string(),
            score: 90,
            reason:
                "Project appears to use Python and the feedback mentions validation or Pydantic."
                    .to_string(),
        });
    }
    candidates
}

fn infer_languages(root: &Path) -> Vec<String> {
    let mut languages = Vec::new();
    let markers = [
        ("pyproject.toml", "python"),
        ("requirements.txt", "python"),
        ("package.json", "javascript"),
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
    ] {
        let path = root.join(file);
        let Ok(content) = fs::read_to_string(path) else {
            continue;
        };
        for library in ["pydantic", "react", "next", "serde", "tokio", "ruff"] {
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
    fn stub_finds_pydantic_candidate() {
        let query = RegistryQuery {
            feedback: "prefer pydantic for validation".to_string(),
            languages: vec!["python".to_string()],
            libraries: vec![],
        };
        let candidates = search_registry(&query, Some("stub")).unwrap();
        assert_eq!(candidates[0].rule_id, "python.prefer-pydantic");
    }
}
