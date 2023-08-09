use std::{
    collections::HashMap,
    env, fs,
    path::{Path, PathBuf},
};

use crate::error::*;

use dirs::home_dir;
use serde::{Deserialize, Serialize};

pub trait Reader {
    type Error;
    fn read() -> Result<Self, Self::Error>
    where
        Self: Sized;
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    buckets: HashMap<String, String>,
    download: Download,
}

impl Default for Config {
    fn default() -> Self {
        let buckets: [(String, String); 3] = [
            (
                String::from("main"),
                String::from("https://github.com/ScoopInstaller/Main"),
            ),
            (
                String::from("extras"),
                String::from("https://github.com/ScoopInstaller/Extras"),
            ),
            (
                String::from("versions"),
                String::from("https://github.com/ScoopInstaller/Versions"),
            ),
        ];

        let buckets = HashMap::from(buckets);

        Self {
            buckets,
            download: Default::default(),
        }
    }
}

pub trait Write<T> {
    type Error;
    fn write(path: T) -> Result<(), ScoopieError>;
}

impl Write<&Path> for Config {
    type Error = ScoopieError;
    fn write(path: &Path) -> Result<(), Self::Error> {
        let default_config: Config = Config::default();
        let config = serde_json::to_string_pretty(&default_config)
            .map_err(|_| ScoopieError::Config(crate::error::ConfigError::InvalidToml))?;

        fs::write(path, config).map_err(|_| ScoopieError::Init(InitError::ConfigWrite))
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Download {
    pub max_retries: u32,
    pub concurrent_downloads: usize,
    pub hide_progress_bar: bool,
    pub progress_bar_style: String,
}

impl Default for Download {
    fn default() -> Self {
        Download {
            max_retries: 5,
            concurrent_downloads: 4,
            hide_progress_bar: false,
            progress_bar_style: "PIP".to_string(),
        }
    }
}

impl TryFrom<PathBuf> for Config {
    type Error = ScoopieError;

    fn try_from(value: PathBuf) -> Result<Self, Self::Error> {
        let buffer = std::fs::read(&value).map_err(|e| match e.kind() {
            std::io::ErrorKind::NotFound => ScoopieError::Config(ConfigError::ConfigNotFound),
            std::io::ErrorKind::PermissionDenied => ScoopieError::PermissionDenied,
            std::io::ErrorKind::InvalidData => ScoopieError::Config(ConfigError::InvalidData),
            std::io::ErrorKind::Interrupted => ScoopieError::Config(ConfigError::Interrupted),
            std::io::ErrorKind::UnexpectedEof => ScoopieError::Config(ConfigError::UnexpectedEof),
            _ => ScoopieError::Unknown,
        })?;

        let content = String::from_utf8(buffer)
            .map_err(|_| ScoopieError::Config(ConfigError::InvalidData))?;

        serde_json::from_str::<Config>(&content)
            .map_err(|_| ScoopieError::Config(ConfigError::InvalidToml))
    }
}

impl Reader for Config {
    type Error = ScoopieError;

    fn read() -> Result<Self, Self::Error> {
        let home_dir = home_dir().ok_or(ScoopieError::UserDirUnavailable)?;
        let scoopie_config = home_dir.join(".config\\scoopie.json");

        match scoopie_config.exists() {
            true => Config::try_from(scoopie_config),
            false => Err(ScoopieError::Config(ConfigError::ConfigNotFound)),
        }
    }
}

impl Config {
    pub fn known_buckets(self) -> HashMap<String, String> {
        self.buckets
    }

    pub fn download(&self) -> Download {
        self.download.to_owned()
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
        let buckets_dir = Self::home_dir()?.join("buckets");

        match buckets_dir.exists() {
            true => Ok(buckets_dir),
            false => Err(ScoopieError::BucketsDirUnavailable),
        }
    }

    fn cache_dir() -> Result<PathBuf, ScoopieError> {
        let cache_dir = Self::home_dir()?.join("cache");

        match cache_dir.exists() {
            true => Ok(cache_dir),
            false => Err(ScoopieError::CacheDirUnavailable),
        }
    }

    fn app_dir() -> Result<PathBuf, ScoopieError> {
        let apps_dir = Self::home_dir()?.join("apps");

        match apps_dir.exists() {
            true => Ok(apps_dir),
            false => Err(ScoopieError::AppsDirUnavailable),
        }
    }

    fn persist_dir() -> Result<PathBuf, ScoopieError> {
        let persist_dir = Self::home_dir()?.join("persists");

        match persist_dir.exists() {
            true => Ok(persist_dir),
            false => Err(ScoopieError::PersistDirUnavailable),
        }
    }

    fn shims_dir() -> Result<PathBuf, ScoopieError> {
        let shims_dir = Self::home_dir()?.join("shims");

        match shims_dir.exists() {
            true => Ok(shims_dir),
            false => Err(ScoopieError::ShimsDirUnavailable),
        }
    }
}

#[derive(Debug)]
pub enum Arch {
    Bit64,
    Bit32,
    Arm64,
}

pub trait Stats {
    fn arch() -> Result<Arch, ScoopieError>;
}

impl Stats for Config {
    fn arch() -> Result<Arch, ScoopieError> {
        match env::consts::ARCH {
            "x86" => Ok(Arch::Bit32),
            "x86_64" => Ok(Arch::Bit64),
            "aarch64" => Ok(Arch::Arm64),
            _ => Err(ScoopieError::UnknownArch),
        }
    }
}
