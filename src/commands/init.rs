use argh::FromArgs;
use std::{fs::File, path::PathBuf};

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
            .try_for_each(|dir| PathBuf::create(&curr_dir.join(dir)))
    }

    fn init_scoopie(scoopie_path: &PathBuf) -> Result<(), ScoopieError> {
        let scoopie_dir = scoopie_path.join("apps/scoopie");

        if !scoopie_dir.exists() {
            PathBuf::create(&scoopie_dir)?;
            PathBuf::create(&scoopie_dir.join("bin"))?;
        }

        Ok(())
    }
}

impl ExecuteCommand for InitCommand {
    fn exec(&self) -> Result<(), ScoopieError> {
        let home_dir = Pwsh::home_dir()?;

        let scoopie_path = match &self.path {
            Some(path) => path.absolute()?,
            None => home_dir.clone(),
        }
        .join("scoopie");

        if scoopie_path.exists() {
            return Err(ScoopieError::DirAlreadyExists(scoopie_path));
        }

        Self::create_dirs(&scoopie_path)?;

        Self::init_scoopie(&scoopie_path)?;

        write_default_metadata()?;

        let config_dir = home_dir.join(".config");

        if !config_dir.exists() {
            PathBuf::create(&config_dir)?;
        }

        let scoopie_config = config_dir.join("scoopie.json");

        if !scoopie_config.exists() {
            Config::write(&scoopie_config)?;
        }

        Pwsh::create_or_update(
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

fn download(file: PathBuf, url: &str) -> Result<(), ScoopieError> {
    let req = minreq::get(url);
    let res = req.send().unwrap();

    let mut file = File::create(file).unwrap();

    std::io::copy(&mut res.as_bytes(), &mut file).unwrap();

    Ok(())
}
