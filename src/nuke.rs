use argh::FromArgs;
use std::{env, fs::remove_dir_all, path::PathBuf, process::Command};

use crate::error::ScoopieError;

pub type NukeResult = Result<(), ScoopieError>;

#[derive(FromArgs, PartialEq, Debug)]
/// Destorys all Scoopie related stuff
#[argh(subcommand, name = "nuke")]
pub struct NukeCommand {}

impl NukeCommand {
    pub fn from() -> NukeResult {
        let scoopie_home = env::var("SCOOPIE_HOME").map_err(|_| ScoopieError::EnvResolve)?;
        let scoopie_home = PathBuf::from(scoopie_home);

        rmdir(&scoopie_home)?;
        remove_env_var("SCOOPIE_HOME")?;

        Ok(())
    }
}

fn rmdir(path: &PathBuf) -> NukeResult {
    remove_dir_all(&path).map_err(|err| match err.kind() {
        std::io::ErrorKind::NotFound => ScoopieError::FileNotExist(path.to_path_buf()),
        std::io::ErrorKind::PermissionDenied => ScoopieError::PermissionDenied,
        _ => ScoopieError::Unknown,
    })
}

fn remove_env_var(var: &str) -> NukeResult {
    Command::new("cmd")
        .args(&["/C", "REG", "delete", r"HKCU\Environment", "/F", "/V", &var])
        .output()
        .map_err(|_| ScoopieError::EnvRemove)
        .and_then(|output| {
            if output.status.success() {
                Ok(())
            } else {
                Err(ScoopieError::EnvRemove)
            }
        })
}
