use std::path::PathBuf;

use argh::FromArgs;
use rayon::prelude::*;
use rusqlite::{params, Connection, Error, Row};
use serde::{Deserialize, Serialize};
use serde_json::{self, from_str, Value};

use crate::{
    bucket::MainfestEntry,
    config::Config,
    error::{DatabaseError, QueryError, ScoopieError},
};

#[derive(FromArgs, PartialEq, Debug)]
/// Search available apps (supports full-text search)
#[argh(subcommand, name = "query")]
pub struct QueryCommand {
    #[argh(positional)]
    query: String,
}

impl QueryCommand {
    pub fn from(query: QueryCommand) -> Result<Vec<AppInfo>, ScoopieError> {
        let q = query.query;
        let q = q.trim();

        match q.contains(" ") {
            true => Self::full_text_search(q),
            false => Self::query_app(q),
        }
    }

    fn full_text_search(query: &str) -> Result<Vec<AppInfo>, ScoopieError> {
        let results = Query::builder(
            "SELECT app_name, mainfest FROM mainfests_fts WHERE mainfest MATCH (?)",
        )?
        .run(query)?;

        results
            .par_iter()
            .map(|raw_result| AppInfo::from(raw_result))
            .collect()
    }

    fn query_app(app_name: &str) -> Result<Vec<AppInfo>, ScoopieError> {
        let results =
            Query::builder("SELECT app_name, mainfest FROM mainfests WHERE app_name LIKE ?")?
                .run(&format!("%{app_name}%"))?;

        results
            .par_iter()
            .map(|raw_result| AppInfo::from(raw_result))
            .collect()
    }
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
    fn from(raw: &RawData) -> Result<Self, ScoopieError> {
        let repo_name = raw.repo_name.clone();
        let app_name = raw.mainfest.app_name.clone();

        let mainfest: Value = from_str(&raw.mainfest.mainfest)
            .map_err(|_| ScoopieError::Query(QueryError::InavlidJSONData))?;

        let description = mainfest
            .get("description")
            .unwrap_or(&Value::Null)
            .to_string();

        let binaries = mainfest.get("bin").unwrap_or(&Value::Null).to_string();
        let version = mainfest.get("version").unwrap_or(&Value::Null).to_string();

        Ok(Self {
            repo_name,
            app_name,
            version,
            description,
            binaries,
        })
    }
}

#[derive(Debug, Clone)]
pub struct RawData {
    pub repo_name: String,
    pub mainfest: MainfestEntry,
}

impl RawData {
    fn new(repo: &PathBuf, app_name: String, mainfest: String) -> Self {
        let db_name = &repo
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        let repo_name = db_name.split('-').next().unwrap_or_default().to_string();

        let mainfest = MainfestEntry { app_name, mainfest };

        Self {
            repo_name,
            mainfest,
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

    pub fn run(&self, params: &str) -> Result<Vec<RawData>, ScoopieError> {
        let fetch_row = |row: &Row| -> Result<MainfestEntry, Error> {
            let app_name = row.get(0)?;
            let mainfest = row.get(1)?;
            Ok(MainfestEntry { app_name, mainfest })
        };

        let results: Result<Vec<Vec<RawData>>, ScoopieError> = self
            .repos
            .par_iter()
            .map(|repo| -> Result<Vec<RawData>, ScoopieError> {
                let conn = Connection::open(&repo)
                    .map_err(|_| ScoopieError::Database(DatabaseError::UnableToOpen))?;

                let mut stmt = conn
                    .prepare(&self.stmt)
                    .map_err(|_| ScoopieError::Database(DatabaseError::FailedToMkStmt))?;

                let rows = stmt
                    .query_map(params![params], fetch_row)
                    .map_err(|_| ScoopieError::Query(QueryError::FailedToQuery))?;

                rows.into_iter()
                    .map(|row| -> Result<RawData, ScoopieError> {
                        let row =
                            row.map_err(|_| ScoopieError::Query(QueryError::FailedToRetrieveData))?;

                        let app_name = row.app_name;
                        let mainfest = row.mainfest;

                        Ok(RawData::new(&repo, app_name, mainfest))
                    })
                    .collect()
            })
            .collect();

        let results: Vec<Vec<RawData>> = results?;
        Ok(results.concat())
    }
}
