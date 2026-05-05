# pass-rs 🦀

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

### Pre-built binaries

Download the latest release for your platform from the [Releases](../../releases) page:

| Platform | File |
|---|---|
| Windows (x86-64) | `pass-windows-x86_64.exe` |
| Linux (x86-64, static) | `pass-linux-x86_64` |
| macOS (Apple Silicon) | `pass-macos-arm64` |
| macOS (Intel) | `pass-macos-x86_64` |

Extract and place the binary somewhere on your `PATH`.

### Install with Cargo

```bash
cargo install --git https://github.com/flejz/password-store-rs
```

### Build from source

```bash
git clone https://github.com/flejz/password-store-rs
cd password-store-rs
cargo build --release
# Binary: target/release/pass  (target/release/pass.exe on Windows)
```

### Windows notes

**Linker conflict:** Git for Windows ships a `link.exe` that shadows MSVC's. If `cargo build` fails with a linker error, prepend the MSVC bin directory to `PATH`. Find it with:
```powershell
& "${env:ProgramFiles(x86)}\Microsoft Visual Studio\Installer\vswhere.exe" -latest -products * -find "VC\Tools\MSVC\*\bin\HostX64\x64\link.exe"
```

**Adding to PATH:** copy `pass.exe` to a directory already on your `PATH` (e.g. `C:\Users\<you>\bin`), or add a new folder via *System Properties → Environment Variables*.

**GPG via Scoop:** `scoop install gpg` installs GnuPG and adds it to `PATH` automatically — no manual path configuration needed.

**Store location:** defaults to `%USERPROFILE%\.password-store` (e.g. `C:\Users\<you>\.password-store`). Override with `PASSWORD_STORE_DIR`.

**Clipboard auto-clear:** when using `--clip`, a hidden background process (`pass.exe --internal-clip-clear`) is spawned to clear the clipboard after the timeout. This is intentional — it will appear briefly in Task Manager and then exit.

**Multiline input:** use Ctrl+Z then Enter to signal EOF (instead of Ctrl+D on Unix).

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
- `--multiline` — read until EOF (Ctrl+D on Unix, Ctrl+Z + Enter on Windows)
- `--force` — overwrite without confirmation

Alias: `add`.

### `pass generate <name> [length] [--no-symbols] [--clip] [--in-place] [--force]`

Generate a random password of `length` characters (default: 25) and encrypt it. Default charset is all printable ASCII; `--no-symbols` restricts to alphanumeric only.

- `--clip` — copy to clipboard instead of printing
- `--in-place` — replace only the first line of an existing entry

### `pass rm <name> [--recursive] [--force]`

Remove a password file. Use `--recursive` to remove a directory.

Aliases: `delete`, `remove`.

### `pass find <pattern>...`

Search password names for a case-insensitive substring match.

Alias: `search`.

### `pass grep [-iE] <pattern>`

Search inside decrypted password files. Supports `-i` (ignore case) and `-E` (extended regex). Used by PassFF for URL metadata search.

### `pass edit <name>`

Decrypt a password, open it in `$EDITOR` (or `notepad.exe` on Windows), then re-encrypt. Uses a temp file in `$TMPDIR` / `%TEMP%`.

### `pass cp <old> <new> [--force]`

Copy a password. If the destination is under a different `.gpg-id`, the password is re-encrypted to the new recipients.

Alias: `copy`.

### `pass mv <old> <new> [--force]`

Move or rename a password. Re-encrypts if crossing a `.gpg-id` boundary.

Alias: `rename`.

### `pass git <args>`

Run any git command inside the password store (`git -C ~/.password-store <args>`).

---

## Shell completion

Generate and install a completion script for your shell. After installation, Tab completes subcommands, flags, and password names from your store.

```bash
# Bash — add to ~/.bashrc or drop in /etc/bash_completion.d/
pass completion bash >> ~/.bash_completion

# Zsh — add to a directory in $fpath
pass completion zsh > ~/.zfunc/_pass
# ensure ~/.zfunc is in your fpath: fpath=(~/.zfunc $fpath)
# then: autoload -Uz compinit && compinit

# Fish
pass completion fish > ~/.config/fish/completions/pass.fish

# PowerShell (Windows) — add to $PROFILE
pass completion powershell >> $PROFILE

# Elvish
pass completion elvish >> ~/.config/elvish/rc.elv
```

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

## PassFF browser extension

[PassFF](https://codeberg.org/PassFF/passff) is a Firefox extension that integrates `pass` directly into the browser for auto-fill. pass-rs is compatible with PassFF via the [passff-host](https://codeberg.org/PassFF/passff-host) native messaging host.

Supported PassFF operations:

| Operation | Status |
|---|---|
| Browse and search passwords | ✅ |
| Auto-fill username & password | ✅ |
| Insert / generate passwords | ✅ |
| URL metadata search (`pass grep`) | ✅ |
| One-time passwords (`pass otp`) | ❌ requires the `pass-otp` extension |

To use: install passff-host normally, pointing it at the `pass` binary from this project.

---

## Compatibility with the original pass

- Same `.gpg-id` file format and per-subfolder recipient resolution
- Same encrypted file format (OpenPGP binary `.gpg`)
- Same git layout and auto-commit messages
- Same environment variables (`PASSWORD_STORE_DIR`, etc.)
- **Not compatible:** `.bash` extension scripts from the original pass

---

## Differences from the original

| Feature | pass (bash) | pass-rs |
|---|---|---|
| Platform | Linux/macOS (bash required) | Windows, macOS, Linux |
| Clipboard | `xclip` / `wl-clipboard` / `pbcopy` | Native per-platform via `arboard` |
| Temp files | `/dev/shm` (RAM) | `$TMPDIR` / `%TEMP%` (disk, less secure) |
| Extensions | `.bash` scripts | Not yet implemented |
| Commands | `init ls show insert generate rm mv cp grep find edit git` | All implemented (no `otp`) |

---

## License

GPL-2.0-or-later, following the original project.
