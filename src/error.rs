//! Error types for git-identity operations.

/// Errors that can occur when managing git identities.
#[derive(Debug)]
pub enum IdentityError {
    /// The profiles config file was not found at `~/.git-identities/profiles`.
    ConfigNotFound,
    /// The requested identity name does not exist in the profiles config.
    IdentityNotFound(String),
    /// An I/O error occurred while reading or writing files.
    IO(std::io::Error),
    /// A git operation failed (e.g., repository not found, config error).
    Git(git2::Error),
}

impl std::fmt::Display for IdentityError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IdentityError::ConfigNotFound => {
                write!(f, "Config file not found at ~/.git-identities/profiles")
            }
            IdentityError::IdentityNotFound(identity) => {
                write!(f, "\"{}\" identity not found", identity)
            }
            IdentityError::IO(error) => write!(f, "IO error: {}", error),
            IdentityError::Git(error) => write!(f, "Git error: {}", error),
        }
    }
}

impl From<std::io::Error> for IdentityError {
    fn from(value: std::io::Error) -> Self {
        IdentityError::IO(value)
    }
}

impl From<git2::Error> for IdentityError {
    fn from(value: git2::Error) -> Self {
        IdentityError::Git(value)
    }
}

impl std::error::Error for IdentityError {}
