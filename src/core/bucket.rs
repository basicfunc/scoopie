use chrono::{DateTime, Local};
use git2::build::RepoBuilder;
use lazy_static::lazy_static;
use rayon::prelude::*;
use rusqlite::{params, Connection};
use tempfile::tempdir;

use std::{collections::HashMap, fmt, fs, path::PathBuf, write};

use super::{config::*, data::*};
use crate::error::*;

lazy_static! {
    /// For Creating various Tables
    static ref METADATA_TABLE_CREATE_STMT: &'static str = "CREATE TABLE IF NOT EXISTS metadata (
                bucket_name VARCHAR(255),
                commit_id VARCHAR(100) NOT NULL PRIMARY KEY,
                url VARCHAR(1000),
                date DATE,
                time TIME,
                number_of_manifests INT)";
    static ref TABLE_CREATE_STMT: &'static str = "CREATE TABLE IF NOT EXISTS manifests (
                app_name TEXT NOT NULL PRIMARY KEY,
                manifest TEXT)";
    static ref FTS_TABLE_CREATE_STMT: &'static str = "CREATE VIRTUAL TABLE IF NOT EXISTS manifests_fts USING FTS5(app_name, manifest)";

    /// For Inserting data into various Tables
    static ref METADATA_INSERT_STMT: &'static str = "INSERT INTO metadata (bucket_Name, commit_id, url, date, time, number_of_manifests)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)";
    static ref INSERT_STMT: &'static str = "INSERT INTO manifests (app_name, manifest) VALUES (?, ?)";
    static ref FTS_INSERT_STMT: &'static str = "INSERT INTO manifests_fts(app_name, manifest) SELECT app_name, manifest FROM manifests";

    /// For querying Tables
    static ref APP_QUERY: &'static str = "SELECT app_name, manifest FROM manifests WHERE app_name LIKE ?";
    static ref KEYWORD_QUERY: &'static str = "SELECT app_name, manifest FROM manifests_fts WHERE app_name MATCH ? COLLATE nocase";
    static ref FTS_QUERY: &'static str = "SELECT app_name, manifest FROM manifests_fts WHERE manifest MATCH ? COLLATE nocase";

    /// For getting latest commit id from metadata table
    static ref COMMIT_QUERY: &'static str = "SELECT commit_id FROM metadata LIMIT 1";
}

#[derive(Debug)]
pub struct Bucket {}

#[derive(Debug, PartialEq, PartialOrd)]
pub enum SyncStatus {
    UpToDate(String),
    Synced(String),
    Created(String),
}

struct MetaData {
    bucket_name: String,
    commit_id: String,
    url: String,
    date: String,
    time: String,
    no_of_manifests: usize,
}

pub trait SyncFrom<T> {
    type Error;
    fn sync_from(value: T) -> Result<SyncStatus, Self::Error>;
}

impl SyncFrom<(&String, &String)> for Bucket {
    type Error = ScoopieError;

    fn sync_from(value: (&String, &String)) -> Result<SyncStatus, Self::Error> {
        let (bucket_name, url) = value;

        let temp_dir = tempdir().map_err(|_| ScoopieError::Sync(SyncError::UnableToMkTmpDir))?;
        let path = temp_dir.path();

        let repo = RepoBuilder::new()
            .clone(&url, &path)
            .map_err(|_| ScoopieError::Sync(SyncError::UnableToFetchRepo))?;

        let head = repo
            .head()
            .map_err(|_| ScoopieError::Sync(SyncError::UnableToGetHead))?;

        let commit_id = head
            .peel_to_commit()
            .map_err(|_| ScoopieError::Sync(SyncError::UnableToGetCommit))?
            .id()
            .to_string();

        let bucket_path = path.join("bucket");

        if !bucket_path.is_dir() {
            return Err(ScoopieError::Bucket(BucketError::NotFound));
        }

        let entries = fs::read_dir(&bucket_path)
            .map_err(|_| ScoopieError::Bucket(BucketError::BucketsNotFound))?
            .par_bridge()
            .map(Entry::try_maybe_from)
            .collect::<Result<Vec<_>, _>>()?;

        let entries: Vec<Entry> = entries.into_par_iter().filter_map(|entry| entry).collect();

        let curr: DateTime<Local> = Local::now();

        let metadata = MetaData {
            bucket_name: bucket_name.into(),
            commit_id,
            url: url.into(),
            date: curr.format("%Y-%m-%d").to_string(),
            time: curr.format("%H:%M:%S").to_string(),
            no_of_manifests: entries.len(),
        };

        let mut bucket_path = PathBuf::from(&bucket_name);
        bucket_path.set_extension("db");

        let bucket_path = Config::buckets_dir()?.join(&bucket_path);

        let create_bucket = |path: &PathBuf| -> Result<String, ScoopieError> {
            let conn = Connection::open(&path)
                .map_err(|_| ScoopieError::Database(DatabaseError::UnableToOpen))?;

            conn.execute(*METADATA_TABLE_CREATE_STMT, [])
                .map_err(|_| ScoopieError::Database(DatabaseError::FailedToCreateTable))?;

            conn.execute(
                *METADATA_INSERT_STMT,
                params![
                    &metadata.bucket_name,
                    &metadata.commit_id,
                    &metadata.url,
                    &metadata.date,
                    &metadata.time,
                    &metadata.no_of_manifests
                ],
            )
            .map_err(|_| ScoopieError::Database(DatabaseError::FailedInsertion))?;

            conn.execute(*TABLE_CREATE_STMT, [])
                .map_err(|_| ScoopieError::Database(DatabaseError::FailedToCreateTable))?;

            let mut stmt = conn
                .prepare(*INSERT_STMT)
                .map_err(|_| ScoopieError::Database(DatabaseError::FailedToMkStmt))?;

            entries
                .iter()
                .try_for_each(|e| -> Result<(), ScoopieError> {
                    let manifest = serde_json::to_value(&e.manifest).unwrap();
                    stmt.execute(params![&e.app_name, &manifest])
                        .map_err(|_| ScoopieError::Database(DatabaseError::FailedInsertion))?;

                    Ok(())
                })?;

            conn.execute(*FTS_TABLE_CREATE_STMT, [])
                .map_err(|_| ScoopieError::Database(DatabaseError::FailedToCreateTable))?;

            conn.execute(*FTS_INSERT_STMT, [])
                .map_err(|_| ScoopieError::Database(DatabaseError::FailedInsertion))?;
            Ok(bucket_name.into())
        };

        match bucket_path.exists() {
            true => match Bucket::query_commit(&bucket_name)? == metadata.commit_id {
                true => Ok(SyncStatus::UpToDate(bucket_name.into())),
                false => fs::remove_file(&bucket_path)
                    .map_err(|_| ScoopieError::Unknown)
                    .and_then(|_| Ok(SyncStatus::Synced(create_bucket(&bucket_path)?))),
            },
            false => Ok(SyncStatus::Created(create_bucket(&bucket_path)?)),
        }
    }
}

