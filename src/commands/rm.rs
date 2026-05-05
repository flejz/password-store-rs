use std::fs;
use std::io::{self, Write};
use anyhow::{bail, Result};
use crate::store::Store;

pub fn run(store: &Store, name: &str, recursive: bool, force: bool) -> Result<()> {
    store.assert_exists()?;
    let pass_file = store.pass_file(name)?;
    let pass_dir = store.root.join(name);

    if recursive && pass_dir.is_dir() {
        if !force {
            print!(
                "Are you sure you want to delete {} and all its contents? [y/N] ",
                name
            );
            io::stdout().flush()?;
            let mut answer = String::new();
            io::stdin().read_line(&mut answer)?;
            if !answer.trim().eq_ignore_ascii_case("y") {
                bail!("Aborted.");
            }
        }
        store.git_commit(&pass_dir, &format!("Remove password directory for {}.", name));
        fs::remove_dir_all(&pass_dir)?;
        println!("Removed {}/.", name);
    } else if pass_file.exists() {
        if !force {
            print!("Are you sure you want to delete {}? [y/N] ", name);
            io::stdout().flush()?;
            let mut answer = String::new();
            io::stdin().read_line(&mut answer)?;
            if !answer.trim().eq_ignore_ascii_case("y") {
                bail!("Aborted.");
            }
        }
        store.git_commit(&pass_file, &format!("Remove password for {}.", name));
        fs::remove_file(&pass_file)?;
        println!("Removed {}.", name);
    } else {
        bail!("{} is not in the password store.", name);
    }

    Ok(())
}
