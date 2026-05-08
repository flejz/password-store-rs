use std::fs;
use std::io::{self, Write};
use anyhow::{bail, Result};
use rand::Rng;
use crate::{clipboard, gpg::Gpg, qrcode, store::Store};

pub fn run(
    store: &Store,
    gpg: &Gpg,
    clip_time: u64,
    default_length: usize,
    charset_override: Option<&str>,
    charset_no_symbols_override: Option<&str>,
    name: &str,
    length: Option<usize>,
    no_symbols: bool,
    clip: bool,
    show_qr: bool,
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

    let length = length.unwrap_or(default_length);

    let charset: Vec<u8> = if no_symbols {
        match charset_no_symbols_override {
            Some(s) => charset_from_str(s),
            None => (b'0'..=b'9').chain(b'A'..=b'Z').chain(b'a'..=b'z').collect(),
        }
    } else {
        match charset_override {
            Some(s) => charset_from_str(s),
            None => (33u8..=126u8).collect(), // all printable ASCII
        }
    };

    if charset.is_empty() {
        bail!("Character set is empty — check PASSWORD_STORE_CHARACTER_SET");
    }

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

    if show_qr {
        qrcode::print_qrcode(&password)?;
    }
    if clip {
        let prev = clipboard::copy_to_clipboard(&password)?;
        clipboard::spawn_clip_clear(&prev, clip_time)?;
        println!(
            "Generated password for {} and copied to clipboard. Will clear in {} seconds.",
            name, clip_time
        );
    } else if !show_qr {
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

/// Build a charset from a literal string, keeping only printable ASCII.
pub(crate) fn charset_from_str(s: &str) -> Vec<u8> {
    let mut seen = std::collections::HashSet::new();
    s.chars()
        .filter(|c| c.is_ascii_graphic())
        .map(|c| c as u8)
        .filter(|b| seen.insert(*b))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_charset_all_printable_ascii() {
        let cs: Vec<u8> = (33u8..=126u8).collect();
        assert_eq!(cs.len(), 94);
        assert!(cs.contains(&b'!'));
        assert!(cs.contains(&b'z'));
        assert!(!cs.contains(&b' '));
    }

    #[test]
    fn no_symbols_charset_alphanumeric_only() {
        let cs: Vec<u8> = (b'0'..=b'9').chain(b'A'..=b'Z').chain(b'a'..=b'z').collect();
        assert_eq!(cs.len(), 62);
        assert!(!cs.contains(&b'!'));
        assert!(cs.contains(&b'a'));
    }

    #[test]
    fn charset_from_str_deduplicates() {
        let cs = charset_from_str("aabbcc");
        assert_eq!(cs, vec![b'a', b'b', b'c']);
    }

    #[test]
    fn charset_from_str_strips_non_printable() {
        let cs = charset_from_str("ab\x00\x01cd");
        assert_eq!(cs, vec![b'a', b'b', b'c', b'd']);
    }

    #[test]
    fn charset_from_str_strips_spaces() {
        let cs = charset_from_str("a b c");
        assert_eq!(cs, vec![b'a', b'b', b'c']);
    }

    #[test]
    fn generated_length_default_is_25() {
        // If no CLI arg and no env var, length resolves to 25
        let length: Option<usize> = None;
        let default_length = 25usize;
        assert_eq!(length.unwrap_or(default_length), 25);
    }

    #[test]
    fn cli_length_overrides_default() {
        let length: Option<usize> = Some(16);
        let default_length = 25usize;
        assert_eq!(length.unwrap_or(default_length), 16);
    }
}
