use std::path::PathBuf;

use argh::FromArgs;
use dirs::data_dir;
use regex::RegexBuilder;

#[derive(FromArgs, PartialEq, Debug)]
/// Search available apps (supports regex and full-text search)
#[argh(subcommand, name = "query")]
pub struct QueryCommand {
    #[argh(positional)]
    query: String,
}

use crate::error::{QueryError, ScoopieError};

#[derive(Debug)]
enum QueryType {
    AppName,
    Regex,
    FullText,
}

impl QueryType {
    fn from(query: &str) -> QueryType {
        if RegexBuilder::new(query)
            .build()
            .is_ok_and(|x| x.is_match(""))
        {
            QueryType::Regex
        } else if query.contains(" ") {
            QueryType::FullText
        } else {
            QueryType::AppName
        }
    }
}

impl QueryCommand {
    pub fn from(query: QueryCommand) {
        let q = query.query;
        let q = q.trim();
        // let q = q.trim_matches('"');

        let result = match QueryType::from(q) {
            QueryType::AppName => Self::query_app(q),
            QueryType::Regex => Self::regex(q),
            QueryType::FullText => Self::full_text_search(q),
        };

        // Ok(())
    }

    fn query_app(app_name: &str) -> Result<(), QueryError> {
        println!("App: {}", app_name);
        Ok(())
    }

    fn full_text_search(query: &str) -> Result<(), QueryError> {
        println!("Query: {}", query);
        Ok(())
    }

    fn regex(re: &str) -> Result<(), QueryError> {
        println!("Regex: {}", re);
        Ok(())
    }
}

fn get_dbs() -> Result<Vec<PathBuf>, ScoopieError> {
    let data_dir = data_dir().ok_or(ScoopieError::DataDirUnavailable)?;
    let scoopie_dir = data_dir.join("scoopie");

    if !scoopie_dir.exists() {
        return Err(ScoopieError::Query(
            QueryError::ScoopieWorkingDirUnavailable,
        ));
    }

    let repo_dir = scoopie_dir.join("repos");

    if !repo_dir.exists() || !repo_dir.is_dir() {
        return Err(ScoopieError::Query(QueryError::ReposDirUnavailable));
    }

    let db_list: Vec<PathBuf> = Vec::new();

    Ok(db_list)
}
