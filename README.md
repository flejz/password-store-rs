# pass-win

A cross-platform Rust implementation of [pass — the standard Unix password manager](https://www.passwordstore.org/) by Jason A. Donenfeld.

The original `pass` is a 700-line bash script with hard dependencies on MSYS2/Cygwin/WSL, `xclip`, `tree`, and other POSIX tools. This rewrite is a single native binary with no shell interpreter required — works on Windows, macOS, and Linux.

**Store format is fully compatible.** A `.password-store` created with the original `pass` opens unchanged here, and vice versa.

---

## Requirements

- [GnuPG](https://gnupg.org) — provides `gpg` for encryption/decryption
  - Windows: [Gpg4win](https://gpg4win.org)
  - macOS: `brew install gnupg`
  - Linux: `apt install gnupg` / `pacman -S gnupg`
- [Git](https://git-scm.com) *(optional)* — for auto-commit on changes

`gpg` is located automatically via:
1. `PATH`
2. `C:\Program Files (x86)\GnuPG\bin\gpg.exe` (Gpg4win default, Windows only)
3. `C:\Program Files\GnuPG\bin\gpg.exe` (Windows only)
4. `~\scoop\shims\gpg.exe` (Scoop, Windows only)

---

## Installation

Build from source:

```bash
cargo build --release
# Binary: target/release/pass  (target/release/pass.exe on Windows)
```

Place the binary somewhere on your `PATH`.

---

## Quick start

```bash
# Initialize the store with your GPG key
pass init <your-gpg-key-id>

# Insert a password
pass insert email/gmail

# Show a password
pass show email/gmail

# Copy password to clipboard (clears after 45 seconds)
pass show --clip email/gmail

# Generate a random 25-character password
pass generate email/work 25

# List all passwords
pass ls

# Remove a password
pass rm email/gmail
```

---

## Commands

### `pass init <gpg-id>... [--path <subfolder>]`

Initialize the password store. Creates `~/.password-store/.gpg-id` containing the given GPG key ID(s). If passwords already exist, offers to re-encrypt them with the new key(s).

Use `--path` to set a per-subfolder key:

```bash
pass init --path work/ 0xABCD1234
```

### `pass ls [subfolder]`

List passwords as a tree. Aliases: `list`.

```
Password Store
├── email
│   ├── gmail
│   └── work
└── social
    └── twitter
```

### `pass show <name> [--clip] [--line N]`

Decrypt and print a password. With `--clip`, copy line N (default: 1) to the clipboard instead. The clipboard is automatically cleared after 45 seconds.

If `<name>` is a subfolder, lists its contents.

### `pass insert <name> [--echo] [--multiline] [--force]`

Insert a new password. Prompts twice (no echo) by default.

- `--echo` — show typed characters
- `--multiline` — read until EOF (Ctrl+Z on Windows, Ctrl+D elsewhere)
- `--force` — overwrite without confirmation

Alias: `add`.

### `pass generate <name> [length] [--no-symbols] [--clip] [--in-place] [--force]`

Generate a random password of `length` characters (default: 25) and encrypt it. Default charset is all printable ASCII; `--no-symbols` restricts to alphanumeric only.

- `--clip` — copy to clipboard instead of printing
- `--in-place` — replace only the first line of an existing entry

### `pass rm <name> [--recursive] [--force]`

Remove a password file. Use `--recursive` to remove a directory.

Aliases: `delete`, `remove`.

---

## Environment variables

| Variable | Default | Description |
|---|---|---|
| `PASSWORD_STORE_DIR` | `~/.password-store` | Path to the password store |
| `PASSWORD_STORE_CLIP_TIME` | `45` | Seconds before clipboard is cleared |
| `PASSWORD_STORE_GPG_OPTS` | *(empty)* | Extra flags passed to `gpg` |

---

## Git integration

If the store directory contains a `.git` repo, mutating commands (`insert`, `generate`, `rm`, `init`) automatically stage and commit the affected file with a descriptive message.

Initialize a git repo in your store:

```bash
cd ~/.password-store
git init
git remote add origin <url>
git push -u origin master
```

---

## Compatibility with the original pass

- Same `.gpg-id` file format and per-subfolder recipient resolution
- Same encrypted file format (OpenPGP binary `.gpg`)
- Same git layout and auto-commit messages
- Same environment variables (`PASSWORD_STORE_DIR`, etc.)
- **Not compatible:** `.bash` extension scripts from the original pass

---

## Differences from the original

| Feature | pass (bash) | pass-win |
|---|---|---|
| Platform | Linux/macOS (bash required) | Windows, macOS, Linux |
| Clipboard | `xclip` / `wl-clipboard` / `pbcopy` | Native per-platform via `arboard` |
| Temp files | `/dev/shm` (RAM) | `$TMPDIR` / `%TEMP%` (disk, less secure) |
| Extensions | `.bash` scripts | Not yet implemented |
| Commands | `init ls show insert generate rm mv cp grep find edit git` | `init ls show insert generate rm` (Phase 1) |

---

## License

GPL-2.0-or-later, following the original project.
