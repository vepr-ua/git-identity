//! Identity management for git profiles.
//!
//! This module handles loading, listing, and applying git identity profiles
//! from the user's configuration file at `~/.git-identities/profiles`.
//!
//! # Config File Format
//!
//! The profiles file uses git config format:
//!
//! ```text
//! [identity "personal"]
//!     name = John Doe
//!     email = john@personal.com
//!
//! [identity "work"]
//!     name = John Doe
//!     email = john@company.com
//!     signingkey = ABC123
//! ```

use crate::error::IdentityError;
use std::{collections::HashMap, env};

/// Default location for the profiles config file, relative to home directory.
const DEFAULT_PROFILE_LOCATION: &str = ".git-identities/profiles";

/// Result of listing available git identities.
///
/// Contains all identities found in the profiles config file
/// and provides methods to determine which identity is currently active.
pub struct ListIdentities {
    /// Whether to display detailed identity information.
    verbose: bool,
    /// Map of identity names to their full profile data.
    available_identities: HashMap<String, GitIdentity>,
}

impl std::fmt::Display for ListIdentities {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let active_identity = self.get_active_identity();

        write!(f, "Available git identities:\n")?;
        for (_, identity) in &self.available_identities {
            let is_active = active_identity.as_ref() == Some(&identity.identity_name);
            let marker = if is_active {
                "*"
            } else if self.verbose {
                ""
            } else {
                "-"
            };

            if self.verbose {
                write!(
                    f,
                    "{}[{}]\n\t{}\n \t{}\n",
                    marker, identity.identity_name, identity.user_name, identity.email
                )?;

                if let Some(signing_key) = &identity.signing_key {
                    write!(f, "\t{}\n", signing_key)?;
                }
            } else {
                write!(f, "{} {}\n", marker, identity.identity_name)?;
            }
        }

        Ok(())
    }
}

impl ListIdentities {
    /// Returns the name of the currently active identity, if one matches.
    ///
    /// Compares the current repository's git config (user.name, user.email,
    /// user.signingkey) against all available identities to find a match.
    ///
    /// Returns `None` if not in a git repository or no identity matches.
    pub fn get_active_identity(&self) -> Option<String> {
        let found_identity = load_identity_from_config().ok()?;
        self.available_identities
            .values()
            .find(|&identity| identity == &found_identity)
            .map(|v| v.identity_name.clone())
    }
}

type ListIdentitiesResult = std::result::Result<ListIdentities, IdentityError>;

/// Lists all available git identities from the user's profiles config.
///
/// Reads identities from `~/.git-identities/profiles` and returns them
/// wrapped in a displayable struct.
///
/// # Arguments
///
/// * `verbose` - If true, the display output includes full identity details.
///
/// # Errors
///
/// Returns [`IdentityError::ConfigNotFound`] if the home directory or profiles file doesn't exist.
/// Returns [`IdentityError::Git`] if the config file cannot be parsed.
pub fn list_identities(verbose: bool) -> ListIdentitiesResult {
    let identity_store = load_identities()?;

    Ok(ListIdentities {
        verbose,
        available_identities: identity_store,
    })
}

/// Result of setting a git identity on the current repository.
///
/// Contains the identity that was applied, used for display confirmation.
pub struct SetIdentity {
    /// Whether to display confirmation message.
    verbose: bool,
    /// The identity that was applied to the repository.
    identity_used: GitIdentity,
}

impl std::fmt::Display for SetIdentity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.verbose {
            writeln!(
                f,
                "Repo now uses {} identity",
                self.identity_used.identity_name
            )?;
        }

        Ok(())
    }
}

type SetIdentityResult = std::result::Result<SetIdentity, IdentityError>;

/// Sets the git identity for the current repository.
///
/// Looks up the identity by name and applies its values (user.name, user.email,
/// user.signingkey) to the current repository's local git config.
///
/// # Arguments
///
/// * `identity_lookup_key` - The name of the identity to apply (e.g., "work", "personal").
/// * `verbose` - If true, displays a confirmation message after applying.
///
/// # Errors
///
/// Returns [`IdentityError::IdentityNotFound`] if no identity matches the given name.
/// Returns [`IdentityError::Git`] if not in a git repository or config cannot be written.
pub fn set_identity(identity_lookup_key: &str, verbose: bool) -> SetIdentityResult {
    let identities = load_identities()?;
    let Some(found_identity) = identities.get(identity_lookup_key) else {
        return Err(IdentityError::IdentityNotFound(
            identity_lookup_key.to_string(),
        ));
    };

    write_identity_to_config(found_identity)?;

    Ok(SetIdentity {
        verbose,
        identity_used: found_identity.to_owned(),
    })
}

