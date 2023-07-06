use std::{fs, path::PathBuf};

use crate::config::Config;
use crate::error::{BucketError, ScoopieError};
use crate::install::sync::Repo;
use rusqlite::Connection;
use serde_json::Value;

#[derive(Debug, Default)]
pub struct Entry {
    app_name: String,
    mainfest: String,
}

impl Entry {
    pub fn new(app_name: String, mainfest: String) -> Self {
        Self { app_name, mainfest }
    }
}

#[derive(Debug, Default)]
pub struct Bucket {
    name: String,
    mainfests: Vec<Entry>,
}

impl Bucket {
    pub fn fetch_from(repo: Repo) -> Result<Bucket, ScoopieError> {
        let db_name = format!("{}-{}", repo.name, repo.commit_id);
        let mut bucket = Bucket {
            name: db_name,
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
            let ext = file_path.extension().unwrap_or_default();

            if ext == "json" && file_path.is_file() {
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
enum DBStatus {
    Created,
    AlreadyExists,
}

#[derive(Debug)]
pub struct DB {
    name: String,
    path: PathBuf,
    status: DBStatus,
}

impl DB {
    pub fn create_from(bucket: Bucket) -> Result<DB, ScoopieError> {
        let repo = format!("{}.db", bucket.name);
        let db = Config::repos_dir().unwrap().join(&repo);

        if db.exists() {
            return Ok(DB {
                name: repo,
                path: db,
                status: DBStatus::AlreadyExists,
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

        // conn.close().map_err(|_| DBError::FailedToCommit)?;

        Ok(DB {
            name: repo,
            path: db,
            status: DBStatus::Created,
        })
    }
}
