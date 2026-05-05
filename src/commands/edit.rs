use std::fs;
use std::io::Write;
use std::process::Command;
use anyhow::{bail, Result};
use crate::{gpg::Gpg, store::Store};

pub fn run(store: &Store, gpg: &Gpg, name: &str) -> Result<()> {
    store.assert_exists()?;
    let pass_file = store.pass_file(name)?;

    let content = if pass_file.exists() {
        gpg.decrypt(&pass_file)?
    } else {
        vec![]
    };

    // Write to a named temp file so the editor can open it by path.
    // Note: this lands in %TEMP% (disk), not a RAM disk — less secure than /dev/shm.
    let mut tmp = tempfile::NamedTempFile::new()?;
    tmp.write_all(&content)?;
    tmp.flush()?;

    let editor = std::env::var("EDITOR")
        .or_else(|_| std::env::var("VISUAL"))
        .unwrap_or_else(|_| {
            if cfg!(windows) { "notepad.exe".into() } else { "vi".into() }
        });

    let status = Command::new(&editor)
        .arg(tmp.path())
        .status()
        .map_err(|_| anyhow::anyhow!("Editor '{}' not found", editor))?;

    if !status.success() {
        bail!("Editor exited with non-zero status");
    }

    let new_content = fs::read(tmp.path())?;

    if let Some(parent) = pass_file.parent() {
        fs::create_dir_all(parent)?;
    }

    let recipients = store.recipients_for(&pass_file)?;
    gpg.encrypt(&new_content, &recipients, &pass_file)?;

    println!("Password for {} saved.", name);
    store.git_commit(&pass_file, &format!("Edit password for {}.", name));
    Ok(())
}
