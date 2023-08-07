use argh::FromArgs;

use super::prelude::*;
use crate::core::buckets::*;
use crate::error::ScoopieError;

#[derive(FromArgs, PartialEq, Debug)]
/// Search available apps from buckets (supports full-text search)
#[argh(subcommand, name = "query")]
pub struct QueryCommand {
    #[argh(positional)]
    query: String,
}

impl ExecuteCommand for QueryCommand {
    fn exec(&self) -> Result<(), ScoopieError> {
        let res = Buckets::query(QueryTerm::Regex(self.query.trim().into()))?;
        println!("{res}");
        Ok(())
    }
}
