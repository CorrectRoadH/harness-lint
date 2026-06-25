use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result, bail};

pub fn changed_files(root: &Path, base: &str) -> Result<Vec<PathBuf>> {
    let mut files = run_git_lines(
        root,
        &[
            "diff",
            "--name-only",
            "--diff-filter=ACMR",
            &format!("{base}...HEAD"),
        ],
    )
    .with_context(|| format!("failed to compare changed files against `{base}`"))?;
    files.extend(run_git_lines(
        root,
        &["diff", "--name-only", "--diff-filter=ACMR", "--staged"],
    )?);
    files.extend(run_git_lines(
        root,
        &["ls-files", "--others", "--exclude-standard"],
    )?);
    files.sort();
    files.dedup();
    Ok(files.into_iter().map(PathBuf::from).collect())
}

pub fn staged_files(root: &Path) -> Result<Vec<PathBuf>> {
    Ok(run_git_lines(
        root,
        &["diff", "--name-only", "--diff-filter=ACMR", "--staged"],
    )?
    .into_iter()
    .map(PathBuf::from)
    .collect())
}

fn run_git_lines(root: &Path, args: &[&str]) -> Result<Vec<String>> {
    let output = Command::new("git")
        .current_dir(root)
        .args(args)
        .output()
        .with_context(|| format!("failed to run git {}", args.join(" ")))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let detail = stderr.trim();
        if detail.is_empty() {
            bail!("git {} exited with {}", args.join(" "), output.status);
        }
        bail!("git {} failed: {detail}", args.join(" "));
    }
    Ok(String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(ToOwned::to_owned)
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn staged_files_fails_outside_git_repo() {
        let tempdir = tempfile::tempdir().unwrap();
        let error = staged_files(tempdir.path()).unwrap_err().to_string();
        assert!(error.contains("git diff"));
    }
}
