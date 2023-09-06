use std::path::PathBuf;

use winreg::{enums::*, RegKey};

use crate::error::ScoopieError;

pub struct Env;

impl Env {
    pub fn home_dir() -> Result<PathBuf, ScoopieError> {
        let curr_user = RegKey::predef(HKEY_CURRENT_USER);
        let vars = curr_user
            .open_subkey_with_flags("Volatile Environment", KEY_READ)
            .map_err(|_| ScoopieError::UnableToOpenEnvRegistry)?;

        let home_dir: String = vars
            .get_value("USERPROFILE")
            .map_err(|_| ScoopieError::UserDirUnavailable)?;

        Ok(PathBuf::from(home_dir))
    }

    pub fn create_or_update(key: &str, value: &str) -> Result<(), ScoopieError> {
        let curr_user = RegKey::predef(HKEY_CURRENT_USER);

        let env = curr_user
            .open_subkey_with_flags("Environment", KEY_ALL_ACCESS)
            .map_err(|_| ScoopieError::UnableToOpenEnvRegistry)?;

        env.set_value(key, &value).map_err(|_| ScoopieError::EnvSet)
    }

    pub fn remove(key: &str) -> Result<(), ScoopieError> {
        let curr_user = RegKey::predef(HKEY_CURRENT_USER);
        let env = curr_user
            .open_subkey_with_flags("Environment", KEY_ALL_ACCESS)
            .map_err(|_| ScoopieError::UnableToOpenEnvRegistry)?;

        env.delete_value(key).map_err(|_| ScoopieError::EnvRemove)
    }
}
