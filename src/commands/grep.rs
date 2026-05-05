use anyhow::{bail, Result};
use regex::RegexBuilder;
use crate::{gpg::Gpg, store::Store};

const BOLD: &str = "\x1b[1m";
const RESET: &str = "\x1b[0m";

pub fn run(store: &Store, gpg: &Gpg, args: &[String]) -> Result<()> {
    store.assert_exists()?;

    // Parse grep-style flags: -i, -E (no-op), combined -iE, -- separator, then pattern
    let mut case_insensitive = false;
    let mut end_of_flags = false;
    let mut pattern: Option<&str> = None;

    for arg in args {
        if end_of_flags {
            pattern = Some(arg);
        } else if arg == "--" {
            end_of_flags = true;
        } else if arg.starts_with('-') && arg.len() > 1 && !arg.starts_with("--") {
            // Single-dash flags, possibly combined: -iE
            for ch in arg[1..].chars() {
                match ch {
                    'i' => case_insensitive = true,
                    'E' => {} // always ER in Rust regex
                    other => bail!("Unsupported grep flag: -{}", other),
                }
            }
        } else if arg == "--ignore-case" {
            case_insensitive = true;
        } else if arg == "--extended-regexp" {
            // no-op
        } else {
            pattern = Some(arg);
        }
    }

    let pattern = pattern.ok_or_else(|| anyhow::anyhow!("Usage: pass grep [-iE] <pattern>"))?;

    let re = RegexBuilder::new(pattern)
        .case_insensitive(case_insensitive)
        .build()
        .map_err(|e| anyhow::anyhow!("Invalid pattern '{}': {}", pattern, e))?;

    let mut found = false;

    for entry in walkdir::WalkDir::new(&store.root)
        .sort_by_file_name()
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map(|x| x == "gpg").unwrap_or(false))
    {
        let path = entry.path();
        let rel = match path.strip_prefix(&store.root) {
            Ok(r) => r,
            Err(_) => continue,
        };

        // Normalise path separators and strip .gpg
        let display = rel.to_string_lossy()
            .replace('\\', "/")
            .trim_end_matches(".gpg")
            .to_string();

        let data = match gpg.decrypt(path) {
            Ok(d) => d,
            Err(_) => continue,
        };

        let text = String::from_utf8_lossy(&data);
        let matches: Vec<&str> = text.lines().filter(|l| re.is_match(l)).collect();

        if !matches.is_empty() {
            println!("{}{}:{}", BOLD, display, RESET);
            for m in matches {
                println!("  {}", m);
            }
            found = true;
        }
    }

    if !found {
        // grep exits 1 when no matches — callers (like PassFF) rely on this
        std::process::exit(1);
    }

    Ok(())
}
