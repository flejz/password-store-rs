use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use anyhow::{anyhow, bail, Context, Result};

pub struct Gpg {
    path: PathBuf,
    extra_opts: Vec<String>,
}

impl Gpg {
    pub fn find(extra_opts: Vec<String>) -> Result<Self> {
        let path = find_gpg_binary()?;
        Ok(Gpg { path, extra_opts })
    }

    pub fn decrypt(&self, file: &Path) -> Result<Vec<u8>> {
        let output = Command::new(&self.path)
            .args(["--quiet", "--yes", "--compress-algo=none", "--no-encrypt-to", "-d"])
            .args(&self.extra_opts)
            .arg(file)
            .output()
            .context("Failed to run gpg")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            bail!("GPG decryption failed: {}", stderr.trim());
        }

        Ok(output.stdout)
    }

    pub fn encrypt(&self, data: &[u8], recipients: &[String], out: &Path) -> Result<()> {
        let mut args: Vec<String> = vec![
            "--quiet".into(),
            "--yes".into(),
            "--compress-algo=none".into(),
            "--no-encrypt-to".into(),
            "--batch".into(),
            "--trust-model".into(),
            "always".into(),
        ];

        for r in recipients {
            args.push("-r".into());
            args.push(format_key_id(r));
        }

        args.extend_from_slice(&self.extra_opts);
        args.extend(["-e".into(), "-o".into(), out.to_string_lossy().into_owned()]);

        let mut child = Command::new(&self.path)
            .args(&args)
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::inherit())
            .spawn()
            .context("Failed to run gpg")?;

        // Write before wait, but always wait even if write fails so the
        // GPG subprocess is not abandoned as a zombie.
        let write_result = if let Some(mut stdin) = child.stdin.take() {
            stdin.write_all(data)
        } else {
            Ok(())
        };

        let status = child.wait()?;
        write_result.context("Failed to write plaintext to gpg stdin")?;

        if !status.success() {
            bail!("GPG encryption failed");
        }

        Ok(())
    }

    pub fn key_exists(&self, key_id: &str) -> bool {
        Command::new(&self.path)
            .args(["--list-keys", &format_key_id(key_id)])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
    }
}

/// Wrap email-style key IDs in angle brackets so GPG resolves them correctly
/// on all platforms (e.g. `user@example.com` → `<user@example.com>`).
pub(crate) fn format_key_id(id: &str) -> String {
    if id.contains('@') && !id.starts_with('<') {
        format!("<{}>", id)
    } else {
        id.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn email_key_gets_angle_brackets() {
        assert_eq!(format_key_id("user@example.com"), "<user@example.com>");
    }

    #[test]
    fn already_bracketed_unchanged() {
        assert_eq!(format_key_id("<user@example.com>"), "<user@example.com>");
    }

    #[test]
    fn fingerprint_unchanged() {
        assert_eq!(format_key_id("0xABCD1234"), "0xABCD1234");
    }

    #[test]
    fn short_key_id_unchanged() {
        assert_eq!(format_key_id("ABCD1234"), "ABCD1234");
    }

    #[test]
    fn email_with_subdomain_gets_angle_brackets() {
        assert_eq!(
            format_key_id("jaimelopesflores@gmail.com"),
            "<jaimelopesflores@gmail.com>"
        );
    }
}

fn find_gpg_binary() -> Result<PathBuf> {
    if let Ok(p) = which::which("gpg") {
        return Ok(p);
    }

    for base in [r"C:\Program Files (x86)\GnuPG", r"C:\Program Files\GnuPG"] {
        let p = Path::new(base).join("bin").join("gpg.exe");
        if p.exists() {
            return Ok(p);
        }
    }

    if let Some(home) = dirs::home_dir() {
        let scoop = home.join("scoop").join("shims").join("gpg.exe");
        if scoop.exists() {
            return Ok(scoop);
        }
    }

    Err(anyhow!(
        "gpg not found. Install Gpg4win from https://gpg4win.org or add gpg to your PATH."
    ))
}
