pub mod manifest;

use git2::build::RepoBuilder;
use lazy_static::lazy_static;
use rayon::prelude::*;
use rusqlite::{params, Connection, Row};

use std::{
    ffi::OsStr,
    fmt::{self, Display},
    fs,
    path::PathBuf,
    write,
};

use crate::{config::*, error::*};

use manifest::Manifest;

lazy_static! {
    static ref TABLE_CREATE_STMT: &'static str = "CREATE TABLE IF NOT EXISTS manifests (
                 app_name TEXT NOT NULL PRIMARY KEY,
                 manifest TEXT
             )";
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

#[derive(Debug, Clone)]
pub struct Entry {
    pub app_name: String,
    pub manifest: Manifest,
}

impl Entry {
    fn new(app_name: &str, manifest: Manifest) -> Self {
        Self {
            app_name: app_name.into(),
            manifest,
        }
    }
}

impl TryFrom<&Row<'_>> for Entry {
    type Error = rusqlite::Error;

    fn try_from(value: &Row) -> Result<Self, Self::Error> {
        let app_name = value.get(0)?;
        let manifest: String = value.get(1)?;

        let manifest: Manifest = serde_json::from_str(&manifest).unwrap();

        Ok(Entry { app_name, manifest })
    }
}

#[derive(Debug, Default)]
pub struct RawData {
    pub id: String,
    pub bucket_name: String,
    pub entries: Vec<Entry>,
}

impl RawData {
    fn new(id: String) -> Self {
        let bucket_name = id.split("-").next().unwrap_or_default().into();

        Self {
            id,
            bucket_name,
            entries: vec![],
        }
    }

    fn insert(&mut self, data: Entry) {
        self.entries.push(data)
    }
}

impl From<PathBuf> for RawData {
    fn from(value: PathBuf) -> Self {
        let id = value
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        let bucket_name = id.split("-").next().unwrap_or_default().to_string();

        RawData {
            id,
            bucket_name,
            entries: vec![],
        }
    }
}

impl Display for RawData {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> fmt::Result {
        let bucket_name = &self.bucket_name;

        for entry in self.entries.iter() {
            let app_name = &entry.app_name;
            let version = &entry.manifest.version;
            let description = &entry.manifest.description;

            writeln!(
                f,
                "\n{}/{} {}\n  {}",
                app_name, bucket_name, version, description
            )?;
        }

        Ok(())
    }
}

pub trait Raw<U, P> {
    type Error;
    fn raw(name: U, url: U, path: P) -> Result<RawData, Self::Error>;
}

impl Raw<&String, &PathBuf> for Bucket {
    type Error = ScoopieError;

    fn raw(name: &String, url: &String, path: &PathBuf) -> Result<RawData, Self::Error> {
        let repo = RepoBuilder::new()
            .clone(&url, &path)
            .map_err(|_| ScoopieError::Sync(SyncError::UnableToFetchRepo))?;

        let head = repo
            .head()
            .map_err(|_| ScoopieError::Sync(SyncError::UnableToGetHead))?;

        let commit_id = head
            .peel_to_commit()
            .map_err(|_| ScoopieError::Sync(SyncError::UnableToGetCommit))?
            .id();

        let id = format!("{}-{}", name, commit_id);
        let mut data = RawData::new(id);

        let bucket_path = path.join("bucket");
        let entries = fs::read_dir(bucket_path)
            .map_err(|_| ScoopieError::Bucket(BucketError::BucketsNotFound))?;

        for entry in entries {
            let entry = entry.map_err(|e| match e.kind() {
                std::io::ErrorKind::NotFound => ScoopieError::Bucket(BucketError::NotFound),
                std::io::ErrorKind::PermissionDenied => ScoopieError::PermissionDenied,
                _ => ScoopieError::Unknown,
            })?;

            let file_path = entry.path();
            let ext = file_path.extension();

            if ext == Some(OsStr::new("json")) && file_path.is_file() {
                if let Some(app_name) = file_path.file_stem().and_then(|stem| stem.to_str()) {
                    let file_content = fs::read_to_string(&file_path)
                        .map_err(|_| ScoopieError::Bucket(BucketError::MainfestRead))?;

                    let manifest: Manifest = serde_json::from_str(&file_content)
                        .map_err(|_| ScoopieError::Bucket(BucketError::InvalidManifest))?;

                    data.insert(Entry::new(app_name, manifest));
                }
            }
        }

        Ok(data)
    }
}

impl TryFrom<RawData> for Bucket {
    type Error = ScoopieError;

    fn try_from(raw_data: RawData) -> Result<Bucket, Self::Error> {
        let mut repo = PathBuf::from(&raw_data.id);
        repo.set_extension("db");

        let db = Config::buckets_dir()?.join(&repo);

        if db.exists() {
            return Ok(Bucket {
                name: raw_data.bucket_name,
                path: db,
                status: BucketStatus::UpToDate,
            });
        }

        let conn = Connection::open(&db)
            .map_err(|_| ScoopieError::Database(DatabaseError::UnableToOpen))?;

        conn.execute(*TABLE_CREATE_STMT, [])
            .map_err(|_| ScoopieError::Database(DatabaseError::FailedToCreateTable))?;

        let mut stmt = conn
            .prepare(*INSERT_STMT)
            .map_err(|_| ScoopieError::Database(DatabaseError::FailedToMkStmt))?;

        for entry in &raw_data.entries {
            let manifest = entry.manifest.clone().try_into()?;
            stmt.execute(&[&entry.app_name, &manifest])
                .map_err(|_| ScoopieError::Database(DatabaseError::FailedInsertion))?;
        }

        conn.execute(*FTS_TABLE_CREATE_STMT, [])
            .map_err(|_| ScoopieError::Database(DatabaseError::FailedToCreateTable))?;

        conn.execute(*FTS_INSERT_STMT, [])
            .map_err(|_| ScoopieError::Database(DatabaseError::FailedInsertion))?;

        Ok(Bucket {
            name: raw_data.bucket_name,
            path: db,
            status: BucketStatus::Created,
        })
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

        Ok(Query {
            buckets,
            stmt: query_stmt.into(),
        })
    }
}

pub trait ExecuteQuery<T> {
    type Error;
    fn execute(&self, params: T) -> Result<Vec<RawData>, Self::Error>;
}

impl ExecuteQuery<String> for Query {
    type Error = ScoopieError;

    fn execute(&self, params: String) -> Result<Vec<RawData>, Self::Error> {
        let query_bucket = |bucket_path: &PathBuf| -> Result<RawData, ScoopieError> {
            let conn = Connection::open(bucket_path)
                .map_err(|_| ScoopieError::Database(DatabaseError::UnableToOpen))?;

            let mut stmt = conn
                .prepare(&self.stmt)
                .map_err(|_| ScoopieError::Database(DatabaseError::FailedToMkStmt))?;

            let mut data = RawData::from(bucket_path.to_path_buf());

            data.entries.extend(
                stmt.query_map(params![params], |row| Entry::try_from(row))
                    .map_err(|_| ScoopieError::Query(QueryError::FailedToRetrieveData))?
                    .flat_map(Result::ok),
            );

            Ok(data)
        };

        self.buckets.par_iter().map(query_bucket).collect()
    }
}
