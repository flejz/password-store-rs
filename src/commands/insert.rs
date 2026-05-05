use std::fs;
use std::io::{self, BufRead, Write};
use anyhow::{bail, Result};
use crate::{gpg::Gpg, store::Store};

pub fn run(
    store: &Store,
    gpg: &Gpg,
    name: &str,
    echo: bool,
    multiline: bool,
    force: bool,
) -> Result<()> {
    store.assert_exists()?;
    let pass_file = store.pass_file(name)?;

    if pass_file.exists() && !force {
        print!("An entry already exists for {}. Overwrite it? [y/N] ", name);
        io::stdout().flush()?;
        let mut answer = String::new();
        io::stdin().read_line(&mut answer)?;
        if !answer.trim().eq_ignore_ascii_case("y") {
            bail!("Aborted.");
        }
    }

    let password = if multiline {
        println!("Enter contents of {} and press Ctrl+Z then Enter (Windows) when finished:", name);
        io::stdin().lock().lines()
            .collect::<Result<Vec<_>, _>>()?
            .join("\n") + "\n"
    } else if echo {
        print!("Enter password for {}: ", name);
        io::stdout().flush()?;
        let mut line = String::new();
        io::stdin().read_line(&mut line)?;
        line.trim_end_matches(['\r', '\n']).to_string() + "\n"
    } else {
        let pass = rpassword::prompt_password(format!("Enter password for {}: ", name))?;
        let confirm = rpassword::prompt_password(format!("Retype password for {}: ", name))?;
        if pass != confirm {
            bail!("Error: the entered passwords do not match.");
        }
        pass + "\n"
    };

    if let Some(parent) = pass_file.parent() {
        fs::create_dir_all(parent)?;
    }

    let recipients = store.recipients_for(&pass_file)?;
    gpg.encrypt(password.as_bytes(), &recipients, &pass_file)?;

    println!("Password for {} added.", name);
    store.git_commit(&pass_file, &format!("Add password for {}.", name));
    Ok(())
}
