use std::{
    collections::HashMap,
    eprintln,
    ffi::OsStr,
    fmt::{self, Display},
    fs,
    path::PathBuf,
    write,
};

use crate::{config::*, error::DatabaseError};
use git2::build::RepoBuilder;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::error::{BucketError, ScoopieError, SyncError};

const TABLE_CREATE_STMT: &'static str = "CREATE TABLE IF NOT EXISTS manifests (
                 app_name TEXT NOT NULL PRIMARY KEY,
                 manifest TEXT
             )";

const FTS_TABLE_CREATE_STMT: &'static str =
    "CREATE VIRTUAL TABLE IF NOT EXISTS manifests_fts USING FTS5(app_name, manifest)";

const INSERT_STMT: &'static str = "INSERT INTO manifests (app_name, manifest) VALUES (?, ?)";

const FTS_INSERT_STMT: &'static str =
    "INSERT INTO manifests_fts(app_name, manifest) SELECT app_name, manifest FROM manifests";

#[derive(Serialize, Deserialize, Debug)]
/// This strictly follows Scoop's convention for app manifests, which could be found here: https://github.com/ScoopInstaller/Scoop/wiki/App-Manifests
struct Manifest {
    // Required Properties
    version: String,
    description: String,
    homepage: String,
    license: Value,
    // Optional Properties
    bin: Option<Value>,
    #[serde(rename = "##")]
    extract_dir: Option<String>,
    comments: Option<String>,
    architecture: Option<Value>, // TODO: to implement as individual struct so that it contains related properties.
    autoupdate: Option<Value>, // It is used by scoop to check for autoupdates which is currrently out-of-scope for Scoopie.
    checkver: Option<Value>, // It is used by scoop to check for updated versions which is currrently out-of-scope for Scoopie.
    depends: Option<Vec<String>>,
    suggest: Option<Value>,
    env_add_path: Option<Vec<String>>,
    env_set: Option<HashMap<String, String>>,
    extract_to: Option<Value>,
    hash: Option<Value>,
    innosetup: Option<bool>,
    installer: Option<Value>, // TODO: implement it as individual struct so that it contains related properties.
    notes: Option<Vec<String>>,
    persist: Option<Vec<String>>,
    post_install: Option<Value>,
    post_uninstall: Option<Value>,
    pre_install: Option<Value>,
    pre_uninstall: Option<Value>,
    psmodule: Option<HashMap<String, String>>,
    shortcuts: Option<Vec<Vec<String>>>,
    uninstaller: Option<Value>, // TODO: Same options as installer, but the file/script is run to uninstall the application.
    url: Option<Value>,
    // Undocumented Properties
    cookie: Option<Value>,
    // Deprecated Properties
    _comment: Option<Vec<String>>,
    msi: Option<String>,
}

#[derive(Debug)]
struct Entry {
    app_name: String,
    manifest: String,
}

#[derive(Debug, Default)]
pub struct RawBucket {
    id: String,
    entries: Vec<Entry>,
}

impl RawBucket {
    pub fn new<U, P>(name: U, url: U, path: P) -> Result<RawBucket, ScoopieError>
    where
        U: Into<String> + Display,
        P: Into<PathBuf>,
    {
        let path = path.into();

        let repo = RepoBuilder::new()
            .clone(&url.into(), &path)
            .map_err(|_| ScoopieError::Sync(SyncError::UnableToFetchRepo))?;

        let head = repo
            .head()
            .map_err(|_| ScoopieError::Sync(SyncError::UnableToGetHead))?;

        let commit_id = head
            .peel_to_commit()
            .map_err(|_| ScoopieError::Sync(SyncError::UnableToGetCommit))?
            .id();

        let id = format!("{}-{}", name, commit_id);
        let mut bucket = RawBucket {
            id,
            entries: vec![],
        };

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

                    let json: Value = serde_json::from_str(&file_content)
                        .map_err(|_| ScoopieError::Bucket(BucketError::InvalidJSON))?;

                    match serde_json::from_str::<Manifest>(&file_content) {
                        Ok(_) => (),
                        Err(e) => eprintln!("Error: {e} on file: {file_path:?}"),
                    }

                    let manifest = json.to_string();

                    bucket.entries.push(Entry {
                        app_name: app_name.to_string(),
                        manifest,
                    });
                }
            }
        }

        Ok(bucket)
    }
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

impl TryFrom<RawBucket> for Bucket {
    type Error = ScoopieError;

    fn try_from(raw_bucket: RawBucket) -> Result<Bucket, Self::Error> {
        let name = &raw_bucket.id.split("-").next().unwrap_or_default();
        let name = name.to_string();

        let mut repo = PathBuf::from(&raw_bucket.id);
        repo.set_extension("db");

        let db = Config::buckets_dir()?.join(&repo);

        if db.exists() {
            return Ok(Bucket {
                name,
                path: db,
                status: BucketStatus::UpToDate,
            });
        }

        let conn = Connection::open(&db)
            .map_err(|_| ScoopieError::Database(DatabaseError::UnableToOpen))?;

        conn.execute(TABLE_CREATE_STMT, [])
            .map_err(|_| ScoopieError::Database(DatabaseError::FailedToCreateTable))?;

        let mut stmt = conn
            .prepare(INSERT_STMT)
            .map_err(|_| ScoopieError::Database(DatabaseError::FailedToMkStmt))?;

        for entry in &raw_bucket.entries {
            stmt.execute(&[&entry.app_name, &entry.manifest])
                .map_err(|_| ScoopieError::Database(DatabaseError::FailedInsertion))?;
        }

        conn.execute(FTS_TABLE_CREATE_STMT, [])
            .map_err(|_| ScoopieError::Database(DatabaseError::FailedToCreateTable))?;

        conn.execute(FTS_INSERT_STMT, [])
            .map_err(|_| ScoopieError::Database(DatabaseError::FailedInsertion))?;

        Ok(Bucket {
            name,
            path: db,
            status: BucketStatus::Created,
        })
    }
}
