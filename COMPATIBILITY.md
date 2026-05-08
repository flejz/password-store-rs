# Compatibility with pass (passwordstore.org)

pass-rs aims for full behavioral compatibility with [pass](https://www.passwordstore.org/) by Jason A. Donenfeld.

---

## Commands

| Command | Aliases | pass-rs | Notes |
|---|---|---|---|
| `init` | — | ✅ | Per-subfolder `.gpg-id` supported |
| `ls` | `list` | ✅ | ANSI tree output |
| `show` | — | ✅ | `--clip`, `--qrcode`, `--line N`; `pass <name>` shorthand works |
| `insert` | `add` | ✅ | `--echo`, `--multiline`, `--force` |
| `generate` | — | ✅ | `--no-symbols`, `--clip`, `--qrcode`, `--in-place`, `--force` |
| `rm` | `delete`, `remove` | ✅ | `--recursive`, `--force` |
| `find` | `search` | ✅ | Case-insensitive substring match |
| `grep` | — | ✅ | `-i`, `-E`; used by PassFF for URL search |
| `edit` | — | ✅ | Opens `$EDITOR`; temp file in `$TMPDIR`/`%TEMP%` |
| `cp` | `copy` | ✅ | Re-encrypts when crossing `.gpg-id` boundary |
| `mv` | `rename` | ✅ | Re-encrypts when crossing `.gpg-id` boundary |
| `git` | — | ✅ | Pass-through: `git -C $STORE <args>` |
| `otp` | — | ❌ | Requires the `pass-otp` extension |

---

## Store format

| Aspect | pass | pass-rs | Compatible |
|---|---|---|---|
| Encrypted file format | OpenPGP binary `.gpg` | OpenPGP binary `.gpg` | ✅ |
| Recipient file | `.gpg-id` per directory | `.gpg-id` per directory | ✅ |
| Recipient resolution | Walk up to store root | Walk up to store root | ✅ |
| Git layout | Commits per change | Commits per change | ✅ |
| Subfolder `.gpg-id` | ✅ | ✅ | ✅ |

A `.password-store` created with `pass` on Linux or macOS opens unchanged with pass-rs, and vice versa.

---

## Environment variables

| Variable | pass | pass-rs |
|---|---|---|
| `PASSWORD_STORE_DIR` | ✅ | ✅ |
| `PASSWORD_STORE_CLIP_TIME` | ✅ | ✅ |
| `PASSWORD_STORE_GPG_OPTS` | ✅ | ✅ |
| `PASSWORD_STORE_KEY` | ✅ | ✅ |
| `PASSWORD_STORE_GIT` | ✅ | ❌ |
| `PASSWORD_STORE_GENERATED_LENGTH` | ✅ | ✅ |
| `PASSWORD_STORE_CHARACTER_SET` | ✅ | ✅ |
| `PASSWORD_STORE_CHARACTER_SET_NO_SYMBOLS` | ✅ | ✅ |
| `PASSWORD_STORE_SIGNING_KEY` | ✅ | ❌ |
| `PASSWORD_STORE_ENABLE_EXTENSIONS` | ✅ | ❌ |

---

## Behaviour differences

| Feature | pass | pass-rs |
|---|---|---|
| Platform | Linux/macOS (bash required) | Windows, macOS, Linux (native binary) |
| Clipboard tool | `xclip` / `wl-clipboard` / `pbcopy` | Native Win32 / X11 / macOS via `arboard` |
| Clipboard restore | Saves and restores previous text | Restores previous plain-text only (binary content dropped) |
| Clipboard clear | Background shell sleep | Hidden detached subprocess |
| Temp files for `edit` | `/dev/shm` (RAM — more secure) | `$TMPDIR` / `%TEMP%` (disk — less secure) |
| Extension system | `.bash` scripts in `.extensions/` | Not implemented |
| QR code output | `--qrcode` flag | ✅ (`pass show -q`, `pass generate -q`) |
| GPG commit signing | `pass.signcommits` git config | ✅ |

---

## PassFF browser extension

[PassFF](https://codeberg.org/PassFF/passff) + [passff-host](https://codeberg.org/PassFF/passff-host) are fully compatible with pass-rs.

| PassFF feature | Status |
|---|---|
| Browse and search passwords | ✅ |
| Auto-fill username & password | ✅ |
| Insert / generate passwords | ✅ |
| URL metadata search (`pass grep -iE`) | ✅ |
| One-time passwords (`pass otp`) | ❌ requires `pass-otp` extension |
