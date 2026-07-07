use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::OnceLock;

use anyhow::{Context, Result, bail};
use serde::Deserialize;

use crate::model::{CompiledRules, Diagnostic, Severity};
use crate::scratch::ScratchDir;

pub fn ensure_grit_available() -> Result<String> {
    static VERSION: OnceLock<std::result::Result<String, String>> = OnceLock::new();
    VERSION
        .get_or_init(|| {
            let output = Command::new("grit")
                .env("GRIT_TELEMETRY_DISABLED", "true")
                .arg("--version")
                .output()
                .map_err(|error| {
                    format!(
                        "failed to run `grit --version`; install Grit CLI before running checks: {error}"
                    )
                })?;
            if !output.status.success() {
                return Err("`grit --version` failed; install or repair the Grit CLI".to_string());
            }
            Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
        })
        .clone()
        .map_err(|error| anyhow::anyhow!(error))
}

pub fn check_grit_compatibility(version_output: &str) -> Result<()> {
    if version_output.trim().is_empty() {
        bail!("`grit --version` returned an empty version string");
    }
    Ok(())
}

pub fn run_grit(
    root: &Path,
    compiled: &CompiledRules,
    paths: &[PathBuf],
) -> Result<Vec<Diagnostic>> {
    if compiled.grit_rules.is_empty() {
        return Ok(Vec::new());
    }

    let version = ensure_grit_available()?;
    check_grit_compatibility(&version)?;
    let grit_cache_dir = root.join(".harness/cache/grit");
    std::fs::create_dir_all(&grit_cache_dir)
        .with_context(|| format!("failed to create {}", grit_cache_dir.display()))?;

    let mut command = Command::new("grit");
    let grit_work_dir = compiled.grit_dir.parent().unwrap_or(root);
    command
        .current_dir(grit_work_dir)
        .env("GRIT_TELEMETRY_DISABLED", "true")
        .env("GRIT_CACHE_DIR", &grit_cache_dir)
        .arg("--json")
        .arg("check");

    if paths.is_empty() {
        command.arg(root);
    } else {
        command.args(paths.iter().map(|path| root.join(path)));
    }

    let output = command.output().context("failed to run `grit check`")?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let diagnostics = parse_grit_output(&stdout, &stderr);

    if !output.status.success() && diagnostics.is_empty() {
        bail!("`grit check` failed: {}", stderr.trim());
    }
    // `grit check` exits non-zero for matches too, so a compile error in one
    // pattern can hide behind another pattern's diagnostics. Surface stderr so
    // a partially failing run is not mistaken for complete coverage.
    if !output.status.success() && !diagnostics.is_empty() && !stderr.trim().is_empty() {
        eprintln!("harness-lint: `grit check` reported: {}", stderr.trim());
    }

    Ok(diagnostics)
}

pub fn validate_grit_pattern(body: &str, sample_language: &str) -> Result<()> {
    let version = ensure_grit_available()?;
    check_grit_compatibility(&version)?;

    let scratch = ScratchDir::new("harness-lint-grit-validate")?;
    let grit_dir = scratch.path().join(".grit");
    let patterns_dir = grit_dir.join("patterns");
    std::fs::create_dir_all(&patterns_dir)
        .with_context(|| format!("failed to create {}", patterns_dir.display()))?;
    std::fs::write(grit_dir.join("grit.yaml"), "version: 0.0.2\npatterns: []\n")
        .context("failed to write scratch grit.yaml")?;
    std::fs::write(
        patterns_dir.join("local_validate.md"),
        format!(
            "---\ntitle: \"Validate\"\nlevel: warn\ntags: []\n---\n\n# Validate\n\n```grit\n{body}\n```\n"
        ),
    )
    .context("failed to write scratch GritQL pattern")?;

    let sample_path = scratch
        .path()
        .join("src")
        .join(format!("bad-example.{}", sample_extension(sample_language)));
    if let Some(parent) = sample_path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }
    std::fs::write(&sample_path, "").with_context(|| {
        format!(
            "failed to write scratch source sample {}",
            sample_path.display()
        )
    })?;

    let grit_cache_dir = scratch.path().join("cache");
    std::fs::create_dir_all(&grit_cache_dir)
        .with_context(|| format!("failed to create {}", grit_cache_dir.display()))?;
    let output = Command::new("grit")
        .current_dir(scratch.path())
        .env("GRIT_TELEMETRY_DISABLED", "true")
        .env("GRIT_CACHE_DIR", &grit_cache_dir)
        .arg("--json")
        .arg("check")
        .arg(&sample_path)
        .output()
        .context("failed to run `grit check` for rule validation")?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let diagnostics = parse_grit_output(&stdout, &stderr);
    if !output.status.success() && diagnostics.is_empty() {
        bail!("GritQL failed validation: {}", stderr.trim());
    }
    Ok(())
}

