use argh::FromArgs;
use lazy_static::lazy_static;

use crate::{
    bucket::{data::BucketData, *},
    error::ScoopieError,
};

lazy_static! {
    static ref APP_QUERY: &'static str =
        "SELECT app_name, manifest FROM manifests_fts WHERE app_name MATCH ? COLLATE nocase";
    static ref FTS_QUERY: &'static str =
        "SELECT app_name, manifest FROM manifests_fts WHERE manifest MATCH ? COLLATE nocase";
}

#[derive(FromArgs, PartialEq, Debug)]
/// Search available apps from buckets (supports full-text search)
#[argh(subcommand, name = "query")]
pub struct QueryCommand {
    #[argh(positional)]
    query: String,
}

pub trait Query<T> {
    type Error;
    fn direct(app_name: T) -> Result<BucketData, Self::Error>;
    fn full_text(terms: T) -> Result<BucketData, Self::Error>;
}

impl Query<&str> for QueryCommand {
    type Error = ScoopieError;

    fn direct(app_name: &str) -> Result<BucketData, Self::Error> {
        Bucket::build_query(*APP_QUERY)?.execute(format!("{app_name}*"))
    }

    fn full_text(terms: &str) -> Result<BucketData, Self::Error> {
        let query = terms
            .split_whitespace()
            .collect::<Vec<&str>>()
            .join(" AND ");
        Bucket::build_query(*FTS_QUERY)?.execute(format!("{query}*"))
    }
}

impl TryFrom<QueryCommand> for BucketData {
    type Error = ScoopieError;

    fn try_from(value: QueryCommand) -> Result<Self, Self::Error> {
        let q = value.query.trim();

        match q.contains(" ") {
            true => QueryCommand::full_text(q),
            false => QueryCommand::direct(q),
        }
    }
}
