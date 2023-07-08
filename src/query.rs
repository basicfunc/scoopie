use std::path::PathBuf;

use argh::FromArgs;
use rayon::prelude::{IntoParallelRefIterator, ParallelIterator};
use rusqlite::{params, Connection, Error, Row};
use serde::{Deserialize, Serialize};
use serde_json::{self, Value};

#[derive(FromArgs, PartialEq, Debug)]
/// Search available apps (supports full-text search)
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
struct QueryResult {
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

impl AppInfo {
    fn from(repo: &PathBuf, value: QueryResult) -> Result<AppInfo, ScoopieError> {
        let app_name = value.app_name;
        let mainfest: Value = serde_json::from_str(&value.mainfest)
            .map_err(|_| ScoopieError::Query(QueryError::InavlidJSONData))?;

        let db_name = &repo
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        let repo_name = db_name.split('-').next().unwrap_or_default().to_string();

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
    }
}

impl QueryCommand {
    pub fn from(query: QueryCommand) -> Result<Vec<AppInfo>, ScoopieError> {
        let q = query.query;
        let q = q.trim();

        let result = match q.contains(" ") {
            true => Self::full_text_search(q),
            false => Self::query_app(q),
        };

        let result = result?;

        Ok(result.concat())
    }

    fn full_text_search(query: &str) -> Result<Vec<Vec<AppInfo>>, ScoopieError> {
        Query::builder("SELECT app_name, mainfest FROM mainfests_fts WHERE mainfest MATCH (?)")?
            .run(query)
    }

    fn query_app(app_name: &str) -> Result<Vec<Vec<AppInfo>>, ScoopieError> {
        Query::builder("SELECT app_name, mainfest FROM mainfests WHERE app_name LIKE ?")?
            .run(&format!("%{app_name}%"))
    }
}

struct Query {
    repos: Vec<PathBuf>,
    stmt: String,
}

impl Query {
    fn builder(stmt: &str) -> Result<Query, ScoopieError> {
        let repos = Config::read()?.latest_repos()?;
        Ok(Self {
            repos,
            stmt: stmt.into(),
        })
    }

    fn run(&self, params: &str) -> Result<Vec<Vec<AppInfo>>, ScoopieError> {
        let fetch_row = |row: &Row| -> Result<QueryResult, Error> {
            let app_name = row.get(0)?;
            let mainfest = row.get(1)?;
            Ok(QueryResult { app_name, mainfest })
        };

        self.repos
            .par_iter()
            .map(|repo| -> Result<Vec<AppInfo>, ScoopieError> {
                let conn = Connection::open(&repo)
                    .map_err(|_| ScoopieError::Database(DatabaseError::UnableToOpen))?;

                let mut stmt = conn
                    .prepare(&self.stmt)
                    .map_err(|_| ScoopieError::Database(DatabaseError::FailedToMkStmt))?;

                let rows = stmt
                    .query_map(params![params], fetch_row)
                    .map_err(|_| ScoopieError::Query(QueryError::FailedToQuery))?;

                rows.into_iter()
                    .map(|row| -> Result<AppInfo, ScoopieError> {
                        let row =
                            row.map_err(|_| ScoopieError::Query(QueryError::FailedToRetrieveData))?;

                        AppInfo::from(&repo, row)
                    })
                    .collect()
            })
            .collect()
    }
}
