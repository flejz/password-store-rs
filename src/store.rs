use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use anyhow::{bail, Result};

pub struct Store {
    pub root: PathBuf,
    /// When set, overrides per-directory .gpg-id for all encrypt operations.
    pub key_override: Option<Vec<String>>,
}

impl Store {
    pub fn open(root: PathBuf) -> Self {
        Store { root, key_override: None }
    }

    pub fn with_key_override(mut self, keys: Option<Vec<String>>) -> Self {
        self.key_override = keys;
        self
    }

    pub fn assert_exists(&self) -> Result<()> {
        if !self.root.exists() {
            bail!(
                "Password store not initialized at {}. Run: pass init <gpg-id>",
                self.root.display()
            );
        }
        if !self.root.join(".gpg-id").exists() {
            bail!(
                "No .gpg-id in {}. Run: pass init <gpg-id>",
                self.root.display()
            );
        }
        Ok(())
    }

    /// Resolve `name` to its .gpg file path, validating it stays within the store.
    pub fn pass_file(&self, name: &str) -> Result<PathBuf> {
        if name.contains("..") {
            bail!("Sneaky path component: {}", name);
        }
        let name = name.trim_matches('/').trim_matches('\\');
        // Append .gpg as a suffix — never use set_extension(), which would
        // replace an existing extension like .com in `user@gmail.com`.
        let full = if name.ends_with(".gpg") {
            name.to_string()
        } else {
            format!("{}.gpg", name)
        };
        let path = self.root.join(&full);
        if !path.starts_with(&self.root) {
            bail!("Path escapes store: {}", path.display());
        }
        Ok(path)
    }

    /// Find the GPG recipients for a given password file by walking up to store root.
    pub fn recipients_for(&self, pass_path: &Path) -> Result<Vec<String>> {
        if let Some(keys) = &self.key_override {
            return Ok(keys.clone());
        }

        let start = if pass_path.extension().map(|e| e == "gpg").unwrap_or(false) {
            pass_path.parent().unwrap_or(pass_path)
        } else {
            pass_path
        };

        // Walk up from start to root, checking for .gpg-id at each level.
        // No intermediate Vec — check on each iteration.
        let mut current = start.to_path_buf();
        loop {
            let candidate = current.join(".gpg-id");
            if candidate.exists() {
                return parse_gpg_id(&candidate);
            }
            if current == self.root {
                break;
            }
            match current.parent() {
                Some(p) if p.starts_with(&self.root) || p == self.root => {
                    current = p.to_path_buf();
                }
                _ => break,
            }
        }

        bail!("No .gpg-id found. Run: pass init <gpg-id>")
    }

    /// Stage and commit a change to the store's git repo (no-op if not a git repo).
    pub fn git_commit(&self, path: &Path, message: &str) {
        let git_dir = self.root.join(".git");
        if !git_dir.exists() {
            return;
        }

        let store_str = match self.root.to_str() {
            Some(s) => s,
            None => return,
        };

        let rel = match path.strip_prefix(&self.root) {
            Ok(r) => r,
            Err(_) => return,
        };

        if path.exists() {
            let _ = Command::new("git")
                .args(["-C", store_str, "add", "--"])
                .arg(rel)
                .status();
        } else {
            let _ = Command::new("git")
                .args(["-C", store_str, "rm", "-rf", "--"])
                .arg(rel)
                .status();
        }

        let _ = Command::new("git")
            .args(["-C", store_str, "commit", "-m", message])
            .status();
    }
}

