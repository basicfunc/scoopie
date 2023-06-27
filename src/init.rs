use crate::{DEFAULT_PREFIX, PREFIX_KEY};
use argh::FromArgs;
use dirs::{config_dir, home_dir};
use std::{fs::DirBuilder, path::PathBuf, process::Command};

#[derive(FromArgs, PartialEq, Debug)]
/// Initialize all scoopie related stuff.
#[argh(subcommand, name = "init")]
pub struct InitCommand {
    #[argh(option)]
    /// path where you would like to give home to scoopie.
    path: Option<String>,
}

#[derive(Debug)]
pub enum InitErrors {
    DirAlreadyExists(PathBuf),
    HomeDirUnavailable,
    ConfigDirUnavailable,
    UnableToCreateDir(PathBuf),
    UnableToSetEnvVar,
}

impl std::error::Error for InitErrors {}

impl std::fmt::Display for InitErrors {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::DirAlreadyExists(p) => write!(f, "Directory: {} already exists.", p.display()),
            Self::HomeDirUnavailable => write!(f, "Unable to get HOME directory."),
            Self::ConfigDirUnavailable => write!(f, "Unable to get CONFIG directory."),
            Self::UnableToCreateDir(p) => write!(f, "Unable to create directory: {}.", p.display()),
            Self::UnableToSetEnvVar => {
                write!(f, "Unable to set environment variable $SCOOPIE_HOME")
            }
        }
    }
}

pub struct InitSuccess {
    home: PathBuf,
    config: PathBuf,
}

impl std::fmt::Display for InitSuccess {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let success = "🎉 Successfully initialized Scoopie. You can now use scoopie!!";
        let info = format!(
            "INFO: Scoopie is located in {} and its configs are located at {}",
            self.home.display(),
            self.config.display()
        );

        write!(f, "{}\n{}", success, info)
    }
}

impl InitCommand {
    pub fn from(config: &InitCommand) -> Result<InitSuccess, InitErrors> {
        let path = config.path.clone();

        let home_dir = home_dir().ok_or(InitErrors::HomeDirUnavailable)?;

        let config_dir = config_dir().ok_or(InitErrors::ConfigDirUnavailable)?;
        let config_dir = config_dir.join(DEFAULT_PREFIX);

        let path = path
            .map(PathBuf::from)
            .unwrap_or_else(|| home_dir.join(DEFAULT_PREFIX));

        if path.exists() {
            return Err(InitErrors::DirAlreadyExists(path.clone()));
        }

        DirBuilder::new()
            .recursive(true)
            .create(&path)
            .map_err(|_| InitErrors::UnableToCreateDir(path.clone()))?;

        DirBuilder::new()
            .recursive(true)
            .create(&config_dir)
            .map_err(|_| InitErrors::UnableToCreateDir(config_dir.clone()))?;

        let value = path.display().to_string();

        let output = Command::new("cmd")
            .args(&["/C", "setx", PREFIX_KEY, &value])
            .output()
            .map_err(|_| InitErrors::UnableToSetEnvVar)?;

        if !output.status.success() {
            return Err(InitErrors::UnableToSetEnvVar);
        }

        Ok(InitSuccess {
            home: path,
            config: config_dir,
        })
    }
}
