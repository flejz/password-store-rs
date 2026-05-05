use anyhow::{bail, Result};
use std::process::Command;
use crate::store::Store;

pub fn run(store: &Store, args: &[String]) -> Result<()> {
    let store_str = store.root.to_str()
        .ok_or_else(|| anyhow::anyhow!("Store path is not valid UTF-8"))?;

    let status = Command::new("git")
        .args(["-C", store_str])
        .args(args)
        .status()
        .map_err(|_| anyhow::anyhow!("git not found. Install Git from https://git-scm.com"))?;

    if !status.success() {
        bail!("git exited with status {}", status);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::store::Store;

    #[test]
    fn non_utf8_store_path_errors() {
        // Store with a valid UTF-8 path should produce a valid str
        let dir = tempfile::tempdir().unwrap();
        let store = Store::open(dir.path().to_path_buf());
        assert!(store.root.to_str().is_some());
    }
}
