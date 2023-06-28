use std::{error::Error, fs::remove_dir_all, path::PathBuf, process::Command, write};

use argh::FromArgs;

use crate::ScoopieInfo;
use crate::PREFIX_KEY;

#[derive(FromArgs, PartialEq, Debug)]
/// Say, Goodbye Scoopie!!
#[argh(subcommand, name = "nuke")]
pub struct NukeCommand {}

#[derive(Debug)]
pub enum NukeError {
    Failed(PathBuf, Box<dyn Error>),
    FileNotExist(PathBuf),
    EnvRemoveError,
    LackOfPermission,
}

impl std::error::Error for NukeError {}

impl std::fmt::Display for NukeError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            NukeError::Failed(d, e) => {
                write!(
                    f,
                    "Unknow error: {e:?} ocurred while deleting: {}.",
                    d.display()
                )
            }
            NukeError::FileNotExist(dir) => write!(f, "Dir: {} doesn't exists.", dir.display()),
            NukeError::LackOfPermission => {
                write!(f, "You don't have valid permissions to delete file.")
            }
            NukeError::EnvRemoveError => write!(f, "Unable to remove environment variable."),
        }
    }
}

impl NukeCommand {
    pub fn nuke(info: &ScoopieInfo) -> Result<(), NukeError> {
        let scoopie_home = &info.home;
        let config_path = &info.config;

        remove_dir_all(scoopie_home).map_err(|err| match err.kind() {
            std::io::ErrorKind::NotFound => NukeError::FileNotExist(scoopie_home.to_path_buf()),
            std::io::ErrorKind::PermissionDenied => NukeError::LackOfPermission,
            _ => NukeError::Failed(scoopie_home.to_path_buf(), Box::new(err)),
        })?;

        remove_dir_all(config_path).map_err(|err| match err.kind() {
            std::io::ErrorKind::NotFound => NukeError::FileNotExist(scoopie_home.to_path_buf()),
            std::io::ErrorKind::PermissionDenied => NukeError::LackOfPermission,
            _ => NukeError::Failed(scoopie_home.to_path_buf(), Box::new(err)),
        })?;

        let o = Command::new("cmd")
            .args(&[
                "/C",
                "REG",
                "delete",
                r"HKCU\Environment",
                "/F",
                "/V",
                PREFIX_KEY,
            ])
            .output()
            .map_err(|_| NukeError::EnvRemoveError)?;

        if !o.status.success() {
            return Err(NukeError::EnvRemoveError);
        }

        Ok(())
    }
}
