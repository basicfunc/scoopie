use crate::error::ScoopieError;
use std::{
    fs::{remove_dir_all, remove_file, DirBuilder},
    path::PathBuf,
};

use winreg::{enums::*, RegKey};

pub struct EnvVar {
    key: String,
    value: String,
}

impl EnvVar {
    pub fn new(key: &str, value: &str) -> Self {
        Self {
            key: key.into(),
            value: value.into(),
        }
    }

    pub fn create_or_update(&self) -> Result<(), ScoopieError> {
        let curr_user = RegKey::predef(HKEY_CURRENT_USER);

        let env = curr_user
            .open_subkey_with_flags("Environment", KEY_ALL_ACCESS)
            .map_err(|_| ScoopieError::UnableToOpenEnvRegistry)?;

        env.set_value(&self.key, &self.value)
            .map_err(|_| ScoopieError::EnvSet)
    }

    pub fn remove(key: &str) -> Result<(), ScoopieError> {
        let curr_user = RegKey::predef(HKEY_CURRENT_USER);
        let env = curr_user
            .open_subkey_with_flags("Environment", KEY_ALL_ACCESS)
            .map_err(|_| ScoopieError::UnableToOpenEnvRegistry)?;

        env.delete_value(key).map_err(|_| ScoopieError::EnvRemove)
    }
}

pub trait Remove {
    type Error;
    fn rm(&self) -> Result<(), Self::Error>;
}

impl Remove for PathBuf {
    type Error = ScoopieError;

    fn rm(&self) -> Result<(), Self::Error> {
        match self.is_file() {
            true => remove_file(self),
            false => remove_dir_all(&self),
        }
        .map_err(|err| match err.kind() {
            std::io::ErrorKind::NotFound => ScoopieError::FileNotExist(self.to_path_buf()),
            std::io::ErrorKind::PermissionDenied => ScoopieError::PermissionDenied,
            _ => ScoopieError::Unknown,
        })
    }
}

pub trait CreateDir {
    type Error;
    fn create(path: Self) -> Result<(), Self::Error>;
}

impl CreateDir for PathBuf {
    type Error = ScoopieError;

    fn create(path: Self) -> Result<(), Self::Error> {
        DirBuilder::new()
            .recursive(true)
            .create(&path)
            .map_err(|_| ScoopieError::FailedToMkdir(path.to_path_buf()))
    }
}

pub trait Absolute {
    type Error;
    fn absolute(&self) -> Result<PathBuf, Self::Error>;
}

impl Absolute for PathBuf {
    type Error = ScoopieError;

    fn absolute(&self) -> Result<PathBuf, Self::Error> {
        let absolute_path = self
            .canonicalize()
            .map_err(|_| ScoopieError::AbsoultePathResolve)?;
        let absolute_path_str = absolute_path.to_string_lossy().to_string();

        // Remove the `\\?\` prefix from the absolute path string
        Ok(PathBuf::from(
            match absolute_path_str.starts_with("\\\\?\\") {
                true => absolute_path_str[4..].to_string(),
                false => absolute_path_str,
            },
        ))
    }
}
