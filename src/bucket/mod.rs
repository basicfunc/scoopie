pub mod data;
pub mod manifest;

use chrono::{DateTime, Local};
use git2::build::RepoBuilder;
use lazy_static::lazy_static;
use rayon::prelude::*;
use rusqlite::{params, Connection};

use std::{fmt, fs, path::PathBuf, write};

use crate::{config::*, error::*};
use data::*;

use manifest::Manifest;

lazy_static! {
    static ref METADATA_TABLE_CREATE_STMT: &'static str = "CREATE TABLE IF NOT EXISTS metadata (
                bucket_name VARCHAR(255),
                commit_id VARCHAR(100),
                url VARCHAR(1000),
                date DATE,
                time TIME,
                number_of_manifests INT)";
    static ref METADATA_INSERT_STMT: &'static str =
        "INSERT INTO metadata (bucket_Name, commit_id, url, date, time, number_of_manifests)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)";
    static ref TABLE_CREATE_STMT: &'static str = "CREATE TABLE IF NOT EXISTS manifests (
                app_name TEXT NOT NULL PRIMARY KEY,
                manifest TEXT)";
    static ref FTS_TABLE_CREATE_STMT: &'static str =
        "CREATE VIRTUAL TABLE IF NOT EXISTS manifests_fts USING FTS5(app_name, manifest)";
    static ref INSERT_STMT: &'static str =
        "INSERT INTO manifests (app_name, manifest) VALUES (?, ?)";
    static ref FTS_INSERT_STMT: &'static str =
        "INSERT INTO manifests_fts(app_name, manifest) SELECT app_name, manifest FROM manifests";
}

#[derive(Debug, PartialEq, PartialOrd)]
pub enum BucketStatus {
    UpToDate,
    Created,
}

#[derive(Debug)]
pub struct Bucket {
    name: String,
    path: PathBuf,
    status: BucketStatus,
}

impl Bucket {
    pub fn create(
        bucket_name: &String,
        url: &String,
        path: &PathBuf,
    ) -> Result<Self, ScoopieError> {
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

        let bucket_dir = fs::read_dir(&bucket_path)
            .map_err(|_| ScoopieError::Bucket(BucketError::BucketsNotFound))?;

        let entries: Result<Vec<_>, _> = bucket_dir
            .into_iter()
            .map(|e| -> Result<Option<Entry>, ScoopieError> {
                let entry = e.map_err(|e| match e.kind() {
                    std::io::ErrorKind::NotFound => ScoopieError::Bucket(BucketError::NotFound),
                    std::io::ErrorKind::PermissionDenied => ScoopieError::PermissionDenied,
                    _ => ScoopieError::Unknown,
                })?;

                let file_path = entry.path();

                if let Some(extension) = file_path.extension() {
                    if extension == "json" {
                        let app_name = file_path
                            .file_stem()
                            .unwrap_or_default()
                            .to_string_lossy()
                            .to_string();

                        let file_content = fs::read_to_string(&file_path)
                            .map_err(|_| ScoopieError::Bucket(BucketError::MainfestRead))?;

                        let manifest: Manifest = serde_json::from_str(&file_content)
                            .map_err(|_| ScoopieError::Bucket(BucketError::InvalidManifest))?;

                        Ok(Some(Entry { app_name, manifest }))
                    } else {
                        Ok(None)
                    }
                } else {
                    Ok(None)
                }
            })
            .collect();

        let entries: Vec<Option<Entry>> = entries?;
        let entries: Vec<Entry> = entries
            .into_par_iter()
            .filter_map(|entry| entry) // Filter out the `None` entries and extract the `Some` values
            .collect();

        let curr: DateTime<Local> = Local::now();

        // Format the date and time as strings
        let date = curr.format("%Y-%m-%d").to_string();
        let time = curr.format("%H:%M:%S").to_string();

        let mut bucket_path = PathBuf::from(&bucket_name);
        bucket_path.set_extension("db");

        let bucket_path = Config::buckets_dir()?.join(&bucket_path);

        if bucket_path.exists() {
            return Ok(Bucket {
                name: bucket_name.into(),
                path: bucket_path,
                status: BucketStatus::UpToDate,
            });
        }

        let conn = Connection::open(&bucket_path)
            .map_err(|_| ScoopieError::Database(DatabaseError::UnableToOpen))?;

        conn.execute(*METADATA_TABLE_CREATE_STMT, [])
            .map_err(|_| ScoopieError::Database(DatabaseError::FailedToCreateTable))?;

        conn.execute(
            *METADATA_INSERT_STMT,
            params![&bucket_name, &commit_id, &url, &date, &time, &entries.len()],
        )
        .map_err(|_| ScoopieError::Database(DatabaseError::FailedInsertion))?;

        conn.execute(*TABLE_CREATE_STMT, [])
            .map_err(|_| ScoopieError::Database(DatabaseError::FailedToCreateTable))?;

        let mut stmt = conn
            .prepare(*INSERT_STMT)
            .map_err(|_| ScoopieError::Database(DatabaseError::FailedToMkStmt))?;

        for entry in &entries {
            let manifest = entry.manifest.clone().try_into()?;
            stmt.execute(&[&entry.app_name, &manifest])
                .map_err(|_| ScoopieError::Database(DatabaseError::FailedInsertion))?;
        }

        conn.execute(*FTS_TABLE_CREATE_STMT, [])
            .map_err(|_| ScoopieError::Database(DatabaseError::FailedToCreateTable))?;

        conn.execute(*FTS_INSERT_STMT, [])
            .map_err(|_| ScoopieError::Database(DatabaseError::FailedInsertion))?;

        Ok(Bucket {
            name: bucket_name.into(),
            path: bucket_path,
            status: BucketStatus::Created,
        })
    }
}

