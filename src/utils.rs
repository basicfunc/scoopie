use dirs;
use std::env;
use std::error::Error;
use std::fmt;
use std::path::PathBuf;

const PREFIX_KEY: &'static str = "SCOOPIE_DIR";
const DEFAULT_PREFIX: &'static str = "scoopie";

#[derive(Debug)]
pub enum PrefixError {
    HomeDirError,
    ConvertToStrError,
    EnvVarError(String),
    InvalidFormatError,
}

impl Error for PrefixError {}

impl fmt::Display for PrefixError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PrefixError::HomeDirError => write!(f, "Unable to get home dir."),
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

pub fn get_prefix() -> Result<PathBuf, PrefixError> {
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
