use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use ignore::WalkBuilder;

use crate::config::ObsidianSection;
use crate::model::{Diagnostic, Severity};

#[derive(Debug, Clone)]
struct Reference {
    target: String,
    line: u32,
    column: u32,
}

#[derive(Debug)]
struct ObsidianIndex {
    files: BTreeSet<PathBuf>,
    by_basename: BTreeMap<String, Vec<PathBuf>>,
    by_stem: BTreeMap<String, Vec<PathBuf>>,
    inbound: BTreeMap<PathBuf, BTreeSet<PathBuf>>,
}

pub fn run_checks(
    root: &Path,
    config: &ObsidianSection,
    selected_paths: &[PathBuf],
) -> Result<Vec<Diagnostic>> {
    if !config.markdown_links
        && !config.orphan_files
        && config.flat_attachment_dir.is_none()
        && config.content_roots.is_empty()
        && !config.require_capitalized_dirs
    {
        return Ok(Vec::new());
    }

    let index = ObsidianIndex::build(root)?;
    let mut diagnostics = Vec::new();

    for path in selected_paths {
        if has_hidden_component(path) {
            continue;
        }
        if config.require_capitalized_dirs
            && let Some(directory) = first_lowercase_directory(path)
        {
            diagnostics.push(Diagnostic {
                rule_id: "obsidian.directory-uppercase".to_string(),
                level: Severity::Warn,
                message: format!("非隐藏文件夹应该以大写开头：{directory}"),
                path: path.clone(),
                start_line: 1,
                start_column: 1,
                end_line: None,
                end_column: None,
                fix_available: false,
            });
        }
        if is_disallowed_content_file(path, config) {
            diagnostics.push(Diagnostic {
                rule_id: "obsidian.content-file-kind".to_string(),
                level: Severity::Warn,
                message: "内容目录只能放 md、base、canvas 等 Obsidian 内容文件；附件请放到 Attachments/。".to_string(),
                path: path.clone(),
                start_line: 1,
                start_column: 1,
                end_line: None,
                end_column: None,
                fix_available: false,
            });
        }
        if config.markdown_links && is_note_path(path, config) && is_markdown(path) {
            diagnostics.extend(check_markdown_references(root, path, &index)?);
        }
        if config.orphan_files && is_orphan_candidate(path, config) && !index.has_inbound(path) {
            diagnostics.push(Diagnostic {
                rule_id: "obsidian.orphan-file".to_string(),
                level: Severity::Warn,
                message: "文件没有被其它笔记或附件引用，可能是孤儿文件。".to_string(),
                path: path.clone(),
                start_line: 1,
                start_column: 1,
                end_line: None,
                end_column: None,
                fix_available: false,
            });
        }
        if is_nested_attachment(path, config) {
            let attachment_dir = config
                .flat_attachment_dir
                .as_ref()
                .map(|path| path.display().to_string())
                .unwrap_or_else(|| "attachment directory".to_string());
            diagnostics.push(Diagnostic {
                rule_id: "obsidian.attachment-flat".to_string(),
                level: Severity::Warn,
                message: format!("附件应该平铺在 {attachment_dir}/ 下，不要放进子目录。"),
                path: path.clone(),
                start_line: 1,
                start_column: 1,
                end_line: None,
                end_column: None,
                fix_available: false,
            });
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

impl ObsidianIndex {
    fn build(root: &Path) -> Result<Self> {
        let mut files = BTreeSet::new();
        let mut by_basename: BTreeMap<String, Vec<PathBuf>> = BTreeMap::new();
        let mut by_stem: BTreeMap<String, Vec<PathBuf>> = BTreeMap::new();

        for entry in WalkBuilder::new(root)
            .hidden(false)
            .git_ignore(true)
            .git_exclude(true)
            .build()
        {
            let entry = entry.context("failed to walk Obsidian vault")?;
            if !entry
                .file_type()
                .map(|kind| kind.is_file())
                .unwrap_or(false)
            {
                continue;
            }
            let path = entry.path();
            let relative = path.strip_prefix(root).unwrap_or(path);
            if is_internal_or_metadata(relative) {
                continue;
            }
            let relative = relative.to_path_buf();
            files.insert(relative.clone());
            if let Some(name) = relative.file_name().and_then(|name| name.to_str()) {
                by_basename
                    .entry(name.to_string())
                    .or_default()
                    .push(relative.clone());
            }
            if is_markdown(&relative) {
                if let Some(stem) = relative.file_stem().and_then(|stem| stem.to_str()) {
                    by_stem
                        .entry(stem.to_string())
                        .or_default()
                        .push(relative.clone());
                }
            }
        }

        let mut index = Self {
            files,
            by_basename,
            by_stem,
            inbound: BTreeMap::new(),
        };
        index.populate_inbound(root)?;
        Ok(index)
    }

    fn populate_inbound(&mut self, root: &Path) -> Result<()> {
        for source in self.files.iter().filter(|path| is_markdown(path)) {
            let content = fs::read_to_string(root.join(source))
                .with_context(|| format!("failed to read {}", root.join(source).display()))?;
            for reference in extract_references(&content) {
                if let Some(target) = self.resolve_from(&reference.target, source.parent()) {
                    if target != *source {
                        self.inbound
                            .entry(target)
                            .or_default()
                            .insert(source.clone());
                    }
                }
            }
        }
        Ok(())
    }

    fn resolve_from(&self, raw_target: &str, source_parent: Option<&Path>) -> Option<PathBuf> {
        let target = normalize_target(raw_target)?;
        let target_path = PathBuf::from(&target);
        if target.contains('/') {
            let target_path = normalize_relative_path(source_parent, &target_path);
            if self.files.contains(&target_path) {
                return Some(target_path);
            }
            if target_path.extension().is_none() {
                let markdown_path = target_path.with_extension("md");
                if self.files.contains(&markdown_path) {
                    return Some(markdown_path);
                }
            }
            return None;
        }

        if self.files.contains(&target_path) {
            return Some(target_path);
        }

        if target_path.extension().is_none() {
            let markdown_path = PathBuf::from(format!("{target}.md"));
            if self.files.contains(&markdown_path) {
                return Some(markdown_path);
            }
        }

        if !target.contains('/') {
            if let Some(matches) = self.by_basename.get(&target) {
                return matches.first().cloned();
            }
            if target_path.extension().is_none() {
                if let Some(matches) = self.by_stem.get(&target) {
                    return matches.first().cloned();
                }
            }
        }

        None
    }

    fn has_inbound(&self, path: &Path) -> bool {
        self.inbound
            .get(path)
            .map(|set| !set.is_empty())
            .unwrap_or(false)
    }
}

fn check_markdown_references(
    root: &Path,
    path: &Path,
    index: &ObsidianIndex,
) -> Result<Vec<Diagnostic>> {
    let content = fs::read_to_string(root.join(path))
        .with_context(|| format!("failed to read {}", root.join(path).display()))?;
    let mut diagnostics = Vec::new();
    for reference in extract_references(&content) {
        if !should_report_missing_target(&reference.target) {
            continue;
        }
        if index
            .resolve_from(&reference.target, path.parent())
            .is_none()
        {
            diagnostics.push(Diagnostic {
                rule_id: "obsidian.missing-link".to_string(),
                level: Severity::Error,
                message: format!("引用目标不存在：{}", reference.target),
                path: path.to_path_buf(),
                start_line: reference.line,
                start_column: reference.column,
                end_line: None,
                end_column: None,
                fix_available: false,
            });
        }
    }
    Ok(diagnostics)
}

fn extract_references(content: &str) -> Vec<Reference> {
    let mut references = Vec::new();
    let mut in_code_fence = false;
    for (line_index, line) in content.lines().enumerate() {
        if line.trim_start().starts_with("```") {
            in_code_fence = !in_code_fence;
            continue;
        }
        if in_code_fence {
            continue;
        }
        references.extend(extract_wikilinks(line, line_index as u32 + 1));
        references.extend(extract_markdown_links(line, line_index as u32 + 1));
    }
    references
}

fn extract_wikilinks(line: &str, line_number: u32) -> Vec<Reference> {
    let mut references = Vec::new();
    let mut offset = 0;
    while let Some(start) = line[offset..].find("[[") {
        let start_index = offset + start;
        let target_start = start_index + 2;
        let Some(end) = line[target_start..].find("]]") else {
            break;
        };
        let raw = &line[target_start..target_start + end];
        let target = raw
            .split('|')
            .next()
            .unwrap_or(raw)
            .split('#')
            .next()
            .unwrap_or(raw)
            .trim();
        if !target.is_empty() {
            references.push(Reference {
                target: target.to_string(),
                line: line_number,
                column: start_index as u32 + 1,
            });
        }
        offset = target_start + end + 2;
    }
    references
}

fn extract_markdown_links(line: &str, line_number: u32) -> Vec<Reference> {
    let mut references = Vec::new();
    let mut offset = 0;
    while let Some(label_end) = line[offset..].find("](") {
        let open = offset + label_end + 2;
        let Some(close) = line[open..].find(')') else {
            break;
        };
        let raw = line[open..open + close].trim();
        let target = raw.split('#').next().unwrap_or(raw).trim();
        if should_check_markdown_target(target) {
            references.push(Reference {
                target: target.to_string(),
                line: line_number,
                column: open as u32 + 1,
            });
        }
        offset = open + close + 1;
    }
    references
}

fn should_check_markdown_target(target: &str) -> bool {
    if target.is_empty()
        || target.starts_with('#')
        || target.starts_with("http://")
        || target.starts_with("https://")
        || target.starts_with("mailto:")
        || target.starts_with("obsidian://")
        || target.starts_with("tel:")
    {
        return false;
    }
    let trimmed = target.trim_matches('<').trim_matches('>');
    trimmed.contains('/')
        || Path::new(trimmed)
            .extension()
            .and_then(|extension| extension.to_str())
            .is_some()
}

fn should_report_missing_target(target: &str) -> bool {
    let Some(target) = normalize_target(target) else {
        return false;
    };
    Path::new(&target)
        .extension()
        .and_then(|extension| extension.to_str())
        .is_some()
}

fn normalize_target(raw_target: &str) -> Option<String> {
    let target = raw_target
        .trim()
        .trim_matches('<')
        .trim_matches('>')
        .trim_start_matches('/');
    if target.is_empty()
        || target.starts_with('#')
        || target.starts_with("http://")
        || target.starts_with("https://")
        || target.starts_with("mailto:")
        || target.starts_with("obsidian://")
        || target.starts_with("tel:")
    {
        return None;
    }
    Some(target.replace("%20", " "))
}

fn normalize_relative_path(source_parent: Option<&Path>, target: &Path) -> PathBuf {
    let mut normalized = PathBuf::new();
    if let Some(parent) = source_parent {
        normalized.push(parent);
    }
    normalized.push(target);

    let mut parts = Vec::new();
    for component in normalized.components() {
        match component {
            std::path::Component::CurDir => {}
            std::path::Component::ParentDir => {
                parts.pop();
            }
            std::path::Component::Normal(part) => parts.push(part.to_os_string()),
            _ => {}
        }
    }
    parts.into_iter().collect()
}

fn is_markdown(path: &Path) -> bool {
    path.extension().and_then(|ext| ext.to_str()) == Some("md")
}

fn is_orphan_candidate(path: &Path, config: &ObsidianSection) -> bool {
    config
        .note_roots
        .iter()
        .any(|root| path.starts_with(root) && is_markdown(path))
        || config
            .flat_attachment_dir
            .as_ref()
            .map(|root| path.starts_with(root))
            .unwrap_or(false)
}

fn is_note_path(path: &Path, config: &ObsidianSection) -> bool {
    config.note_roots.iter().any(|root| path.starts_with(root))
}

fn is_nested_attachment(path: &Path, config: &ObsidianSection) -> bool {
    let Some(root) = config.flat_attachment_dir.as_ref() else {
        return false;
    };
    path.starts_with(root) && path.parent() != Some(root.as_path())
}

fn is_disallowed_content_file(path: &Path, config: &ObsidianSection) -> bool {
    if config.content_roots.is_empty() || config.content_extensions.is_empty() {
        return false;
    }
    if !config
        .content_roots
        .iter()
        .any(|root| path.starts_with(root))
    {
        return false;
    }
    let Some(extension) = path.extension().and_then(|extension| extension.to_str()) else {
        return true;
    };
    !config
        .content_extensions
        .iter()
        .any(|allowed| extension.eq_ignore_ascii_case(allowed))
}

fn first_lowercase_directory(path: &Path) -> Option<String> {
    for component in path.parent()?.components() {
        let name = component.as_os_str().to_str()?;
        if name.starts_with('.') {
            continue;
        }
        let Some(first) = name.chars().next() else {
            continue;
        };
        if first.is_ascii_lowercase() {
            return Some(name.to_string());
        }
    }
    None
}

fn is_internal_or_metadata(path: &Path) -> bool {
    has_hidden_component(path)
        || path.starts_with(".git")
        || path.starts_with(".harness")
        || path.starts_with(".obsidian")
        || path.starts_with("Rules")
        || path.starts_with("rules")
        || path.starts_with("target")
        || path.starts_with("node_modules")
        || path.starts_with(".venv")
}

fn has_hidden_component(path: &Path) -> bool {
    path.components().any(|component| {
        component
            .as_os_str()
            .to_str()
            .map(|name| name.starts_with('.'))
            .unwrap_or(false)
    })
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_wikilinks_and_markdown_links() {
        let refs = extract_references(
            "[[Note|alias]] ![[image.png]] [local](Other.md) [plain](Not a path) [web](https://example.com)\n```markdown\n[[Ignored]]\n```",
        );
        let targets: Vec<_> = refs.into_iter().map(|reference| reference.target).collect();
        assert_eq!(targets, vec!["Note", "image.png", "Other.md"]);
    }

    #[test]
    fn reports_missing_links() {
        let tempdir = tempfile::tempdir().unwrap();
        fs::create_dir(tempdir.path().join("Notes")).unwrap();
        fs::write(tempdir.path().join("Notes/A.md"), "[[Missing.png]]").unwrap();
        let config = ObsidianSection {
            markdown_links: true,
            orphan_files: false,
            flat_attachment_dir: None,
            note_roots: vec![PathBuf::from("Notes")],
            content_roots: Vec::new(),
            content_extensions: Vec::new(),
            require_capitalized_dirs: false,
        };
        let diagnostics =
            run_checks(tempdir.path(), &config, &[PathBuf::from("Notes/A.md")]).unwrap();
        assert!(
            diagnostics
                .iter()
                .any(|diagnostic| diagnostic.rule_id == "obsidian.missing-link")
        );
    }

    #[test]
    fn ignores_missing_concept_wikilinks_without_file_suffix() {
        let tempdir = tempfile::tempdir().unwrap();
        fs::create_dir(tempdir.path().join("Notes")).unwrap();
        fs::write(tempdir.path().join("Notes/A.md"), "[[项目建议书]]").unwrap();
        let config = ObsidianSection {
            markdown_links: true,
            orphan_files: false,
            flat_attachment_dir: None,
            note_roots: vec![PathBuf::from("Notes")],
            content_roots: Vec::new(),
            content_extensions: Vec::new(),
            require_capitalized_dirs: false,
        };
        let diagnostics =
            run_checks(tempdir.path(), &config, &[PathBuf::from("Notes/A.md")]).unwrap();
        assert!(
            !diagnostics
                .iter()
                .any(|diagnostic| diagnostic.rule_id == "obsidian.missing-link")
        );
    }

    #[test]
    fn resolves_relative_attachment_paths() {
        let tempdir = tempfile::tempdir().unwrap();
        fs::create_dir(tempdir.path().join("Daily")).unwrap();
        fs::create_dir(tempdir.path().join("Attachments")).unwrap();
        fs::write(
            tempdir.path().join("Daily/A.md"),
            "![](../Attachments/a.png)",
        )
        .unwrap();
        fs::write(tempdir.path().join("Attachments/a.png"), "").unwrap();
        let config = ObsidianSection {
            markdown_links: true,
            orphan_files: false,
            flat_attachment_dir: Some(PathBuf::from("Attachments")),
            note_roots: vec![PathBuf::from("Daily")],
            content_roots: Vec::new(),
            content_extensions: Vec::new(),
            require_capitalized_dirs: false,
        };
        let diagnostics =
            run_checks(tempdir.path(), &config, &[PathBuf::from("Daily/A.md")]).unwrap();
        assert!(
            !diagnostics
                .iter()
                .any(|diagnostic| diagnostic.rule_id == "obsidian.missing-link")
        );
    }

    #[test]
    fn reports_nested_attachments() {
        let tempdir = tempfile::tempdir().unwrap();
        fs::create_dir_all(tempdir.path().join("Attachments/nested")).unwrap();
        fs::write(tempdir.path().join("Attachments/nested/a.png"), "").unwrap();
        let config = ObsidianSection {
            markdown_links: false,
            orphan_files: false,
            flat_attachment_dir: Some(PathBuf::from("Attachments")),
            note_roots: Vec::new(),
            content_roots: Vec::new(),
            content_extensions: Vec::new(),
            require_capitalized_dirs: false,
        };
        let diagnostics = run_checks(
            tempdir.path(),
            &config,
            &[PathBuf::from("Attachments/nested/a.png")],
        )
        .unwrap();
        assert!(
            diagnostics
                .iter()
                .any(|diagnostic| diagnostic.rule_id == "obsidian.attachment-flat")
        );
    }

    #[test]
    fn reports_content_files_and_lowercase_directories() {
        let tempdir = tempfile::tempdir().unwrap();
        fs::create_dir_all(tempdir.path().join("Notes/assets")).unwrap();
        fs::write(tempdir.path().join("Notes/assets/a.png"), "").unwrap();
        let config = ObsidianSection {
            markdown_links: false,
            orphan_files: false,
            flat_attachment_dir: None,
            note_roots: Vec::new(),
            content_roots: vec![PathBuf::from("Notes")],
            content_extensions: vec!["md".to_string(), "base".to_string(), "canvas".to_string()],
            require_capitalized_dirs: true,
        };
        let diagnostics = run_checks(
            tempdir.path(),
            &config,
            &[PathBuf::from("Notes/assets/a.png")],
        )
        .unwrap();
        assert!(
            diagnostics
                .iter()
                .any(|diagnostic| diagnostic.rule_id == "obsidian.content-file-kind")
        );
        assert!(
            diagnostics
                .iter()
                .any(|diagnostic| diagnostic.rule_id == "obsidian.directory-uppercase")
        );
    }
}
