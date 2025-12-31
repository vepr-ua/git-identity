//! git-identity: A CLI tool for managing git identity profiles.
//!
//! This tool allows you to define multiple git identities (name, email, signing key)
//! in a central config file and quickly switch between them on a per-repository basis.
//!
//! # Usage
//!
//! ```text
//! git-identity list           # List all available identities
//! git-identity set <name>     # Apply an identity to the current repo
//! ```
//!
//! # Configuration
//!
//! Create a profiles file at `~/.git-identities/profiles` with your identities:
//!
//! ```text
//! [identity "personal"]
//!     name = Your Name
//!     email = you@personal.com
//!
//! [identity "work"]
//!     name = Your Name
//!     email = you@company.com
//!     signingkey = ABC123
//! ```

mod error;
mod identity;

use clap::{Parser, Subcommand};
use error::IdentityError;

use crate::identity::{list_identities, set_identity};

/// Convenience type alias for Results with [`IdentityError`].
type Result<T> = std::result::Result<T, IdentityError>;

/// Command-line interface definition.
#[derive(Parser)]
#[command(version, about = "Manage git identity profiles", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
    /// Enable verbose output with additional details.
    #[arg(short, long, action = clap::ArgAction::SetTrue)]
    verbose: Option<bool>,
}

/// Available subcommands.
#[derive(Subcommand)]
enum Commands {
    /// List all available identities from the profiles config.
    List,
    /// Set the git identity for the current repository.
    Set {
        /// The name of the identity to apply (e.g., "work", "personal").
        #[arg(help = "The name of the identity. Can be referenced from list command.")]
        identity: String,
    },
}

impl Cli {
    /// Executes the CLI command and returns any errors.
    pub fn run(&self) -> Result<()> {
        let is_verbose = self.verbose.unwrap_or_default();
        match &self.command {
            Commands::List => {
                let results = list_identities(is_verbose)?;
                print!("{}\n", results);

                Ok(())
            }
            Commands::Set { identity } => {
                let results = set_identity(identity.as_str(), is_verbose)?;
                print!("{}\n", results);

                Ok(())
            }
        }
    }
}

fn main() {
    let cli = Cli::parse();

    if let Err(err) = cli.run() {
        eprintln!("{}", err);
        std::process::exit(1);
    }
}
