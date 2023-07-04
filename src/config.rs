use std::{collections::HashMap, fs::File, io::Read, path::PathBuf};

use dirs::{data_dir, home_dir};
use toml::Value;

pub type RepoList = HashMap<String, String>;

#[derive(Debug)]
pub enum ConfigError {
    HomeDirUnavailable,
    ConfigDirUnavailable,
    ConfigNotFound,
    PermissionDenied,
    InvalidData,
    Interrupted,
    UnexpectedEof,
    Unknown,
    InvalidToml,
    NoRepo,
    DataDirUnavailable,
}

pub struct Config {
    config: Value,
}

impl Config {
    pub fn read() -> Result<Config, ConfigError> {
        let home_dir = home_dir().ok_or(ConfigError::HomeDirUnavailable)?;
        let config_dir = home_dir.join(".config");

        if !config_dir.exists() {
            return Err(ConfigError::ConfigDirUnavailable);
        }

        let scoopie_config = config_dir.join("scoopie.toml");

        if !scoopie_config.exists() {
            return Err(ConfigError::ConfigNotFound);
        }

        let mut file = File::open(&scoopie_config).map_err(|e| match e.kind() {
            std::io::ErrorKind::NotFound => ConfigError::ConfigNotFound,
            std::io::ErrorKind::PermissionDenied => ConfigError::PermissionDenied,
            _ => ConfigError::Unknown,
        })?;

        let mut buffer = String::new();
        file.read_to_string(&mut buffer)
            .map_err(|e| match e.kind() {
                std::io::ErrorKind::InvalidData => ConfigError::InvalidData,
                std::io::ErrorKind::Interrupted => ConfigError::Interrupted,
                std::io::ErrorKind::UnexpectedEof => ConfigError::UnexpectedEof,
                _ => ConfigError::Unknown,
            })?;

        let toml: Value = toml::from_str(&buffer).map_err(|_| ConfigError::InvalidToml)?;

        Ok(Config { config: toml })
    }

    pub fn repos(&self) -> Result<RepoList, ConfigError> {
        let repos = self.config.get("repos").ok_or(ConfigError::NoRepo)?;

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

    pub fn repos_dir() -> Result<PathBuf, ConfigError> {
        Ok(data_dir()
            .ok_or(ConfigError::DataDirUnavailable)?
            .join(r"scoopie\repos"))
    }
}
