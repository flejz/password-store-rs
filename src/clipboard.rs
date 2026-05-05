use anyhow::Result;
use base64::{engine::general_purpose::STANDARD, Engine};
use std::time::Duration;

/// Copy `text` to clipboard, returning the previous clipboard text (for restore).
pub fn copy_to_clipboard(text: &str) -> Result<String> {
    let mut cb = arboard::Clipboard::new()?;
    let prev = cb.get_text().unwrap_or_default();
    cb.set_text(text)?;
    Ok(prev)
}

/// Spawn a detached, hidden subprocess that clears the clipboard after `secs` seconds.
/// The previous clipboard text is passed as base64 so it can be restored.
pub fn spawn_clip_clear(prev: &str, secs: u64) -> Result<()> {
    let exe = std::env::current_exe()?;
    let prev_b64 = STANDARD.encode(prev.as_bytes());

    let mut cmd = std::process::Command::new(exe);
    cmd.args(["--internal-clip-clear", &prev_b64, &secs.to_string()]);

    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const DETACHED_PROCESS: u32 = 0x0000_0008;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        cmd.creation_flags(DETACHED_PROCESS | CREATE_NO_WINDOW);
    }

    cmd.spawn()?;
    Ok(())
}

/// Called inside the detached subprocess: sleep, then restore previous clipboard text.
/// Note: only plain-text clipboard is saved/restored; binary content (images) is dropped.
pub fn internal_clip_clear(prev_b64: &str, secs: u64) -> Result<()> {
    std::thread::sleep(Duration::from_secs(secs));

    let prev_bytes = STANDARD.decode(prev_b64).unwrap_or_default();
    let prev = String::from_utf8(prev_bytes).unwrap_or_default();

    if let Ok(mut cb) = arboard::Clipboard::new() {
        let _ = cb.set_text(&prev);
    }

    Ok(())
}