/// A git identity profile containing user information.
///
/// Represents a single identity that can be applied to a git repository.
/// Identities are compared by their values (user_name, email, signing_key),
/// not by their identity_name.
#[derive(Clone)]
struct GitIdentity {
    /// The name used to reference this identity (e.g., "work", "personal").
    identity_name: String,
    /// The git user.name value.
    user_name: String,
    /// The git user.email value.
    email: String,
    /// Optional git user.signingkey value for commit signing.
    signing_key: Option<String>,
}

impl PartialEq for GitIdentity {
    fn eq(&self, other: &Self) -> bool {
        self.user_name == other.user_name
            && self.email == other.email
            && self.signing_key == other.signing_key
    }
}

/// Loads all identities from the profiles config file.
///
/// Reads `~/.git-identities/profiles` and parses each `[identity "name"]`
/// section into a `GitIdentity` struct.
fn load_identities() -> std::result::Result<HashMap<String, GitIdentity>, IdentityError> {
    // TODO: Allow user to set an environment variable that we can use to select a path.
    let home = env::home_dir().ok_or(IdentityError::ConfigNotFound)?;
    let path = home.join(DEFAULT_PROFILE_LOCATION);

    let config = git2::Config::open(&path)?;
    let mut cfg_entries = config.entries(None)?;
    let mut identity_store: HashMap<String, GitIdentity> = HashMap::new();
    while let Some(entry) = cfg_entries.next() {
        let entry = entry?;
        if entry.name().is_none() {
            continue;
        }

        let value = entry.value().filter(|v| !v.is_empty());

        let Some((key, property)) = parse_identity_key(entry.name().unwrap()) else {
            // TODO: Add a warning or something here! How can a key from a config be empty?
            continue;
        };

        let identity = identity_store
            .entry(key.to_string())
            .or_insert_with(|| GitIdentity {
                identity_name: key.to_string(),
                user_name: String::new(),
                email: String::new(),
                signing_key: None,
            });

        match property {
            "name" => identity.user_name = value.unwrap_or_default().to_string(),
            "email" => identity.email = value.unwrap_or_default().to_string(),
            "signingkey" => identity.signing_key = value.map(|v| v.to_string()),
            _ => continue,
        };
    }

    Ok(identity_store)
}

/// Parses a git config key into (identity_name, property) tuple.
///
/// Git config keys from the profiles file are formatted as:
/// `identity.<name>.<property>` (e.g., `identity.work.email`).
///
/// Returns `None` if the key doesn't have at least 3 parts.
fn parse_identity_key(key: &str) -> Option<(&str, &str)> {
    let parts: Vec<&str> = key.split('.').collect();
    if parts.len() >= 3 {
        Some((parts[1], parts[2]))
    } else {
        None
    }
}

/// Reads the current git identity from the repository's config.
///
/// Discovers the git repository from the current directory and reads
/// user.name, user.email, and optionally user.signingkey.
///
/// The returned `GitIdentity` has `identity_name` set to "unknown" since
/// it's read from config, not from our profiles.
fn load_identity_from_config() -> std::result::Result<GitIdentity, IdentityError> {
    let repo = git2::Repository::discover(".")?;

    let config = repo.config()?;
    let user_name = config.get_string("user.name")?;
    let email = config.get_string("user.email")?;
    let signing_key = config.get_string("user.signingkey").ok();

    Ok(GitIdentity {
        identity_name: "unknown".to_string(),
        user_name: user_name,
        email: email,
        signing_key: signing_key,
    })
}

/// Writes an identity's values to the current repository's git config.
///
/// Sets user.name, user.email, and user.signingkey (if present) in the
/// repository's local config. If the identity has no signing key, any
/// existing signingkey config is removed.
fn write_identity_to_config(identity: &GitIdentity) -> std::result::Result<(), IdentityError> {
    let repo = git2::Repository::discover(".")?;

    // TODO: Allow user to specify the global or local config
    let mut config = repo.config()?;

    config.set_str("user.name", &identity.user_name)?;
    config.set_str("user.email", &identity.email)?;

    if let Some(signing_key) = &identity.signing_key {
        config.set_str("user.signingkey", signing_key)?;
    } else {
        config.remove("user.signingkey")?;
    };

    Ok(())
}
