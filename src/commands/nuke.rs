use argh::FromArgs;
use std::{env, path::PathBuf};

use super::prelude::*;
use crate::error::ScoopieError;
use crate::utils::*;

#[derive(FromArgs, PartialEq, Debug)]
/// Destorys all Scoopie related stuff
#[argh(subcommand, name = "nuke")]
pub struct NukeCommand {}

impl ExecuteCommand for NukeCommand {
    fn exec(&self) -> Result<(), ScoopieError> {
        let scoopie_home = env::var("SCOOPIE_HOME").map_err(|_| ScoopieError::EnvResolve)?;
        PathBuf::from(scoopie_home).rm()?;
        EnvVar::remove("SCOOPIE_HOME")?;

        Ok(())
    }
}
