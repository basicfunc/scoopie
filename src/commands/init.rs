use argh::FromArgs;
use dirs::home_dir;
use std::{
    fmt::Display,
    fs::DirBuilder,
    path::{Path, PathBuf},
    process::Command,
};

use crate::{
    core::config::*,
    error::{InitError, ScoopieError},
};

pub type InitResult = Result<ScoopieDirStats, ScoopieError>;

#[derive(FromArgs, PartialEq, Debug)]
/// Initialize Scoopie, useful while installing Scoopie itself
#[argh(subcommand, name = "init")]
pub struct InitCommand {
    #[argh(positional)]
    path: Option<PathBuf>,
}

#[derive(Debug)]
pub struct ScoopieDirStats {
    home: PathBuf,
    config: PathBuf,
}

impl Display for ScoopieDirStats {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "🎊 Congrats! Scoopie initialized.\nLocated at: {}\nConfig at: {}", self.home.display(), self.config.display())
    }
}

impl InitCommand {
    pub fn from(config: InitCommand) -> InitResult {
        let home_dir = home_dir().ok_or(ScoopieError::HomeDirUnavailable)?;

        let scoopie_path = match config.path {
            Some(path) => path.get_absolute()?,
            None => home_dir.clone(),
        }
        .join("scoopie");

        if scoopie_path.exists() {
            return Err(ScoopieError::DirAlreadyExists(scoopie_path));
        }

        let directories = vec![
            scoopie_path.clone(),
            scoopie_path.join("apps"),
            scoopie_path.join("buckets"),
            scoopie_path.join("cache"),
            scoopie_path.join("persists"),
            scoopie_path.join("shims"),
        ];

        directories.iter().try_for_each(|path| Path::mkdir(path))?;

        let config_dir = home_dir.join(".config");

        if !config_dir.exists() {
            Path::mkdir(&config_dir)?;
        }

        let scoopie_config = config_dir.join("scoopie.toml");

        if !scoopie_config.exists() {
            Config::write(&scoopie_config)?;
        }

        EnvVar::try_from(("SCOOPIE_HOME", scoopie_path.as_path().to_str().unwrap_or_default()))?;

        Ok(ScoopieDirStats {
            home: scoopie_path,
            config: scoopie_config,
        })
    }
}

trait Absolute {
    type Error;
    fn get_absolute(&self) -> Result<PathBuf, Self::Error>;
}

impl Absolute for PathBuf {
    type Error = ScoopieError;

    fn get_absolute(&self) -> Result<PathBuf, Self::Error> {
        let absolute_path = self.canonicalize().map_err(|_| ScoopieError::AbsoultePathResolve)?;
        let absolute_path_str = absolute_path.to_string_lossy().to_string();

        // Remove the `\\?\` prefix from the absolute path string
        Ok(PathBuf::from(match absolute_path_str.starts_with("\\\\?\\") {
            true => absolute_path_str[4..].to_string(),
            false => absolute_path_str,
        }))
    }
}

struct EnvVar(String, String);

impl TryFrom<(&str, &str)> for EnvVar {
    type Error = ScoopieError;

    fn try_from(value: (&str, &str)) -> Result<Self, Self::Error> {
        let name = value.0;
        let value = value.1;

        let cmd = Command::new("cmd")
            .args(&["/C", "setx", name, value])
            .status()
            .map_err(|_| ScoopieError::Init(InitError::UnableToSetEnvVar))?;

        match cmd.success() {
            true => Ok(EnvVar(name.into(), value.into())),
            false => Err(ScoopieError::Init(InitError::UnableToSetEnvVar)),
        }
    }
}

trait MkDir {
    type Error;
    fn mkdir(&self) -> Result<(), Self::Error>;
}

impl MkDir for Path {
    type Error = ScoopieError;
    fn mkdir(&self) -> Result<(), Self::Error> {
        DirBuilder::new().recursive(true).create(self).map_err(|_| ScoopieError::FailedToMkdir(self.to_path_buf()))
    }
}
