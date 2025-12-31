mod error;
mod identity;

use clap::{Parser, Subcommand};
use error::IdentityError;

use crate::identity::{list_identities, set_identity};

type Result<T> = std::result::Result<T, IdentityError>;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
    #[arg(short, long, action = clap::ArgAction::SetTrue)]
    verbose: Option<bool>,
}

#[derive(Subcommand)]
enum Commands {
    List,
    Set {
        #[arg(help = "The name of the identity. Can be referenced from list command.")]
        identity: String,
    },
}

impl Cli {
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
