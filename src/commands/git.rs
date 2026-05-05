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
