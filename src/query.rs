use std::{path::PathBuf, write};

use argh::FromArgs;
use rayon::prelude::*;
use rusqlite::{params, Connection, Error, Row};
use serde_json::{self, from_str, Value};

use crate::{
    config::*,
    error::{DatabaseError, QueryError, ScoopieError},
};

const APP_QUERY: &'static str =
    "SELECT app_name, manifest FROM manifests_fts WHERE app_name MATCH ? COLLATE nocase";
const FTS_QUERY: &'static str =
    "SELECT app_name, manifest FROM manifests_fts WHERE manifest MATCH ? COLLATE nocase";

#[derive(FromArgs, PartialEq, Debug)]
/// Search available apps from buckets (supports full-text search)
#[argh(subcommand, name = "query")]
pub struct QueryCommand {
    #[argh(positional)]
    query: String,
}

impl QueryCommand {
    pub fn from(query: QueryCommand) -> Result<Vec<AppInfo>, ScoopieError> {
        let q = query.query.trim();

        match q.contains(" ") {
            true => Self::query_fts(q),
            false => Self::query_app(q),
        }
    }

    fn query_app(app_name: &str) -> Result<Vec<AppInfo>, ScoopieError> {
        let results = Query::builder(APP_QUERY)?.run(&format!("{app_name}*"))?;

        results
            .par_iter()
            .map(|raw_result| AppInfo::from(raw_result))
            .collect()
    }

    fn query_fts(query: &str) -> Result<Vec<AppInfo>, ScoopieError> {
        let query = query
            .split_whitespace()
            .map(|term| format!("{term}*"))
            .collect::<Vec<String>>()
            .join(" AND ");

        let results = Query::builder(FTS_QUERY)?.run(&query)?;

        results
            .par_iter()
            .map(|raw_result| AppInfo::from(raw_result))
            .collect()
    }
}

pub struct AppInfo {
    repo_name: String,
    app_name: String,
    version: String,
    description: String,
}

impl AppInfo {
    fn from(data: &Data) -> Result<Self, ScoopieError> {
        let repo_name = data.repo_name.clone();
        let app_name = data.app_name.clone();

        let manifest: Value = from_str(&data.manifest)
            .map_err(|_| ScoopieError::Query(QueryError::InavlidJSONData))?;

        let description = manifest
            .get("description")
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string();

        let version = manifest
            .get("version")
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string();

        Ok(Self {
            repo_name,
            app_name,
            version,
            description,
        })
    }
}

impl std::fmt::Display for AppInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "{}/{} {}\n  {}\n",
            self.app_name, self.repo_name, self.version, self.description
        )
    }
}

pub struct Data {
    pub repo_name: String,
    pub app_name: String,
    pub manifest: String,
}

impl Data {
    fn new(repo: &PathBuf, app_name: String, manifest: String) -> Self {
        let db_name = &repo
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        let repo_name = db_name.split('-').next().unwrap_or_default().to_string();

        Self {
            repo_name,
            app_name,
            manifest,
        }
    }
}

pub struct Query {
    repos: Vec<PathBuf>,
    stmt: String,
}

impl Query {
    pub fn builder(stmt: &str) -> Result<Query, ScoopieError> {
        let repos = Config::read()?.latest_repos()?;

        Ok(Self {
            repos,
            stmt: stmt.into(),
        })
    }

    pub fn run(&self, params: &str) -> Result<Vec<Data>, ScoopieError> {
        let fetch_row = |row: &Row| -> Result<(String, String), Error> {
            let app_name = row.get(0)?;
            let manifest = row.get(1)?;
            Ok((app_name, manifest))
        };

        let results: Result<Vec<Vec<Data>>, ScoopieError> = self
            .repos
            .par_iter()
            .map(|repo| -> Result<Vec<Data>, ScoopieError> {
                let conn = Connection::open(&repo)
                    .map_err(|_| ScoopieError::Database(DatabaseError::UnableToOpen))?;

                let mut stmt = conn
                    .prepare(&self.stmt)
                    .map_err(|_| ScoopieError::Database(DatabaseError::FailedToMkStmt))?;

                let rows = stmt
                    .query_map(params![params], fetch_row)
                    .map_err(|_| ScoopieError::Query(QueryError::FailedToQuery))?;

                rows.into_iter()
                    .map(|row| -> Result<Data, ScoopieError> {
                        let row =
                            row.map_err(|_| ScoopieError::Query(QueryError::FailedToRetrieveData))?;

                        let app_name = row.0;
                        let manifest = row.1;

                        Ok(Data::new(&repo, app_name, manifest))
                    })
                    .collect()
            })
            .collect();

        results.and_then(|results| Ok(results.into_iter().flatten().collect()))
    }
}
