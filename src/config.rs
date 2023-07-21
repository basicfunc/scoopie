use std::{
    collections::HashMap,
    env,
    fs::{self, File},
    io::Read,
    path::PathBuf,
    time::SystemTime,
};

use crate::error::{ConfigError, ScoopieError};

use dirs::home_dir;
use rayon::prelude::*;
use toml::Value;

pub type RepoList = HashMap<String, String>;

pub trait Reader {
    fn read() -> Result<Self, ScoopieError>
    where
        Self: Sized;
}

pub struct Config {
    config: Value,
}

impl Reader for Config {
    fn read() -> Result<Self, ScoopieError> {
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

        let config: Value =
            toml::from_str(&buffer).map_err(|_| ScoopieError::Config(ConfigError::InvalidToml))?;

        Ok(Config { config })
    }
}

pub trait Buckets {
    fn known_buckets(&self) -> Result<RepoList, ScoopieError>;
    fn latest_buckets(&self) -> Result<Vec<PathBuf>, ScoopieError>;
}

impl Buckets for Config {
    fn known_buckets(&self) -> Result<RepoList, ScoopieError> {
        let repos = self
            .config
            .get("buckets")
            .ok_or(ScoopieError::Config(ConfigError::NoBucketsConfigured))?;

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

    fn latest_buckets(&self) -> Result<Vec<PathBuf>, ScoopieError> {
        let repo_dir = Self::buckets_dir()?;

        let repos = self
            .config
            .get("buckets")
            .ok_or(ScoopieError::Config(ConfigError::NoBucketsConfigured))?;

        let repo_names: Result<Vec<String>, ScoopieError> = if let Value::Table(table) = repos {
            Ok(table.keys().cloned().collect::<Vec<String>>())
        } else {
            Err(ScoopieError::Config(ConfigError::NoBucketsConfigured))
        };

        let repo_names = repo_names?;

        let entries = fs::read_dir(repo_dir)
            .map_err(|_| ScoopieError::Config(ConfigError::NoBucketsConfigured))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|_| ScoopieError::Config(ConfigError::NoBucketsConfigured))?;

        let mut latest_files: Vec<PathBuf> = Vec::new();

        repo_names.iter().for_each(|repo_name| {
            let latest_path = entries
                .par_iter()
                .filter(|entry| {
                    let file_name = entry.file_name().to_string_lossy().to_string();
                    file_name.ends_with(".db") && file_name.starts_with(repo_name)
                })
                .max_by_key(|entry| {
                    entry
                        .metadata()
                        .and_then(|metadata| metadata.modified())
                        .unwrap_or_else(|_| SystemTime::UNIX_EPOCH)
                });

            latest_path.map_or((), |path| latest_files.push(path.path()));
        });

        Ok(latest_files)
    }
}

pub trait DefaultDirs {
    fn home_dir() -> Result<PathBuf, ScoopieError>;
    fn app_dir() -> Result<PathBuf, ScoopieError>;
    fn cache_dir() -> Result<PathBuf, ScoopieError>;
    fn persist_dir() -> Result<PathBuf, ScoopieError>;
    fn buckets_dir() -> Result<PathBuf, ScoopieError>;
    fn shims_dir() -> Result<PathBuf, ScoopieError>;
}

impl DefaultDirs for Config {
    fn home_dir() -> Result<PathBuf, ScoopieError> {
        let scoopie_home = env::var("SCOOPIE_HOME").map_err(|_| ScoopieError::EnvResolve)?;
        Ok(PathBuf::from(scoopie_home))
    }

    fn buckets_dir() -> Result<PathBuf, ScoopieError> {
        let scoopie_home = Self::home_dir()?;
        let buckets_dir = scoopie_home.join("buckets");

        if buckets_dir.exists() {
            Ok(buckets_dir)
        } else {
            Err(ScoopieError::BucketsDirUnavailable)
        }
    }

    fn cache_dir() -> Result<PathBuf, ScoopieError> {
        let scoopie_home = Self::home_dir()?;
        let cache_dir = scoopie_home.join("cache");

        if cache_dir.exists() {
            Ok(cache_dir)
        } else {
            Err(ScoopieError::CacheDirUnavailable)
        }
    }

    fn app_dir() -> Result<PathBuf, ScoopieError> {
        let scoopie_home = Self::home_dir()?;
        let apps_dir = scoopie_home.join("apps");

        if apps_dir.exists() {
            Ok(apps_dir)
        } else {
            Err(ScoopieError::AppsDirUnavailable)
        }
    }

    fn persist_dir() -> Result<PathBuf, ScoopieError> {
        let scoopie_home = Self::home_dir()?;
        let persist_dir = scoopie_home.join("persists");

        if persist_dir.exists() {
            Ok(persist_dir)
        } else {
            Err(ScoopieError::PersistDirUnavailable)
        }
    }

    fn shims_dir() -> Result<PathBuf, ScoopieError> {
        let scoopie_home = Self::home_dir()?;
        let shims_dir = scoopie_home.join("shims");

        if shims_dir.exists() {
            Ok(shims_dir)
        } else {
            Err(ScoopieError::ShimsDirUnavailable)
        }
    }
}

pub trait Stats {
    fn arch() -> Result<&'static str, ScoopieError>;
}

impl Stats for Config {
    fn arch() -> Result<&'static str, ScoopieError> {
        match env::consts::ARCH {
            "x86" => Ok("32bit"),
            "x86_64" => Ok("64bit"),
            _ => Err(ScoopieError::UnknownArch),
        }
    }
}
