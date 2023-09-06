use argh::FromArgs;
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

impl InitCommand {
    fn create_dirs(curr_dir: &PathBuf) -> Result<(), ScoopieError> {
        ["apps", "buckets", "cache", "persists", "shims"]
            .iter()
            .try_for_each(|dir| PathBuf::create(curr_dir.join(dir)))
    }

    fn init_scoopie(scoopie_path: &PathBuf) -> Result<(), ScoopieError> {
        let scoopie_dir = scoopie_path.join("apps/scoopie");

        if !scoopie_dir.exists() {
            PathBuf::create(scoopie_dir)?;
        }

        Ok(())
    }

    fn install_7z(scoopie_dir: &PathBuf) -> Result<(), ScoopieError> {
        let url = "https://www.7-zip.org/a/7zr.exe";

        let request = minreq::get(url);

        let response = request.send().map_err(|_| ScoopieError::FailedToSendReq)?;

        Ok(())
    }
}

impl ExecuteCommand for InitCommand {
    fn exec(&self) -> Result<(), ScoopieError> {
        let home_dir = Env::home_dir()?;

        let scoopie_path = match &self.path {
            Some(path) => path.absolute()?,
            None => home_dir.clone(),
        }
        .join("scoopie");

        if scoopie_path.exists() {
            return Err(ScoopieError::DirAlreadyExists(scoopie_path));
        }

        Self::create_dirs(&scoopie_path)?;

        write_default_metadata()?;

        let config_dir = home_dir.join(".config");

        if !config_dir.exists() {
            PathBuf::create(config_dir.clone())?;
        }

        let scoopie_config = config_dir.join("scoopie.json");

        if !scoopie_config.exists() {
            Config::write(&scoopie_config)?;
        }

        Env::create_or_update(
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
