pub mod bucket;
pub mod cache;
pub mod cat;
pub mod cleanup;
pub mod download;
pub mod info;
pub mod init;
pub mod install;
pub mod list;
pub mod nuke;
pub mod prefix;
pub mod search;
pub mod shim;
pub mod status;
pub mod uninstall;
pub mod update;
pub mod which;

use std::{env, fmt, path::PathBuf};

// External Crate Imports.
use argh::FromArgs;

// Internal Module Imports
use bucket::BucketCommand;
use cache::CacheCommand;
use cat::CatCommand;
use cleanup::CleanupCommand;
use download::DownloadCommand;
use info::InfoCommand;
use init::InitCommand;
use install::InstallCommand;
use list::ListCommand;
use nuke::NukeCommand;
use prefix::PrefixCommand;
use search::SearchCommand;
use shim::ShimCommand;
use status::StatusCommand;
use uninstall::UninstallCommand;
use update::UpdateCommand;
use which::WhichCommand;

pub const PREFIX_KEY: &'static str = "SCOOPIE_HOME";
pub const DEFAULT_PREFIX: &'static str = "scoopie";

#[derive(FromArgs, PartialEq, Debug)]
/// Scoopie, your simple package manager
pub struct Scoopie {
    #[argh(subcommand)]
    pub cmd: Command,
}

#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand)]
pub enum Command {
    Bucket(BucketCommand),
    Cache(CacheCommand),
    Cat(CatCommand),
    Cleanup(CleanupCommand),
    Download(DownloadCommand),
    Info(InfoCommand),
    Init(InitCommand),
    Install(InstallCommand),
    List(ListCommand),
    Nuke(NukeCommand),
    Prefix(PrefixCommand),
    Search(SearchCommand),
    Shim(ShimCommand),
    Status(StatusCommand),
    Uninstall(UninstallCommand),
    Update(UpdateCommand),
    Which(WhichCommand),
}

#[derive(Debug)]
pub enum ScoopieDirError {
    HomeDirError,
    ConfigDirError,
    EnvVarError(String),
    InvalidFormatError,
}

impl std::error::Error for ScoopieDirError {}

impl fmt::Display for ScoopieDirError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ScoopieDirError::HomeDirError => write!(f, "Unable to get $HOME dir."),
            ScoopieDirError::ConfigDirError => write!(f, "Unable to get $CONFIG dir."),
            ScoopieDirError::EnvVarError(var_name) => {
                write!(f, "Error retrieving environment variable: {}", var_name)
            }
            ScoopieDirError::InvalidFormatError => {
                write!(f, "Invalid format for environment variable.")
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct ScoopieInfo {
    pub home: PathBuf,
    pub apps: PathBuf,
    pub buckets: PathBuf,
    pub cache: PathBuf,
    pub persist: PathBuf,
    pub shims: PathBuf,
    pub config: PathBuf,
}

impl fmt::Display for ScoopieInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Scoopie at: {}\nConfigs at: {}",
            self.home.display(),
            self.config.display()
        )
    }
}

impl ScoopieInfo {
    pub fn get() -> Result<Self, ScoopieDirError> {
        let home = Self::prefix()?;
        let config = Self::config()?;
        let apps = home.join("apps");
        let buckets = home.join("buckets");
        let cache = home.join("cache");
        let persist = home.join("persist");
        let shims = home.join("shims");

        Ok(ScoopieInfo {
            home,
            apps,
            buckets,
            cache,
            persist,
            shims,
            config,
        })
    }

    pub fn prefix() -> Result<PathBuf, ScoopieDirError> {
        let home_dir = dirs::home_dir().ok_or(ScoopieDirError::HomeDirError)?;

        let scoopie_default_path = home_dir.join(DEFAULT_PREFIX);

        let path = match env::var(PREFIX_KEY) {
            Ok(path) => PathBuf::from(path),
            Err(e) => match e {
                env::VarError::NotPresent => scoopie_default_path,
                env::VarError::NotUnicode(_) => return Err(ScoopieDirError::InvalidFormatError),
            },
        };

        Ok(path)
    }

    pub fn config() -> Result<PathBuf, ScoopieDirError> {
        let config_dir = dirs::config_dir().ok_or(ScoopieDirError::ConfigDirError)?;
        let config_dir = config_dir.join(DEFAULT_PREFIX);
        Ok(config_dir)
    }
}
