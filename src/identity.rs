use crate::error::IdentityError;
use std::{collections::HashMap, env};

const DEFAULT_PROFILE_LOCATION: &str = ".git-identities/profiles";

pub struct ListIdentities {
    verbose: bool,
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
    pub fn get_active_identity(&self) -> Option<String> {
        let found_identity = load_identity_from_config().ok()?;
        self.available_identities
            .values()
            .find(|&identity| identity == &found_identity)
            .map(|v| v.identity_name.clone())
    }
}

type ListIdentitiesResult = std::result::Result<ListIdentities, IdentityError>;

pub fn list_identities(verbose: bool) -> ListIdentitiesResult {
    let identity_store = load_identities()?;

    Ok(ListIdentities {
        verbose,
        available_identities: identity_store,
    })
}

pub struct SetIdentity {
    verbose: bool,
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

#[derive(Clone)]
struct GitIdentity {
    identity_name: String,
    user_name: String,
    email: String,
    signing_key: Option<String>,
}

impl PartialEq for GitIdentity {
    fn eq(&self, other: &Self) -> bool {
        self.user_name == other.user_name
            && self.email == other.email
            && self.signing_key == other.signing_key
    }
}

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

fn parse_identity_key(key: &str) -> Option<(&str, &str)> {
    let parts: Vec<&str> = key.split('.').collect();
    if parts.len() >= 3 {
        Some((parts[1], parts[2]))
    } else {
        None
    }
}

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