impl std::fmt::Display for Bucket {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let status = match self.status {
            BucketStatus::UpToDate => "up-to-date",
            BucketStatus::Created => "created",
        };

        write!(
            f,
            "Bucket: {} is {} at {}",
            self.name,
            status,
            self.path.display()
        )
    }
}

pub struct Query {
    buckets: Vec<PathBuf>,
    stmt: String,
}

pub trait QueryBuilder<T> {
    type Error;
    fn build_query(query_stmt: T) -> Result<Query, Self::Error>;
}

impl QueryBuilder<&'static str> for Bucket {
    type Error = ScoopieError;

    fn build_query(query_stmt: &'static str) -> Result<Query, Self::Error> {
        let buckets = Config::read()?.latest_buckets()?;

        match !buckets.is_empty() {
            true => Ok(Query {
                buckets,
                stmt: query_stmt.into(),
            }),
            false => Err(ScoopieError::Bucket(BucketError::NotFound)),
        }
    }
}

pub trait ExecuteQuery<T> {
    type Error;
    fn execute(&self, params: T) -> Result<BucketData, Self::Error>;
}

impl ExecuteQuery<String> for Query {
    type Error = ScoopieError;

    fn execute(&self, params: String) -> Result<BucketData, Self::Error> {
        let query_bucket = |bucket_path: &PathBuf| -> Result<(String, Vec<Entry>), ScoopieError> {
            let conn = Connection::open(bucket_path)
                .map_err(|_| ScoopieError::Database(DatabaseError::UnableToOpen))?;

            let mut stmt = conn
                .prepare(&self.stmt)
                .map_err(|_| ScoopieError::Database(DatabaseError::FailedToMkStmt))?;

            let mut entries: Vec<Entry> = vec![];

            entries.extend(
                stmt.query_map(params![params], |row| Entry::try_from(row))
                    .map_err(|_| ScoopieError::Query(QueryError::FailedToRetrieveData))?
                    .flat_map(Result::ok),
            );

            let bucket_name = bucket_path
                .file_stem()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();

            Ok((bucket_name, entries))
        };

        Ok(BucketData(
            self.buckets
                .par_iter()
                .map(query_bucket)
                .collect::<Result<_, _>>()?,
        ))
    }
}
