use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "pass",
    about = "The standard Unix password manager, rewritten in Rust.\n\n  pass <name>          show password (default command)\n  pass <name> --clip   copy to clipboard",
    version
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Cmd>,
}

#[derive(Subcommand)]
pub enum Cmd {
    /// Initialize the password store with a GPG key ID
    Init {
        /// GPG key IDs to encrypt with
        #[arg(required = true)]
        gpg_ids: Vec<String>,
        /// Initialize a subfolder with its own .gpg-id
        #[arg(long, short = 'p')]
        path: Option<String>,
    },
    /// List passwords
    #[command(alias = "list")]
    Ls {
        /// Subfolder to list
        subfolder: Option<String>,
    },
    /// Show existing password (default: `pass <name>` runs this)
    Show {
        /// Password name (or subfolder to list)
        name: String,
        /// Copy to clipboard instead of printing
        #[arg(long, short = 'c')]
        clip: bool,
        /// Line number to copy (default: 1)
        #[arg(long, short = 'n', default_value = "1")]
        line: usize,
    },
    /// Insert a new password
    #[command(alias = "add")]
    Insert {
        /// Password name
        name: String,
        /// Echo typed characters
        #[arg(long, short = 'e')]
        echo: bool,
        /// Read multi-line input until EOF
        #[arg(long, short = 'm')]
        multiline: bool,
        /// Overwrite existing without confirmation
        #[arg(long, short = 'f')]
        force: bool,
    },
    /// Generate a random password
    Generate {
        /// Password name
        name: String,
        /// Password length
        #[arg(default_value = "25")]
        length: usize,
        /// Use only alphanumeric characters
        #[arg(long, short = 'n')]
        no_symbols: bool,
        /// Copy generated password to clipboard
        #[arg(long, short = 'c')]
        clip: bool,
        /// Replace only the first line of an existing entry
        #[arg(long, short = 'i')]
        in_place: bool,
        /// Overwrite existing without confirmation
        #[arg(long, short = 'f')]
        force: bool,
    },
    /// Remove a password or directory
    #[command(aliases = ["delete", "remove"])]
    Rm {
        /// Password name or directory
        name: String,
        /// Remove directory recursively
        #[arg(long, short = 'r')]
        recursive: bool,
        /// Remove without confirmation
        #[arg(long, short = 'f')]
        force: bool,
    },
    /// Show password names matching a pattern
    #[command(alias = "search")]
    Find {
        /// Pattern(s) to search (case-insensitive substring)
        #[arg(required = true)]
        patterns: Vec<String>,
    },
    /// Edit a password in $EDITOR
    Edit {
        /// Password name
        name: String,
    },
    /// Copy a password to a new path
    #[command(alias = "copy")]
    Cp {
        /// Source password name
        old_path: String,
        /// Destination password name
        new_path: String,
        /// Overwrite existing without confirmation
        #[arg(long, short = 'f')]
        force: bool,
    },
    /// Move or rename a password
    #[command(alias = "rename")]
    Mv {
        /// Source password name
        old_path: String,
        /// Destination password name
        new_path: String,
        /// Overwrite existing without confirmation
        #[arg(long, short = 'f')]
        force: bool,
    },
    /// Run a git command inside the password store
    Git {
        /// git arguments
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    /// Search inside decrypted password files
    Grep {
        /// grep-compatible arguments: [-i] [-E] [--] <pattern>
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    /// Generate a shell completion script
    Completion {
        /// Shell to generate completion for: bash, zsh, fish, powershell, elvish
        shell: String,
    },
}
