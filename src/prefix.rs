use argh::FromArgs;
use dirs;
use std::error::Error;
use std::fmt;
use std::path::PathBuf;
use std::{env, format};

use crate::{DEFAULT_PREFIX, PREFIX_KEY};

#[derive(FromArgs, PartialEq, Debug)]
/// Get Scoopie Prefix
#[argh(subcommand, name = "prefix")]
pub struct PrefixCommand {}

#[derive(Debug)]
pub enum PrefixError {
    HomeDirError,
    ConfigDirError,
    ConvertToStrError,
    EnvVarError(String),
    InvalidFormatError,
}

impl Error for PrefixError {}

impl fmt::Display for PrefixError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PrefixError::HomeDirError => write!(f, "Unable to get CONFIG dir."),
            PrefixError::ConfigDirError => write!(f, "Unable to get CONFIG dir."),
            PrefixError::ConvertToStrError => write!(f, "Unable to convert path to string."),
            PrefixError::EnvVarError(var_name) => {
                write!(f, "Error retrieving environment variable: {}", var_name)
            }
            PrefixError::InvalidFormatError => {
                write!(f, "Invalid format for environment variable.")
            }
        }
    }
}

impl PrefixCommand {
    pub fn show() -> Result<(PathBuf, PathBuf), PrefixError> {
        let prefix = Self::prefix()?;
        let config = Self::config()?;

        Ok((prefix, config))
    }

    pub fn prefix() -> Result<PathBuf, PrefixError> {
        let home_dir = dirs::home_dir().ok_or(PrefixError::HomeDirError)?;
        let home_path = home_dir.to_str().ok_or(PrefixError::ConvertToStrError)?;

        let default_path = format!("{}\\{}", home_path, DEFAULT_PREFIX);
        let default_path_buf = PathBuf::from(default_path);

        let path = match env::var(PREFIX_KEY) {
            Ok(path) => PathBuf::from(path),
            Err(e) => match e {
                env::VarError::NotPresent => default_path_buf,
                env::VarError::NotUnicode(_) => return Err(PrefixError::InvalidFormatError),
            },
        };

        Ok(path)
    }

    pub fn config() -> Result<PathBuf, PrefixError> {
        let config_dir = dirs::config_dir().ok_or(PrefixError::ConfigDirError)?;
        let config_dir = config_dir.to_str().ok_or(PrefixError::ConvertToStrError)?;

        let config_path = format!("{}\\{}", config_dir, DEFAULT_PREFIX);

        Ok(PathBuf::from(config_path))
    }
}
