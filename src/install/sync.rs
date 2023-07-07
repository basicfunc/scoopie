use std::{ffi::OsStr, fs, path::PathBuf};

use git2::build::RepoBuilder;
use rayon::{current_num_threads, current_thread_index, prelude::*};
use rusqlite::Connection;
use serde_json::Value;
use tempfile::tempdir;

use crate::config::Config;
use crate::error::{BucketError, DatabaseError};
use crate::error::{ScoopieError, SyncError};

pub struct Sync {}

impl Sync {
    pub fn sync() -> Result<(), ScoopieError> {
        let config = Config::read()?;
        let repos = config.repos()?;
        let temp_dir = tempdir().map_err(|_| ScoopieError::Sync(SyncError::UnableToMkTmpDir))?;
        let temp_dir = temp_dir.path();

        let databases: Result<Vec<Database>, ScoopieError> = repos
            .par_iter()
            .map(|(name, url)| {
                let repo = Repo::fetch(name, url, &temp_dir.join(name))?;
                let id = format!("{}-{}.db", repo.name, repo.commit_id);
                let repo_dir = Config::repos_dir()?;
                let repo_path = repo_dir.join(id);

                if !repo_path.exists() {
                    let bucket = Bucket::fetch_from(repo)?;
                    Database::create_from(bucket)
                } else {
                    Ok(Database {
                        name: repo.name.into(),
                        path: repo_dir,
                        state: DatabaseState::AlreadyExists,
                    })
                }
            })
            .collect();

        let databases = databases?;
        println!("{:?}", databases);

        Ok(())
    }
}

#[derive(Debug, Default)]
struct Repo {
    name: String,
    commit_id: String,
    path: PathBuf,
}

impl Repo {
    fn fetch(name: &String, url: &String, path: &PathBuf) -> Result<Repo, ScoopieError> {
        let repo = RepoBuilder::new()
            .clone(url, &path)
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
struct Entry(String, String);

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

                bucket.mainfests.push(Entry(app_name, mainfest));
            }
        }

        Ok(bucket)
    }
}

#[derive(Debug, PartialEq, PartialOrd)]
enum DatabaseState {
    AlreadyExists,
    Created,
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

        let conn = Connection::open(&db)
            .map_err(|_| ScoopieError::Database(DatabaseError::UnableToOpen))?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS mainfests (
                 app_name TEXT NOT NULL PRIMARY KEY,
                 mainfest TEXT
             )",
            [],
        )
        .map_err(|_| ScoopieError::Database(DatabaseError::FailedToCreateTable))?;

        let mut stmt = conn
            .prepare("INSERT INTO mainfests (app_name, mainfest) VALUES (?, ?)")
            .map_err(|_| ScoopieError::Database(DatabaseError::FailedToMkStmt))?;

        for mainfest in bucket.mainfests {
            stmt.execute(&[&mainfest.0, &mainfest.1])
                .map_err(|_| ScoopieError::Database(DatabaseError::FailedInsertion))?;
        }

        Ok(Database {
            name: repo,
            path: db,
            state: DatabaseState::Created,
        })
    }
}

// fn has_older_database(db_dir: &PathBuf, db_prefix: &str) -> bool {
//     fs::read_dir(db_dir)
//         .map(|entries| {
//             entries
//                 .filter_map(|entry| {
//                     entry
//                         .ok()
//                         .and_then(|e| e.file_name().to_str().map(|name| name.to_owned()))
//                 })
//                 .any(|name| name.starts_with(db_prefix) && name.ends_with(".db"))
//         })
//         .unwrap_or(false)
// }