impl std::fmt::Display for SyncStatus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            SyncStatus::UpToDate(bucket_name) => write!(f, "{bucket_name} is already up-to-date"),
            SyncStatus::Created(bucket_name) => write!(f, "Created new {bucket_name}"),
            SyncStatus::Synced(bucket_name) => write!(f, "Synced {bucket_name} to the remote"),
        }
    }
}

pub trait Query<T> {
    type Error;
    fn query(kind: T, params: String) -> Result<BucketData, Self::Error>;
    fn query_commit(bucket_name: &str) -> Result<String, Self::Error>;
}

pub enum QueryKind {
    FULLTEXT,
    KEYWORD,
    APP,
}

impl Query<QueryKind> for Bucket {
    type Error = ScoopieError;

    fn query(kind: QueryKind, params: String) -> Result<BucketData, Self::Error> {
        let query_stmt = match kind {
            QueryKind::FULLTEXT => *FTS_QUERY,
            QueryKind::KEYWORD => *KEYWORD_QUERY,
            QueryKind::APP => *APP_QUERY,
        };

        let query_each = |bucket_path: &PathBuf| -> Result<(String, Vec<Entry>), ScoopieError> {
            let conn = Connection::open(bucket_path)
                .map_err(|_| ScoopieError::Database(DatabaseError::UnableToOpen))?;

            let mut stmt = conn
                .prepare(query_stmt)
                .map_err(|_| ScoopieError::Database(DatabaseError::FailedToMkStmt))?;

            let entries: Vec<Entry> = stmt
                .query_map(params![params], |row| Entry::try_from(row))
                .map_err(|_| ScoopieError::Query(QueryError::FailedToRetrieveData))?
                .flat_map(Result::ok)
                .collect();

            let bucket_name = bucket_path
                .file_stem()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();

            Ok((bucket_name, entries))
        };

        let entries = Config::read()?
            .latest_buckets()?
            .par_iter()
            .map(query_each)
            .collect::<Result<HashMap<_, Vec<_>>, _>>()?;
        Ok(BucketData::from(entries))
    }

    fn query_commit(bucket_name: &str) -> Result<String, Self::Error> {
        let mut bucket_path = Config::buckets_dir()?.join(bucket_name);
        bucket_path.set_extension("db");

        let conn = Connection::open(bucket_path)
            .map_err(|_| ScoopieError::Database(DatabaseError::UnableToOpen))?;

        let mut stmt = conn
            .prepare(*COMMIT_QUERY)
            .map_err(|_| ScoopieError::Database(DatabaseError::FailedToMkStmt))?;

        let commit_id = stmt.query_map([], |row| {
            let id: String = row.get(0)?;
            Ok(id)
        });

        let commit_id = commit_id
            .map_err(|_| ScoopieError::Query(QueryError::FailedToRetrieveData))?
            .next()
            .ok_or(ScoopieError::Query(QueryError::FailedToRetrieveData))?
            .map_err(|_| ScoopieError::Query(QueryError::FailedToRetrieveData))?;

        Ok(commit_id)
    }
}
