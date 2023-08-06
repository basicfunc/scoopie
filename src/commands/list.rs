use argh::FromArgs;

use super::prelude::*;
use crate::error::ScoopieError;

#[derive(FromArgs, PartialEq, Debug)]
/// List all installed apps
#[argh(subcommand, name = "list")]
pub struct ListCommand {}

impl ExecuteCommand for ListCommand {
    fn exec(&self) -> Result<(), ScoopieError> {
        todo!()
    }
}