pub fn sample_extension(language: &str) -> &'static str {
    match language.to_ascii_lowercase().as_str() {
        "typescript" | "ts" => "ts",
        "tsx" => "tsx",
        "javascript" | "ecmascript" | "node" | "nodejs" | "js" | "mjs" | "cjs" => "js",
        "jsx" => "jsx",
        "python" | "py" => "py",
        "go" | "golang" => "go",
        "rust" | "rs" => "rs",
        "ruby" | "rb" => "rb",
        "elixir" | "ex" | "exs" => "ex",
        "csharp" | "c#" | "cs" => "cs",
        "java" => "java",
        "kotlin" | "kt" | "kts" => "kt",
        "solidity" | "sol" => "sol",
        "hcl" => "hcl",
        "terraform" | "tf" => "tf",
        "html" | "htm" => "html",
        "css" => "css",
        "markdown" | "md" => "md",
        "yaml" | "yml" => "yaml",
        "json" => "json",
        "toml" => "toml",
        "sql" => "sql",
        "vue" => "vue",
        "php" => "php",
        _ => "txt",
    }
}

fn parse_grit_output(stdout: &str, stderr: &str) -> Vec<Diagnostic> {
    let json = if stdout.trim_start().starts_with('{') {
        stdout
    } else if stderr.trim_start().starts_with('{') {
        stderr
    } else {
        return parse_grit_jsonl(stdout);
    };
    match serde_json::from_str::<GritJsonOutput>(json) {
        Ok(output) => output
            .results
            .into_iter()
            .map(GritJsonResult::into_diagnostic)
            .collect(),
        Err(error) => {
            let diagnostics = parse_grit_jsonl(json);
            if diagnostics.is_empty() {
                eprintln!(
                    "harness-lint: warning: could not parse `grit check` JSON output ({error}); \
                     diagnostics from this batch may be missing"
                );
            }
            diagnostics
        }
    }
}

#[derive(Debug, Deserialize)]
struct GritJsonOutput {
    results: Vec<GritJsonResult>,
}

#[derive(Debug, Deserialize)]
struct GritJsonResult {
    #[serde(default)]
    check_id: Option<String>,
    #[serde(default)]
    local_name: Option<String>,
    start: GritJsonPosition,
    #[serde(default)]
    end: Option<GritJsonPosition>,
    path: PathBuf,
    #[serde(default)]
    extra: Option<GritJsonExtra>,
}

#[derive(Debug, Deserialize)]
struct GritJsonPosition {
    line: u32,
    #[serde(alias = "column")]
    col: u32,
}

#[derive(Debug, Deserialize, Default)]
struct GritJsonExtra {
    #[serde(default)]
    message: Option<String>,
    #[serde(default)]
    severity: Option<String>,
}

impl GritJsonResult {
    fn into_diagnostic(self) -> Diagnostic {
        let extra = self.extra.unwrap_or_default();
        Diagnostic {
            rule_id: self
                .local_name
                .or(self.check_id)
                .unwrap_or_else(|| "grit.unknown".to_string()),
            level: parse_level(extra.severity.as_deref()),
            message: extra
                .message
                .unwrap_or_else(|| "Grit diagnostic".to_string()),
            path: self.path,
            start_line: self.start.line,
            start_column: self.start.col,
            end_line: self.end.as_ref().map(|position| position.line),
            end_column: self.end.as_ref().map(|position| position.col),
            fix_available: false,
        }
    }
}

