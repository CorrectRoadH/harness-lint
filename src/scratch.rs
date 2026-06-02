use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

#[derive(Debug)]
pub struct ScratchDir {
    path: PathBuf,
}

impl ScratchDir {
    pub fn new(prefix: &str) -> Result<Self> {
        let mut last_error = None;
        for attempt in 0..100 {
            let path = std::env::temp_dir().join(format!(
                "{prefix}-{}-{}-{attempt}",
                std::process::id(),
                monotonic_nanos()
            ));
            match std::fs::create_dir(&path) {
                Ok(()) => return Ok(Self { path }),
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
                    last_error = Some(error);
                }
                Err(error) => {
                    return Err(error).with_context(|| {
                        format!("failed to create scratch directory {}", path.display())
                    });
                }
            }
        }
        let error = last_error
            .map(anyhow::Error::from)
            .unwrap_or_else(|| anyhow::anyhow!("failed to allocate scratch directory"));
        Err(error).context("failed to allocate scratch directory")
    }

    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for ScratchDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.path);
    }
}

fn monotonic_nanos() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or_default()
}
