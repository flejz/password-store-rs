use anyhow::{bail, Result};
use crate::{clipboard, gpg::Gpg, qrcode, store::Store, tree};

pub fn run(
    store: &Store,
    gpg: &Gpg,
    clip_time: u64,
    name: &str,
    clip: bool,
    show_qr: bool,
    line_num: usize,
) -> Result<()> {
    store.assert_exists()?;

    // Bare "/" or empty → list root (PassFF sends `pass show /` for the root)
    let name = name.trim_matches('/').trim_matches('\\');
    if name.is_empty() {
        tree::print_tree(&store.root, None);
        return Ok(());
    }

    let pass_file = store.pass_file(name)?;

    if !pass_file.exists() {
        // If it's a subfolder, list it instead
        let subdir = store.root.join(name);
        if subdir.is_dir() {
            tree::print_tree(&store.root, Some(name));
            return Ok(());
        }
        bail!("{} is not in the password store.", name);
    }

    let data = gpg.decrypt(&pass_file)?;
    let text = String::from_utf8_lossy(&data);

    if clip || show_qr {
        let lines: Vec<&str> = text.lines().collect();
        let idx = line_num.saturating_sub(1);
        let password = lines
            .get(idx)
            .ok_or_else(|| anyhow::anyhow!("There is no line {} in {}.", line_num, name))?;

        if show_qr {
            qrcode::print_qrcode(password)?;
        }
        if clip {
            let prev = clipboard::copy_to_clipboard(password)?;
            clipboard::spawn_clip_clear(&prev, clip_time)?;
            println!(
                "Copied {} to clipboard. Will clear in {} seconds.",
                name, clip_time
            );
        }
    } else {
        print!("{}", text);
    }

    Ok(())
}
