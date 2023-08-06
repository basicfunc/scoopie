use std::unimplemented;

use argh::FromArgs;

use super::prelude::*;

use crate::error::ScoopieError;

#[derive(FromArgs, PartialEq, Debug)]
/// Shows information related to specified app
#[argh(subcommand, name = "info")]
pub struct InfoCommand {
    #[argh(positional)]
    app: Option<String>,

    #[argh(switch)]
    /// show mainfest of app
    show_mainfest: bool,
}
impl ExecuteCommand for InfoCommand {
    fn exec(&self) -> Result<(), ScoopieError> {
        println!("{:?}", self);

        unimplemented!();
    }
}
