use std::fs;
use std::io::{self, Write};
use anyhow::{bail, Result};
use crate::{gpg::Gpg, store::Store};

pub fn run(
    store: &Store,
    gpg: &Gpg,
    old_name: &str,
    new_name: &str,
    force: bool,
) -> Result<()> {
    store.assert_exists()?;
    let old_file = store.pass_file(old_name)?;
    let new_file = store.pass_file(new_name)?;

    if !old_file.exists() {
        bail!("{} is not in the password store.", old_name);
    }

    if new_file.exists() && !force {
        print!("An entry already exists for {}. Overwrite it? [y/N] ", new_name);
        io::stdout().flush()?;
        let mut answer = String::new();
        io::stdin().read_line(&mut answer)?;
        if !answer.trim().eq_ignore_ascii_case("y") {
            bail!("Aborted.");
        }
    }

    if let Some(parent) = new_file.parent() {
        fs::create_dir_all(parent)?;
    }

    let old_recipients = store.recipients_for(&old_file)?;
    let new_recipients = store.recipients_for(&new_file)?;

    if old_recipients == new_recipients {
        fs::rename(&old_file, &new_file)?;
    } else {
        let data = gpg.decrypt(&old_file)?;
        gpg.encrypt(&data, &new_recipients, &new_file)?;
        fs::remove_file(&old_file)?;
    }

    println!("Renamed {} to {}.", old_name, new_name);

    // Commit removal of old and addition of new in one shot
    let store_str = store.root.to_str().unwrap_or("");
    if store.root.join(".git").exists() {
        let _ = std::process::Command::new("git")
            .args(["-C", store_str, "add", "--all"])
            .status();
        let _ = std::process::Command::new("git")
            .args(["-C", store_str, "commit", "-m",
                   &format!("Rename {} to {}.", old_name, new_name)])
            .status();
    }

    Ok(())
}
