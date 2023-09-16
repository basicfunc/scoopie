use std::{path::PathBuf, process::Command};

use crate::error::ScoopieError;

pub struct Pwsh;

impl Pwsh {
    pub fn home_dir() -> Result<PathBuf, ScoopieError> {
        let cmd = Command::new("powershell.exe")
            .args([
                "-NoLogo",
                "-NoProfile",
                "-NonInteractive",
                "-Command",
                "$env:UserProfile",
            ])
            .output()
            .map_err(|_| ScoopieError::UnableToExecuteCmd)?;

        if cmd.status.success() {
            let home_dir = String::from_utf8(cmd.stdout).map_err(|_| ScoopieError::NonUTF8Bytes)?;
            let home_dir = home_dir.trim_end_matches("\r\n");
            Ok(PathBuf::from(home_dir))
        } else {
            Err(ScoopieError::UserDirUnavailable)
        }
    }

    pub fn create_or_update(key: &str, value: &str) -> Result<(), ScoopieError> {
        let cmd = Command::new("powershell.exe")
            .args([
                "-NoLogo",
                "-NoProfile",
                "-NonInteractive",
                "-Command",
                &format!("reg add HKCU\\Environment /v {key} /t REG_SZ /d {value} /f"),
            ])
            .output()
            .map_err(|_| ScoopieError::UnableToExecuteCmd)?;

        if cmd.status.success() {
            Ok(())
        } else {
            Err(ScoopieError::EnvSet)
        }
    }

    pub fn remove(key: &str) -> Result<(), ScoopieError> {
        let cmd = Command::new("powershell.exe")
            .args([
                "-NoLogo",
                "-NoProfile",
                "-NonInteractive",
                "-Command",
                &format!("reg add HKCU\\Environment /v {key} /f"),
            ])
            .output()
            .map_err(|_| ScoopieError::UnableToExecuteCmd)?;

        if cmd.status.success() {
            Ok(())
        } else {
            Err(ScoopieError::EnvRemove)
        }
    }

    pub fn run(profile: Option<&String>, prog: &str) -> Result<String, ScoopieError> {
        let mut cmd = Command::new("powershell.exe");

        let args = match profile {
            Some(psf) => vec![
                "-NoLogo",
                "-NoProfile",
                "-NonInteractive",
                "-PSConsoleFile",
                psf,
                "-Command",
                prog,
            ],
            None => vec!["-NoLogo", "-NoProfile", "-NonInteractive", "-Command", prog],
        };

        let cmd = cmd.args(args);

        let result = cmd.output().unwrap();

        Ok(String::from_utf8(result.stdout).unwrap())
    }
}
