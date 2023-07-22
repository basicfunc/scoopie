use argh::FromArgs;
use lazy_static::lazy_static;

use crate::{bucket::*, error::ScoopieError};

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

impl QueryCommand {
    pub fn from(query: QueryCommand) -> Result<Vec<RawData>, ScoopieError> {
        let q = query.query.trim();

        match q.contains(" ") {
            true => Self::query_fts(q),
            false => Self::query_app(q),
        }
    }

    fn query_app(app_name: &str) -> Result<Vec<RawData>, ScoopieError> {
        Bucket::build_query(*APP_QUERY)?.execute(format!("{app_name}*"))
    }

    fn query_fts(terms: &str) -> Result<Vec<RawData>, ScoopieError> {
        let query = terms
            .split_whitespace()
            .collect::<Vec<&str>>()
            .join(" AND ");
        Bucket::build_query(*FTS_QUERY)?.execute(format!("{query}*"))
    }
}
