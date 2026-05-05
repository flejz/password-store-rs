mod cli;
mod clipboard;
mod commands;
mod config;
mod gpg;
mod store;
mod tree;

use anyhow::Result;
use clap::Parser;
use cli::{Cli, Cmd};
use config::Config;
use gpg::Gpg;
use store::Store;

fn main() -> Result<()> {
    // Intercept internal clipboard-clear subcommand before clap runs.
    // Spawned as a detached hidden process by `show --clip` and `generate --clip`.
    let mut raw: Vec<String> = std::env::args().collect();
    if raw.get(1).map(|s| s.as_str()) == Some("--internal-clip-clear") {
        let prev_b64 = raw.get(2).map(|s| s.as_str()).unwrap_or("");
        let secs: u64 = raw.get(3).and_then(|s| s.parse().ok()).unwrap_or(45);
        return clipboard::internal_clip_clear(prev_b64, secs);
    }

    // If no positional arg matches a known subcommand, inject `show` so that
    // `pass <name>`, `pass -c <name>`, and `pass <name> --clip` all work.
    const SUBCOMMANDS: &[&str] = &[
        "init", "ls", "list", "show", "insert", "add", "generate",
        "rm", "delete", "remove", "grep", "git", "find", "search",
        "edit", "cp", "copy", "mv", "rename", "completion", "help",
    ];
    let positional: Vec<&str> = raw[1..]
        .iter()
        .filter(|a| !a.starts_with('-') && a.as_str() != "--")
        .map(|a| a.as_str())
        .collect();
    let has_subcommand = positional.iter().any(|a| SUBCOMMANDS.contains(a));
    if !has_subcommand && !positional.is_empty() {
        raw.insert(1, "show".to_string());
    }

    let cli = Cli::parse_from(&raw);
    let config::Config { store_dir, clip_time, gpg_opts } = Config::load()?;
    let store = Store::open(store_dir);

    match cli.command {
        None => commands::list::run(&store, None),

        Some(Cmd::Ls { subfolder }) => commands::list::run(&store, subfolder.as_deref()),

        Some(Cmd::Rm { name, recursive, force }) => {
            commands::rm::run(&store, &name, recursive, force)
        }

        Some(Cmd::Completion { shell }) => commands::completion::run(&shell),

        Some(Cmd::Git { args }) => commands::git::run(&store, &args),

        Some(Cmd::Find { patterns }) => commands::find::run(&store, &patterns),

        Some(Cmd::Grep { args }) => {
            let gpg = Gpg::find(gpg_opts)?;
            commands::grep::run(&store, &gpg, &args)
        }

        Some(cmd) => {
            let gpg = Gpg::find(gpg_opts)?;

            match cmd {
                Cmd::Init { gpg_ids, path } => {
                    commands::init::run(&store, &gpg, &gpg_ids, path.as_deref())
                }

                Cmd::Show { name, clip, line } => {
                    commands::show::run(&store, &gpg, clip_time, &name, clip, line)
                }

                Cmd::Insert { name, echo, multiline, force } => {
                    commands::insert::run(&store, &gpg, &name, echo, multiline, force)
                }

                Cmd::Generate { name, length, no_symbols, clip, in_place, force } => {
                    commands::generate::run(
                        &store, &gpg, clip_time, &name, length, no_symbols, clip, in_place, force,
                    )
                }

                Cmd::Edit { name } => commands::edit::run(&store, &gpg, &name),

                Cmd::Cp { old_path, new_path, force } => {
                    commands::cp::run(&store, &gpg, &old_path, &new_path, force)
                }

                Cmd::Mv { old_path, new_path, force } => {
                    commands::mv::run(&store, &gpg, &old_path, &new_path, force)
                }

                // Already handled above; unreachable but needed to exhaust enum
                Cmd::Ls { .. } | Cmd::Rm { .. } | Cmd::Completion { .. }
                | Cmd::Git { .. } | Cmd::Grep { .. } | Cmd::Find { .. } => unreachable!(),
            }
        }
    }
}
