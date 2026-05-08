use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use anyhow::{bail, Result};

pub struct Store {
    pub root: PathBuf,
    /// Overrides per-directory .gpg-id ($PASSWORD_STORE_KEY)
    pub key_override: Option<Vec<String>>,
    /// Alternative git directory ($PASSWORD_STORE_GIT)
    pub git_dir: Option<PathBuf>,
    /// GPG key ID for signing/verifying .gpg-id ($PASSWORD_STORE_SIGNING_KEY)
    pub signing_key: Option<String>,
}

impl Store {
    pub fn open(root: PathBuf) -> Self {
        Store { root, key_override: None, git_dir: None, signing_key: None }
    }

    pub fn with_key_override(mut self, keys: Option<Vec<String>>) -> Self {
        self.key_override = keys;
        self
    }

    pub fn with_git_dir(mut self, git_dir: Option<PathBuf>) -> Self {
        self.git_dir = git_dir;
        self
    }

    pub fn with_signing_key(mut self, key: Option<String>) -> Self {
        self.signing_key = key;
        self
    }

    pub fn assert_exists(&self) -> Result<()> {
        if !self.root.exists() {
            bail!(
                "Password store not initialized at {}. Run: pass init <gpg-id>",
                self.root.display()
            );
        }
        let gpg_id = self.root.join(".gpg-id");
        if !gpg_id.exists() {
            bail!(
                "No .gpg-id in {}. Run: pass init <gpg-id>",
                self.root.display()
            );
        }

        // Verify .gpg-id signature if signing key is configured
        let sig_file = self.root.join(".gpg-id.sig");
        if self.signing_key.is_some() && sig_file.exists() {
            let gpg = which::which("gpg").unwrap_or_else(|_| std::path::PathBuf::from("gpg"));
            let verified = Command::new(gpg)
                .args(["--quiet", "--verify"])
                .arg(&sig_file)
                .arg(&gpg_id)
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status()
                .map(|s| s.success())
                .unwrap_or(false);
            if !verified {
                bail!(
                    "GPG verification of .gpg-id failed — possible tampering detected. \
                     If you intentionally changed the signing key, re-run: pass init <gpg-id>"
                );
            }
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
        // Use PASSWORD_STORE_GIT override or fall back to $STORE/.git
        let effective_git_dir = self.git_dir.clone()
            .unwrap_or_else(|| self.root.join(".git"));
        if !effective_git_dir.exists() {
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

        let git_env: Option<(&str, &str)> = self.git_dir.as_ref()
            .and_then(|d| d.to_str().map(|s| ("GIT_DIR", s)));

        let mut add_cmd = Command::new("git");
        add_cmd.args(["-C", store_str]);
        if let Some((k, v)) = git_env { add_cmd.env(k, v); }

        if path.exists() {
            let _ = add_cmd.args(["add", "--"]).arg(rel).status();
        } else {
            let _ = add_cmd.args(["rm", "-rf", "--"]).arg(rel).status();
        }

        // Respect `git config pass.signcommits true`
        let mut cfg_cmd = Command::new("git");
        cfg_cmd.args(["-C", store_str, "config", "--local", "pass.signcommits"]);
        if let Some((k, v)) = git_env { cfg_cmd.env(k, v); }
        let sign = cfg_cmd.output()
            .map(|o| String::from_utf8_lossy(&o.stdout).trim() == "true")
            .unwrap_or(false);

        let mut commit_args = vec!["-C", store_str, "commit"];
        if sign { commit_args.push("-S"); }
        commit_args.extend(["-m", message]);

        let mut commit_cmd = Command::new("git");
        commit_cmd.args(&commit_args);
        if let Some((k, v)) = git_env { commit_cmd.env(k, v); }
        let _ = commit_cmd.status();
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

    // git_commit is a no-op when no .git directory exists (no panic)
    #[test]
    fn git_commit_no_op_without_git_dir() {
        let (_dir, store) = make_store();
        let path = store.root.join("test.gpg");
        store.git_commit(&path, "Test commit.");
    }

    // with_git_dir sets the git_dir field
    #[test]
    fn with_git_dir_sets_field() {
        let (_dir, store) = make_store();
        let gd = PathBuf::from("/tmp/my.git");
        let store = store.with_git_dir(Some(gd.clone()));
        assert_eq!(store.git_dir, Some(gd));
    }

    // with_signing_key sets the signing_key field
    #[test]
    fn with_signing_key_sets_field() {
        let (_dir, store) = make_store();
        let store = store.with_signing_key(Some("signing-key".to_string()));
        assert_eq!(store.signing_key.as_deref(), Some("signing-key"));
    }

    // assert_exists succeeds when no .gpg-id.sig present (no signing key check)
    #[test]
    fn assert_exists_no_sig_no_check() {
        let (_dir, store) = make_store();
        let store = store.with_signing_key(Some("some-key".to_string()));
        // No .gpg-id.sig file — should pass without running gpg
        assert!(store.assert_exists().is_ok());
    }
}
