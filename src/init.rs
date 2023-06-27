use argh::FromArgs;
use dirs::{config_dir, home_dir};
use std::{fs::create_dir, path::PathBuf, process::Command, write};

#[derive(FromArgs, PartialEq, Debug)]
/// Initialize all scoopie related stuff.
#[argh(subcommand, name = "init")]
pub struct InitCommand {
    #[argh(option)]
    /// path where you would like give home to scoopie.
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
            Self::DirAlreadyExists(p) => write!(f, "Directory: {} Already exists.", p.display()),
            Self::HomeDirUnavailable => write!(f, "Unable to get HOME directory."),
            Self::ConfigDirUnavailable => write!(f, "Unable to get CONFIG directory."),
            Self::UnableToCreateDir(p) => write!(f, "Unable to create Directory: {}.", p.display()),
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
        let sucess = "🎉 Successfully initialized Scoopie.You can now use scoopie!!";
        let info = format!(
            "INFO: Scoopie is located in {} and its configs are located at {}",
            self.home.display(),
            self.config.display()
        );

        write!(f, "{sucess}\n{info}")
    }
}

impl InitCommand {
    pub fn from(config: &InitCommand) -> Result<InitSuccess, InitErrors> {
        let path = &config.path;

        let home_dir = match home_dir() {
            Some(home_dir) => home_dir,
            None => return Err(InitErrors::HomeDirUnavailable),
        };

        let config_dir = match config_dir() {
            Some(config_dir) => PathBuf::from(format!("{}\\scoopie", config_dir.display())),
            None => return Err(InitErrors::ConfigDirUnavailable),
        };

        let path = match path {
            Some(x) => PathBuf::from(x),
            None => PathBuf::from(format!("{}\\scoopie", home_dir.display())),
        };

        if path.exists() {
            return Err(InitErrors::DirAlreadyExists(path));
        }

        match create_dir(&path) {
            Ok(()) => (),
            Err(_) => return Err(InitErrors::UnableToCreateDir(path)),
        }

        match create_dir(&config_dir) {
            Ok(()) => (),
            Err(_) => return Err(InitErrors::UnableToCreateDir(config_dir)),
        }

        let value = path.clone().display().to_string();

        match Command::new("cmd")
            .args(&["/C", "setx", "SCOOPIE_HOME", &value])
            .output()
        {
            Ok(o) => {
                if !o.status.success() {
                    return Err(InitErrors::UnableToSetEnvVar);
                }
            }
            Err(_) => return Err(InitErrors::UnableToSetEnvVar),
        };

        Ok(InitSuccess {
            home: path,
            config: config_dir,
        })
    }
}
