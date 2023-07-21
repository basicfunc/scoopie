use std::{
    fmt::{self, Display},
    path::PathBuf,
    write,
};

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
        let results = Query::builder(APP_QUERY)?.run(format!("{app_name}*"))?;
        results.par_iter().map(|d| d.try_into()).collect()
    }

    fn query_fts(terms: &str) -> Result<Vec<AppInfo>, ScoopieError> {
        let query = terms
            .split_whitespace()
            .collect::<Vec<&str>>()
            .join(" AND ");
        let results = Query::builder(FTS_QUERY)?.run(format!("{query}*"))?;
        results.par_iter().map(|d| d.try_into()).collect()
    }
}

pub struct AppInfo {
    repo_name: String,
    app_name: String,
    version: String,
    description: String,
}

impl TryFrom<&Data> for AppInfo {
    type Error = ScoopieError;

    fn try_from(value: &Data) -> Result<Self, ScoopieError> {
        let repo_name = value.repo_name.clone();
        let app_name = value.app_name.clone();

        let manifest: Value = from_str(&value.manifest)
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

impl Display for AppInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "\n{}/{} {}\n  {}",
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
    pub fn builder<T>(stmt: T) -> Result<Query, ScoopieError>
    where
        T: Into<String>,
    {
        let repos = Config::read()?.latest_buckets()?;

        Ok(Self {
            repos,
            stmt: stmt.into(),
        })
    }

    pub fn run<T>(&self, params: T) -> Result<Vec<Data>, ScoopieError>
    where
        T: Into<String>,
    {
        let params = params.into();

        let fetch_row = |row: &Row| -> Result<(String, String), Error> {
            let app_name = row.get(0)?;
            let manifest = row.get(1)?;
            Ok((app_name, manifest))
        };

        let process_row =
            |repo: &PathBuf, row: Result<(String, String), Error>| -> Result<Data, ScoopieError> {
                let row = row.map_err(|_| ScoopieError::Query(QueryError::FailedToRetrieveData))?;
                let app_name = row.0;
                let manifest = row.1;
                Ok(Data::new(repo, app_name, manifest))
            };

        let fetch_data = |repo: &PathBuf| -> Result<Vec<Data>, ScoopieError> {
            let conn = Connection::open(repo)
                .map_err(|_| ScoopieError::Database(DatabaseError::UnableToOpen))?;

            let mut stmt = conn
                .prepare(&self.stmt)
                .map_err(|_| ScoopieError::Database(DatabaseError::FailedToMkStmt))?;

            let rows = stmt
                .query_map(params![params], fetch_row)
                .map_err(|_| ScoopieError::Query(QueryError::FailedToQuery))?;

            rows.into_iter()
                .map(|row| process_row(&repo, row))
                .collect()
        };

        let results: Result<Vec<Vec<Data>>, ScoopieError> =
            self.repos.par_iter().map(fetch_data).collect();

        results.and_then(|results| Ok(results.into_iter().flatten().collect()))
    }
}
