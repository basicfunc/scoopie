use argh::FromArgs;

use super::prelude::*;
use crate::error::ScoopieError;

#[derive(FromArgs, PartialEq, Debug)]
/// Shows the location of specified app
#[argh(subcommand, name = "locate")]
pub struct LocateCommand {
    #[argh(positional)]
    app: String,
}

impl ExecuteCommand for LocateCommand {
    fn exec(&self) -> Result<(), ScoopieError> {
        todo!()
    }
}
