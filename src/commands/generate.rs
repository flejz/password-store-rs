use std::fs;
use std::io::{self, Write};
use anyhow::{bail, Result};
use rand::Rng;
use crate::{clipboard, gpg::Gpg, store::Store};

pub fn run(
    store: &Store,
    gpg: &Gpg,
    clip_time: u64,
    name: &str,
    length: usize,
    no_symbols: bool,
    clip: bool,
    in_place: bool,
    force: bool,
) -> Result<()> {
    store.assert_exists()?;
    let pass_file = store.pass_file(name)?;

    if pass_file.exists() && !force && !in_place {
        print!("An entry already exists for {}. Overwrite it? [y/N] ", name);
        io::stdout().flush()?;
        let mut answer = String::new();
        io::stdin().read_line(&mut answer)?;
        if !answer.trim().eq_ignore_ascii_case("y") {
            bail!("Aborted.");
        }
    }

    let charset: Vec<u8> = if no_symbols {
        (b'0'..=b'9').chain(b'A'..=b'Z').chain(b'a'..=b'z').collect()
    } else {
        // All printable ASCII 33–126 (excludes space)
        (33u8..=126u8).collect()
    };

    let mut rng = rand::thread_rng();
    let password: String = (0..length)
        .map(|_| charset[rng.gen_range(0..charset.len())] as char)
        .collect();

    let content = if in_place && pass_file.exists() {
        let existing = gpg.decrypt(&pass_file)?;
        let existing_str = String::from_utf8_lossy(&existing);
        let lines: Vec<&str> = existing_str.lines().collect();
        if lines.len() > 1 {
            format!("{}\n{}\n", password, lines[1..].join("\n"))
        } else {
            format!("{}\n", password)
        }
    } else {
        format!("{}\n", password)
    };

    if let Some(parent) = pass_file.parent() {
        fs::create_dir_all(parent)?;
    }

    let recipients = store.recipients_for(&pass_file)?;
    gpg.encrypt(content.as_bytes(), &recipients, &pass_file)?;

    if clip {
        let prev = clipboard::copy_to_clipboard(&password)?;
        clipboard::spawn_clip_clear(&prev, clip_time)?;
        println!(
            "Generated password for {} and copied to clipboard. Will clear in {} seconds.",
            name, clip_time
        );
    } else {
        println!("Generated password for {}:", name);
        println!("{}", password);
    }

    let msg = if in_place {
        format!("Replace generated password for {}.", name)
    } else {
        format!("Add generated password for {}.", name)
    };
    store.git_commit(&pass_file, &msg);
    Ok(())
}
