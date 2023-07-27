use argh::FromArgs;

use crate::{
    bucket::{data::BucketData, *},
    error::ScoopieError,
};

#[derive(FromArgs, PartialEq, Debug)]
/// Search available apps from buckets (supports full-text search)
#[argh(subcommand, name = "query")]
pub struct QueryCommand {
    #[argh(positional)]
    query: String,
}

impl TryFrom<QueryCommand> for BucketData {
    type Error = ScoopieError;

    fn try_from(value: QueryCommand) -> Result<Self, Self::Error> {
        let term = value.query.trim();

        match term.contains(" ") {
            true => {
                let query = term.split_whitespace().collect::<Vec<&str>>().join(" AND ");
                Bucket::query(QueryKind::FULLTEXT, format!("{query}*"))
            }
            false => Bucket::query(QueryKind::KEYWORD, format!("{term}*")),
        }
    }
}
