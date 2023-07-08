use argh::FromArgs;
use rayon::prelude::{IntoParallelRefIterator, ParallelIterator};
use regex::RegexBuilder;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use serde_json::{self, Value};

#[derive(FromArgs, PartialEq, Debug)]
/// Search available apps (supports regex and full-text search)
#[argh(subcommand, name = "query")]
pub struct QueryCommand {
    #[argh(positional)]
    query: String,
}

use crate::{
    config::Config,
    error::{DatabaseError, QueryError, ScoopieError},
};

#[derive(Debug)]
enum QueryType {
    AppName,
    Regex,
    FullText,
}

#[derive(Debug)]
struct Entry {
    app_name: String,
    mainfest: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AppInfo {
    repo_name: String,
    app_name: String,
    version: String,
    description: String,
    binaries: String,
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
    pub fn from(query: QueryCommand) -> Result<Vec<AppInfo>, ScoopieError> {
        let q = query.query;
        let q = q.trim();

        let result = match QueryType::from(q) {
            QueryType::AppName => Self::query_app(q),
            QueryType::Regex => Self::regex(q),
            QueryType::FullText => Self::full_text_search(q),
        };

        let result = result?;

        Ok(result.concat())
    }

    fn query_app(app_name: &str) -> Result<Vec<Vec<AppInfo>>, ScoopieError> {
        Self::query(&format!("%{app_name}%"))
    }

    fn full_text_search(query: &str) -> Result<Vec<Vec<AppInfo>>, ScoopieError> {
        println!("Query: {}", query);
        Ok(vec![])
    }

    fn regex(re: &str) -> Result<Vec<Vec<AppInfo>>, ScoopieError> {
        println!("Regex: {}", re);
        Ok(vec![])
    }

    fn query(app_name: &str) -> Result<Vec<Vec<AppInfo>>, ScoopieError> {
        let repos = Config::read()?.latest_repos()?;

        let all_results: Result<Vec<Vec<AppInfo>>, ScoopieError> = repos
            .par_iter()
            .map(|repo| -> Result<Vec<AppInfo>, ScoopieError> {
                let conn = Connection::open(&repo)
                    .map_err(|_| ScoopieError::Database(DatabaseError::UnableToOpen))?;

                let mut stmt = conn
                    .prepare("SELECT app_name, mainfest FROM mainfests WHERE app_name LIKE ?")
                    .map_err(|_| ScoopieError::Database(DatabaseError::FailedToMkStmt))?;

                let rows = stmt
                    .query_map(params![app_name], |row| {
                        let app_name = row.get(0)?;
                        let mainfest = row.get(1)?;
                        Ok(Entry { app_name, mainfest })
                    })
                    .map_err(|_| ScoopieError::Query(QueryError::FailedToQuery))?;

                let results: Result<Vec<AppInfo>, ScoopieError> = rows
                    .into_iter()
                    .map(|row| -> Result<AppInfo, ScoopieError> {
                        let row =
                            row.map_err(|_| ScoopieError::Query(QueryError::FailedToRetrieveData))?;

                        let app_name = row.app_name;
                        let mainfest: Value = serde_json::from_str(&row.mainfest)
                            .map_err(|_| ScoopieError::Query(QueryError::InavlidJSONData))?;

                        let db_name = &repo
                            .file_name()
                            .unwrap_or_default()
                            .to_string_lossy()
                            .to_string();
                        let repo_name = db_name.split('-').nth(0).unwrap_or_default().to_string();

                        let description = mainfest
                            .get("description")
                            .unwrap_or(&Value::Null)
                            .to_string();

                        let binaries = mainfest.get("bin").unwrap_or(&Value::Null).to_string();
                        let version = mainfest.get("version").unwrap_or(&Value::Null).to_string();

                        Ok(AppInfo {
                            repo_name,
                            app_name,
                            version,
                            description,
                            binaries,
                        })
                    })
                    .collect();

                results
            })
            .collect();

        all_results
    }
}
