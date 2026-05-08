use std::fs;
use std::io::{self, Write};
use anyhow::Result;
use crate::{gpg::Gpg, store::Store};

pub fn run(store: &Store, gpg: &Gpg, gpg_ids: &[String], path: Option<&str>) -> Result<()> {
    for id in gpg_ids {
        if !gpg.key_exists(id) {
            eprintln!("Warning: GPG key not found in keyring: {}", id);
        }
    }

    let target_dir = if let Some(p) = path {
        store.root.join(p)
    } else {
        store.root.clone()
    };

    fs::create_dir_all(&target_dir)?;

    let gpg_id_file = target_dir.join(".gpg-id");
    fs::write(&gpg_id_file, gpg_ids.join("\n") + "\n")?;

    println!(
        "Password store initialized for {}{} .",
        gpg_ids.join(", "),
        path.map(|p| format!(" ({})", p)).unwrap_or_default()
    );

    // Offer to re-encrypt existing passwords when changing root key
    if path.is_none() {
        let has_passwords = walkdir::WalkDir::new(&store.root)
            .into_iter()
            .filter_map(|e| e.ok())
            .any(|e| e.path().extension().map(|x| x == "gpg").unwrap_or(false));

        if has_passwords {
            print!("Re-encrypt existing passwords with new key(s)? [y/N] ");
            io::stdout().flush()?;
            let mut answer = String::new();
            io::stdin().read_line(&mut answer)?;
            if answer.trim().eq_ignore_ascii_case("y") {
                reencrypt_all(store, gpg)?;
            }
        }
    }

    // Sign .gpg-id with PASSWORD_STORE_SIGNING_KEY if configured
    if let Some(ref key) = store.signing_key {
        gpg.sign_detach(&gpg_id_file, key)?;
        let sig_file = target_dir.join(".gpg-id.sig");
        store.git_commit(&sig_file, &format!("Set GPG id to {}.", gpg_ids.join(", ")));
    }

    store.git_commit(&gpg_id_file, &format!("Set GPG id to {}.", gpg_ids.join(", ")));
    Ok(())
}

fn reencrypt_all(store: &Store, gpg: &Gpg) -> Result<()> {
    for entry in walkdir::WalkDir::new(&store.root)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map(|x| x == "gpg").unwrap_or(false))
    {
        let path = entry.path();
        let recipients = store.recipients_for(path)?;
        let data = gpg.decrypt(path)?;
        gpg.encrypt(&data, &recipients, path)?;
        println!("Re-encrypted: {}", path.display());
    }
    Ok(())
}