pub(crate) fn parse_gpg_id(path: &Path) -> Result<Vec<String>> {
    let content = fs::read_to_string(path)?;
    let ids: Vec<String> = content
        .lines()
        .filter(|l| {
            let t = l.trim();
            !t.is_empty() && !t.starts_with('#')
        })
        .map(|l| l.trim().to_string())
        .collect();

    if ids.is_empty() {
        bail!("No GPG key IDs in {}", path.display());
    }

    Ok(ids)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn make_store() -> (TempDir, Store) {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join(".gpg-id"), "test@example.com\n").unwrap();
        let store = Store::open(dir.path().to_path_buf());
        (dir, store)
    }

    // pass_file — basic name
    #[test]
    fn pass_file_simple() {
        let (_dir, store) = make_store();
        let p = store.pass_file("email/gmail").unwrap();
        assert_eq!(p.file_name().unwrap().to_string_lossy(), "gmail.gpg");
    }

    // pass_file — email filename: .com must NOT be replaced by .gpg
    #[test]
    fn pass_file_preserves_dotcom_extension() {
        let (_dir, store) = make_store();
        let p = store.pass_file("wise.com/user@gmail.com").unwrap();
        assert_eq!(
            p.file_name().unwrap().to_string_lossy(),
            "user@gmail.com.gpg"
        );
    }

    // pass_file — leading slash stripped (PassFF sends /keyname)
    #[test]
    fn pass_file_strips_leading_slash() {
        let (_dir, store) = make_store();
        let with_slash = store.pass_file("/email/gmail").unwrap();
        let without = store.pass_file("email/gmail").unwrap();
        assert_eq!(with_slash, without);
    }

    // pass_file — already has .gpg extension: no double extension
    #[test]
    fn pass_file_no_double_gpg() {
        let (_dir, store) = make_store();
        let p = store.pass_file("email/gmail.gpg").unwrap();
        assert_eq!(p.file_name().unwrap().to_string_lossy(), "gmail.gpg");
    }

    // pass_file — directory traversal blocked
    #[test]
    fn pass_file_rejects_dotdot() {
        let (_dir, store) = make_store();
        assert!(store.pass_file("../etc/passwd").is_err());
    }

    // parse_gpg_id — basic parsing
    #[test]
    fn parse_gpg_id_basic() {
        let dir = tempfile::tempdir().unwrap();
        let f = dir.path().join(".gpg-id");
        fs::write(&f, "keyA\nkeyB\n").unwrap();
        let ids = parse_gpg_id(&f).unwrap();
        assert_eq!(ids, vec!["keyA", "keyB"]);
    }

    // parse_gpg_id — strips comments and blank lines
    #[test]
    fn parse_gpg_id_strips_comments() {
        let dir = tempfile::tempdir().unwrap();
        let f = dir.path().join(".gpg-id");
        fs::write(&f, "# comment\n\nkeyA\n").unwrap();
        let ids = parse_gpg_id(&f).unwrap();
        assert_eq!(ids, vec!["keyA"]);
    }

    // parse_gpg_id — empty file errors
    #[test]
    fn parse_gpg_id_empty_errors() {
        let dir = tempfile::tempdir().unwrap();
        let f = dir.path().join(".gpg-id");
        fs::write(&f, "# only comments\n").unwrap();
        assert!(parse_gpg_id(&f).is_err());
    }

    // recipients_for — root .gpg-id used when no subdir .gpg-id
    #[test]
    fn recipients_for_falls_back_to_root() {
        let (_dir, store) = make_store();
        let pass_path = store.root.join("email").join("gmail.gpg");
        let ids = store.recipients_for(&pass_path).unwrap();
        assert_eq!(ids, vec!["test@example.com"]);
    }

    // recipients_for — subdir .gpg-id takes precedence
    #[test]
    fn recipients_for_prefers_subdir() {
        let (dir, store) = make_store();
        let subdir = dir.path().join("work");
        fs::create_dir_all(&subdir).unwrap();
        fs::write(subdir.join(".gpg-id"), "work-key\n").unwrap();
        let pass_path = subdir.join("secret.gpg");
        let ids = store.recipients_for(&pass_path).unwrap();
        assert_eq!(ids, vec!["work-key"]);
    }

    // key_override — PASSWORD_STORE_KEY overrides .gpg-id
    #[test]
    fn key_override_takes_precedence_over_gpg_id() {
        let (_dir, store) = make_store();
        let store = store.with_key_override(Some(vec!["override-key".to_string()]));
        let pass_path = store.root.join("email").join("gmail.gpg");
        let ids = store.recipients_for(&pass_path).unwrap();
        assert_eq!(ids, vec!["override-key"]);
    }

    // key_override — multiple keys
    #[test]
    fn key_override_supports_multiple_keys() {
        let (_dir, store) = make_store();
        let keys = vec!["key1".to_string(), "key2".to_string()];
        let store = store.with_key_override(Some(keys.clone()));
        let pass_path = store.root.join("test.gpg");
        assert_eq!(store.recipients_for(&pass_path).unwrap(), keys);
    }

    // no key_override — falls back to .gpg-id
    #[test]
    fn no_key_override_uses_gpg_id() {
        let (_dir, store) = make_store();
        let pass_path = store.root.join("test.gpg");
        let ids = store.recipients_for(&pass_path).unwrap();
        assert_eq!(ids, vec!["test@example.com"]);
    }
}