fn parse_grit_jsonl(output: &str) -> Vec<Diagnostic> {
    output
        .lines()
        .filter_map(|line| serde_json::from_str::<GritJsonLine>(line).ok())
        .filter_map(GritJsonLine::into_diagnostic)
        .collect()
}

#[derive(Debug, Deserialize)]
struct GritJsonLine {
    #[serde(default)]
    file: Option<PathBuf>,
    #[serde(default)]
    path: Option<PathBuf>,
    #[serde(default)]
    message: Option<String>,
    #[serde(default)]
    rule: Option<String>,
    #[serde(default)]
    rule_id: Option<String>,
    #[serde(default)]
    level: Option<String>,
    #[serde(default)]
    line: Option<u32>,
    #[serde(default)]
    column: Option<u32>,
    #[serde(default)]
    end_line: Option<u32>,
    #[serde(default)]
    end_column: Option<u32>,
}

impl GritJsonLine {
    fn into_diagnostic(self) -> Option<Diagnostic> {
        let path = self.path.or(self.file)?;
        Some(Diagnostic {
            rule_id: self
                .rule_id
                .or(self.rule)
                .unwrap_or_else(|| "grit.unknown".to_string()),
            level: parse_level(self.level.as_deref()),
            message: self
                .message
                .unwrap_or_else(|| "Grit diagnostic".to_string()),
            path,
            start_line: self.line.unwrap_or(1),
            start_column: self.column.unwrap_or(1),
            end_line: self.end_line,
            end_column: self.end_column,
            fix_available: false,
        })
    }
}

fn parse_level(level: Option<&str>) -> Severity {
    match level.unwrap_or("warn").to_ascii_lowercase().as_str() {
        "none" => Severity::None,
        "info" => Severity::Info,
        "error" => Severity::Error,
        _ => Severity::Warn,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_simple_jsonl_diagnostic() {
        let output = r#"{"path":"src/main.py","message":"No print","rule_id":"python.no-print","level":"error","line":2,"column":4}"#;
        let diagnostics = parse_grit_output(output, "");
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].rule_id, "python.no-print");
        assert_eq!(diagnostics[0].level, Severity::Error);
    }

    #[test]
    fn parses_grit_json_diagnostic() {
        let output = r##"{"paths":["a.md"],"results":[{"check_id":"#fence/markdown","local_name":"fence","start":{"line":1,"col":1,"offset":0},"end":{"line":1,"col":5,"offset":4},"path":"a.md","extra":{"message":"Bad fence","severity":"warn"}}]}"##;
        let diagnostics = parse_grit_output("", output);
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].rule_id, "fence");
        assert_eq!(diagnostics[0].message, "Bad fence");
        assert_eq!(diagnostics[0].path, PathBuf::from("a.md"));
    }

    #[test]
    fn accepts_non_empty_grit_version() {
        check_grit_compatibility("grit 0.1.0").unwrap();
        assert!(check_grit_compatibility("").is_err());
    }

    #[test]
    fn sample_extension_covers_grit_cli_languages_and_aliases() {
        let cases = [
            ("typescript", "ts"),
            ("tsx", "tsx"),
            ("javascript", "js"),
            ("jsx", "jsx"),
            ("python", "py"),
            ("go", "go"),
            ("rust", "rs"),
            ("ruby", "rb"),
            ("elixir", "ex"),
            ("csharp", "cs"),
            ("java", "java"),
            ("kotlin", "kt"),
            ("solidity", "sol"),
            ("hcl", "hcl"),
            ("terraform", "tf"),
            ("html", "html"),
            ("css", "css"),
            ("markdown", "md"),
            ("yaml", "yaml"),
            ("json", "json"),
            ("toml", "toml"),
            ("sql", "sql"),
            ("vue", "vue"),
            ("php", "php"),
        ];
        for (language, extension) in cases {
            assert_eq!(sample_extension(language), extension);
        }
    }
}
