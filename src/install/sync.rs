use std::ffi::OsStr;
use std::{fs, path::PathBuf};

use git2::Repository;
use rayon::prelude::*;
use rusqlite::Connection;
use serde_json::Value;
use tempfile::tempdir;

use crate::config::Config;
use crate::error::BucketError;
use crate::error::{ScoopieError, SyncError};

pub struct Sync {}

impl Sync {
    pub fn sync() -> Result<(), ScoopieError> {
        let config = Config::read()?;
        let repos = config.repos()?;
        let temp_dir = tempdir().map_err(|_| ScoopieError::Sync(SyncError::UnableToMkTmpDir))?;
        let temp_dir = temp_dir.path();

        repos
            .par_iter()
            .try_for_each(|(name, url)| -> Result<(), ScoopieError> {
                let repo = Repo::clone(name, url, &temp_dir.join(name))?;
                let bucket = Bucket::fetch_from(repo)?;
                let database = Database::create_from(bucket)?;

                println!("{:?}", database);
                Ok(())
            })?;

        Ok(())
    }
}

#[derive(Debug, Default)]
struct Repo {
    pub name: String,
    pub commit_id: String,
    pub path: PathBuf,
}

impl Repo {
    fn clone(name: &String, url: &String, path: &PathBuf) -> Result<Repo, ScoopieError> {
        let repo = Repository::clone(url, &path)
            .map_err(|_| ScoopieError::Sync(SyncError::UnableToFetchRepo))?;
        let head = repo
            .head()
            .map_err(|_| ScoopieError::Sync(SyncError::UnableToGetHead))?;
        let commit_id = head
            .peel_to_commit()
            .map_err(|_| ScoopieError::Sync(SyncError::UnableToGetCommit))?
            .id();

        Ok(Repo {
            name: name.into(),
            commit_id: commit_id.to_string(),
            path: path.to_path_buf(),
        })
    }
}

#[derive(Debug, Default)]
struct Entry {
    app_name: String,
    mainfest: String,
}

impl Entry {
    pub fn new(app_name: String, mainfest: String) -> Self {
        Self { app_name, mainfest }
    }
}

#[derive(Debug, Default)]
struct Bucket {
    name: String,
    id: String,
    mainfests: Vec<Entry>,
}

impl Bucket {
    fn fetch_from(repo: Repo) -> Result<Bucket, ScoopieError> {
        let id = format!("{}-{}", repo.name, repo.commit_id);
        let mut bucket = Bucket {
            name: repo.name,
            id,
            mainfests: vec![],
        };

        let bucket_path = &repo.path.join("bucket");
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
                let app_name = file_path.file_stem().unwrap_or(file_path.as_os_str());
                let app_name = app_name.to_string_lossy().to_string();

                let file_content = fs::read_to_string(&file_path)
                    .map_err(|_| ScoopieError::Bucket(BucketError::MainfestRead))?;

                let json: Value = serde_json::from_str(&file_content)
                    .map_err(|_| ScoopieError::Bucket(BucketError::InvalidJSON))?;

                let mainfest = json.to_string();

                bucket.mainfests.push(Entry::new(app_name, mainfest));
            }
        }

        Ok(bucket)
    }
}

#[derive(Debug)]
enum DatabaseState {
    AlreadyExists,
    Created,
    Updated,
}

#[derive(Debug)]
struct Database {
    name: PathBuf,
    path: PathBuf,
    state: DatabaseState,
}

impl Database {
    fn create_from(bucket: Bucket) -> Result<Database, ScoopieError> {
        let mut repo = PathBuf::from(&bucket.id);
        repo.set_extension("db");

        let repo_dir = Config::repos_dir()?;
        let db = repo_dir.join(&repo);

        if db.exists() {
            return Ok(Database {
                name: repo,
                path: db,
                state: DatabaseState::AlreadyExists,
            });
        }

        let conn = Connection::open(&db)
            .map_err(|_| ScoopieError::Database(crate::error::DatabaseError::UnableToOpen))?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS mainfests (
                 app_name TEXT PRIMARY KEY,
                 mainfest TEXT
             )",
            [],
        )
        .map_err(|_| ScoopieError::Database(crate::error::DatabaseError::FailedToCreateTable))?;

        let mut stmt = conn
            .prepare("INSERT INTO mainfests (app_name, mainfest) VALUES (?, ?)")
            .map_err(|_| ScoopieError::Database(crate::error::DatabaseError::FailedToMkStmt))?;

        for mainfest in bucket.mainfests {
            stmt.execute(&[&mainfest.app_name, &mainfest.mainfest])
                .map_err(|_| {
                    ScoopieError::Database(crate::error::DatabaseError::FailedInsertion)
                })?;
        }

        let state = if has_database(&repo_dir, &bucket.name) {
            DatabaseState::Updated
        } else {
            DatabaseState::Created
        };

        Ok(Database {
            name: repo,
            path: db,
            state,
        })
    }
}

fn has_database(db_dir: &PathBuf, db_prefix: &str) -> bool {
    fs::read_dir(db_dir)
        .map(|entries| {
            entries
                .filter_map(|entry| {
                    entry
                        .ok()
                        .and_then(|e| e.file_name().to_str().map(|name| name.to_owned()))
                })
                .any(|name| name.starts_with(db_prefix) && name.ends_with(".db"))
        })
        .unwrap_or(false)
}
