use argh::FromArgs;
use dirs::home_dir;
use std::path::PathBuf;

use crate::{
    core::{buckets::write_default_metadata, config::*},
    error::ScoopieError,
    utils::*,
};

use super::prelude::*;

#[derive(FromArgs, PartialEq, Debug)]
/// Initialize Scoopie, useful while installing Scoopie itself
#[argh(subcommand, name = "init")]
pub struct InitCommand {
    #[argh(positional)]
    path: Option<PathBuf>,
}

impl ExecuteCommand for InitCommand {
    fn exec(&self) -> Result<(), ScoopieError> {
        let home_dir = home_dir().ok_or(ScoopieError::HomeDirUnavailable)?;

        let scoopie_path = match &self.path {
            Some(path) => path.absolute()?,
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

        directories.into_iter().try_for_each(PathBuf::create)?;

        write_default_metadata()?;

        let config_dir = home_dir.join(".config");

        if !config_dir.exists() {
            PathBuf::create(config_dir.clone())?;
        }

        let scoopie_config = config_dir.join("scoopie.json");

        if !scoopie_config.exists() {
            Config::write(&scoopie_config)?;
        }

        EnvVar::create_or_update(
            "SCOOPIE_HOME",
            scoopie_path.as_path().to_str().unwrap_or_default(),
        )?;

        println!(
            "🎊 Congrats! Scoopie initialized.\nLocated at: {}\nConfig at: {}",
            scoopie_path.display(),
            scoopie_config.display()
        );

        Ok(())
    }
}
