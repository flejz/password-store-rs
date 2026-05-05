use anyhow::{bail, Result};
use regex::RegexBuilder;
use crate::{gpg::Gpg, store::Store};

const BOLD: &str = "\x1b[1m";
const RESET: &str = "\x1b[0m";

pub(crate) struct GrepOpts {
    pub case_insensitive: bool,
    pub pattern: String,
}

pub(crate) fn parse_args(args: &[String]) -> Result<GrepOpts> {
    let mut case_insensitive = false;
    let mut end_of_flags = false;
    let mut pattern: Option<&str> = None;

    for arg in args {
        if end_of_flags {
            pattern = Some(arg);
        } else if arg == "--" {
            end_of_flags = true;
        } else if arg.starts_with('-') && arg.len() > 1 && !arg.starts_with("--") {
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

    let pattern = pattern
        .ok_or_else(|| anyhow::anyhow!("Usage: pass grep [-iE] <pattern>"))?
        .to_string();

    Ok(GrepOpts { case_insensitive, pattern })
}

pub fn run(store: &Store, gpg: &Gpg, args: &[String]) -> Result<()> {
    store.assert_exists()?;

    let opts = parse_args(args)?;

    let re = RegexBuilder::new(&opts.pattern)
        .case_insensitive(opts.case_insensitive)
        .build()
        .map_err(|e| anyhow::anyhow!("Invalid pattern '{}': {}", opts.pattern, e))?;

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
        std::process::exit(1);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(v: &[&str]) -> Vec<String> {
        v.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn plain_pattern() {
        let opts = parse_args(&args(&["url:"])).unwrap();
        assert_eq!(opts.pattern, "url:");
        assert!(!opts.case_insensitive);
    }

    #[test]
    fn short_i_flag() {
        let opts = parse_args(&args(&["-i", "url:"])).unwrap();
        assert!(opts.case_insensitive);
        assert_eq!(opts.pattern, "url:");
    }

    #[test]
    fn combined_ie_flag() {
        let opts = parse_args(&args(&["-iE", "^url:"])).unwrap();
        assert!(opts.case_insensitive);
        assert_eq!(opts.pattern, "^url:");
    }

    #[test]
    fn double_dash_separator() {
        let opts = parse_args(&args(&["-i", "--", "-not-a-flag"])).unwrap();
        assert!(opts.case_insensitive);
        assert_eq!(opts.pattern, "-not-a-flag");
    }

    #[test]
    fn long_ignore_case_flag() {
        let opts = parse_args(&args(&["--ignore-case", "URL"])).unwrap();
        assert!(opts.case_insensitive);
    }

    #[test]
    fn passff_style_invocation() {
        let opts = parse_args(&args(&["-iE", "^(url|username|login):"])).unwrap();
        assert!(opts.case_insensitive);
        assert_eq!(opts.pattern, "^(url|username|login):");
    }

    #[test]
    fn missing_pattern_errors() {
        assert!(parse_args(&args(&["-i"])).is_err());
    }

    #[test]
    fn unknown_flag_errors() {
        assert!(parse_args(&args(&["-x", "pattern"])).is_err());
    }
}
