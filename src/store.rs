use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use anyhow::{bail, Result};

pub struct Store {
    pub root: PathBuf,
}

impl Store {
    pub fn open(root: PathBuf) -> Self {
        Store { root }
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
        let start = if pass_path.extension().map(|e| e == "gpg").unwrap_or(false) {
            pass_path.parent().unwrap_or(pass_path)
        } else {
            pass_path
        };

        let mut dirs: Vec<PathBuf> = vec![];
        let mut current = start.to_path_buf();
        loop {
            dirs.push(current.clone());
            if current == self.root {
                break;
            }
            match current.parent() {
                Some(p) if p.starts_with(&self.root) || p == self.root => {
                    current = p.to_path_buf();
                }
                _ => {
                    dirs.push(self.root.clone());
                    break;
                }
            }
        }

        for dir in &dirs {
            let candidate = dir.join(".gpg-id");
            if candidate.exists() {
                return parse_gpg_id(&candidate);
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

fn parse_gpg_id(path: &Path) -> Result<Vec<String>> {
    let content = fs::read_to_string(path)?;
    let ids: Vec<String> = content
        .lines()
        .map(|l| l.trim().to_string())
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .collect();

    if ids.is_empty() {
        bail!("No GPG key IDs in {}", path.display());
    }

    Ok(ids)
}
