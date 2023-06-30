use argh::FromArgs;
use dirs::data_dir;
use std::{env, error::Error, fs::remove_dir_all, path::PathBuf, process::Command};

#[derive(FromArgs, PartialEq, Debug)]
/// Destorys all Scoopie related stuff
#[argh(subcommand, name = "nuke")]
pub struct NukeCommand {}

#[derive(Debug)]
pub enum NukeError {
    EnvResolve,
    FileNotExist(PathBuf),
    LackOfPermission,
    Failed(PathBuf, Box<dyn Error>),
    EnvRemoveError,
    DataDirUnavailable,
}

impl NukeCommand {
    pub fn from() -> Result<(), NukeError> {
        let scoopie_home = env::var("SCOOPIE_HOME").map_err(|_| NukeError::EnvResolve)?;
        let scoopie_home = PathBuf::from(scoopie_home);

        let data_dir = data_dir().ok_or(NukeError::DataDirUnavailable)?;
        let scoopie_data_dir = data_dir.join("scoopie");

        rmdir(&scoopie_home)?;
        rmdir(&scoopie_data_dir)?;
        remove_env_var("SCOOPIE_HOME")?;

        Ok(())
    }
}

fn rmdir(path: &PathBuf) -> Result<(), NukeError> {
    remove_dir_all(&path).map_err(|err| match err.kind() {
        std::io::ErrorKind::NotFound => NukeError::FileNotExist(path.to_path_buf()),
        std::io::ErrorKind::PermissionDenied => NukeError::LackOfPermission,
        _ => NukeError::Failed(path.to_path_buf(), Box::new(err)),
    })
}

fn remove_env_var(var: &str) -> Result<(), NukeError> {
    Command::new("cmd")
        .args(&["/C", "REG", "delete", r"HKCU\Environment", "/F", "/V", &var])
        .output()
        .map_err(|_| NukeError::EnvRemoveError)
        .and_then(|output| {
            if output.status.success() {
                Ok(())
            } else {
                Err(NukeError::EnvRemoveError)
            }
        })
}
