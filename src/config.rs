use std::{collections::HashMap, fs::File, io::Read, path::PathBuf};

use dirs::{data_dir, home_dir};
use toml::Value;

use crate::error::{ConfigError, ScoopieError};

pub type RepoList = HashMap<String, String>;

pub struct Config {
    config: Value,
}

impl Config {
    pub fn read() -> Result<Config, ScoopieError> {
        let home_dir = home_dir().ok_or(ScoopieError::HomeDirUnavailable)?;
        let config_dir = home_dir.join(".config");

        if !config_dir.exists() {
            return Err(ScoopieError::ConfigDirUnavailable);
        }

        let scoopie_config = config_dir.join("scoopie.toml");

        if !scoopie_config.exists() {
            return Err(ScoopieError::Config(ConfigError::ConfigNotFound));
        }

        let mut file = File::open(&scoopie_config).map_err(|e| match e.kind() {
            std::io::ErrorKind::NotFound => ScoopieError::Config(ConfigError::ConfigNotFound),
            std::io::ErrorKind::PermissionDenied => ScoopieError::PermissionDenied,
            _ => ScoopieError::Unknown,
        })?;

        let mut buffer = String::new();
        file.read_to_string(&mut buffer)
            .map_err(|e| match e.kind() {
                std::io::ErrorKind::InvalidData => ScoopieError::Config(ConfigError::InvalidData),
                std::io::ErrorKind::Interrupted => ScoopieError::Config(ConfigError::Interrupted),
                std::io::ErrorKind::UnexpectedEof => {
                    ScoopieError::Config(ConfigError::UnexpectedEof)
                }
                _ => ScoopieError::Unknown,
            })?;

        let toml: Value =
            toml::from_str(&buffer).map_err(|_| ScoopieError::Config(ConfigError::InvalidToml))?;

        Ok(Config { config: toml })
    }

    pub fn repos(&self) -> Result<RepoList, ScoopieError> {
        let repos = self
            .config
            .get("repos")
            .ok_or(ScoopieError::Config(ConfigError::NoRepo))?;

        let mut repo_list = RepoList::new();

        if let Value::Table(table) = repos {
            for (key, value) in table.iter() {
                if let Value::String(str_val) = value {
                    repo_list.insert(key.clone(), str_val.clone());
                }
            }
        }

        Ok(repo_list)
    }

    pub fn repos_dir() -> Result<PathBuf, ScoopieError> {
        Ok(data_dir()
            .ok_or(ScoopieError::DataDirUnavailable)?
            .join(r"scoopie\repos"))
    }
}
