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
    let raw: Vec<String> = std::env::args().collect();
    if raw.get(1).map(|s| s.as_str()) == Some("--internal-clip-clear") {
        let prev_b64 = raw.get(2).map(|s| s.as_str()).unwrap_or("");
        let secs: u64 = raw.get(3).and_then(|s| s.parse().ok()).unwrap_or(45);
        return clipboard::internal_clip_clear(prev_b64, secs);
    }

    let cli = Cli::parse();
    let config = Config::load()?;
    let store = Store::open(config.store_dir.clone());

    match cli.command {
        None => commands::list::run(&store, None),

        Some(Cmd::Ls { subfolder }) => commands::list::run(&store, subfolder.as_deref()),

        Some(Cmd::Rm { name, recursive, force }) => {
            commands::rm::run(&store, &name, recursive, force)
        }

        Some(cmd) => {
            // Commands below all need gpg — find it once here
            let gpg = Gpg::find(config.gpg_opts.clone())?;

            match cmd {
                Cmd::Init { gpg_ids, path } => {
                    commands::init::run(&store, &gpg, &gpg_ids, path.as_deref())
                }

                Cmd::Show { name, clip, line } => {
                    commands::show::run(&store, &gpg, &config, &name, clip, line)
                }

                Cmd::Insert { name, echo, multiline, force } => {
                    commands::insert::run(&store, &gpg, &name, echo, multiline, force)
                }

                Cmd::Generate { name, length, no_symbols, clip, in_place, force } => {
                    commands::generate::run(
                        &store, &gpg, &config, &name, length, no_symbols, clip, in_place, force,
                    )
                }

                // Already handled above; unreachable but needed to exhaust enum
                Cmd::Ls { .. } | Cmd::Rm { .. } => unreachable!(),
            }
        }
    }
}
