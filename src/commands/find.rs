use anyhow::Result;
use crate::store::Store;

const BOLD_BLUE: &str = "\x1b[1;34m";
const RESET: &str = "\x1b[0m";

pub fn run(store: &Store, patterns: &[String]) -> Result<()> {
    store.assert_exists()?;

    println!(
        "{}Search Terms:{} {}",
        BOLD_BLUE,
        RESET,
        patterns.join(", ")
    );

    let lower_patterns: Vec<String> = patterns.iter().map(|p| p.to_lowercase()).collect();

    let mut found = false;
    for entry in walkdir::WalkDir::new(&store.root)
        .sort_by_file_name()
        .into_iter()
        .filter_map(|e| e.ok())
        .skip(1) // skip root itself
    {
        let path = entry.path();
        let raw_name = entry.file_name().to_string_lossy();

        let stem = if path.is_file() {
            match raw_name.strip_suffix(".gpg") {
                Some(s) => s.to_string(),
                None => continue,
            }
        } else {
            raw_name.to_string()
        };

        if lower_patterns.iter().any(|p| stem.to_lowercase().contains(p.as_str())) {
            let rel = path.strip_prefix(&store.root).unwrap_or(path);
            let display = rel.to_string_lossy()
                .replace('\\', "/")
                .trim_end_matches(".gpg")
                .to_string();

            if path.is_dir() {
                println!("  {}{}{}", BOLD_BLUE, display, RESET);
            } else {
                println!("  {}", display);
            }
            found = true;
        }
    }

    if !found {
        println!("  (none)");
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    // Pure pattern-matching logic extracted for testing
    fn matches(stem: &str, patterns: &[&str]) -> bool {
        let lower = stem.to_lowercase();
        patterns.iter().any(|p| lower.contains(&p.to_lowercase()))
    }

    #[test]
    fn matches_substring() {
        assert!(matches("gmail", &["mail"]));
    }

    #[test]
    fn matches_case_insensitive() {
        assert!(matches("Gmail", &["gmail"]));
        assert!(matches("gmail", &["Gmail"]));
        assert!(matches("GMAIL", &["gmail"]));
    }

    #[test]
    fn no_match_returns_false() {
        assert!(!matches("twitter", &["gmail"]));
    }

    #[test]
    fn matches_any_pattern() {
        assert!(matches("twitter", &["gmail", "twitter"]));
        assert!(!matches("facebook", &["gmail", "twitter"]));
    }

    #[test]
    fn matches_email_stem() {
        // Simulate email-as-filename: stem is everything before .gpg
        // e.g. "user@gmail.com.gpg" → stem "user@gmail.com"
        assert!(matches("user@gmail.com", &["gmail"]));
        assert!(!matches("user@gmail.com", &["yahoo"]));
    }
}
