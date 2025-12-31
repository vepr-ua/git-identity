#[derive(Debug)]
pub enum IdentityError {
    ConfigNotFound,
    IdentityNotFound(String),
    IO(std::io::Error),
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
