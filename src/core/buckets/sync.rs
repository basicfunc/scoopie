use std::{
    collections::HashMap,
    fmt::{self, Display, Formatter},
    fs::{self, OpenOptions},
    io::Write,
    path::PathBuf,
};

use crate::core::config::*;
use crate::error::*;

use super::manifest::Manifest;
use super::metadata::MetaData;
use super::{Bucket, Buckets};

use git2::build::RepoBuilder;
use rayon::prelude::*;
use serde_json::json;
use tempfile::tempdir;

#[derive(Debug, PartialEq, PartialOrd)]
pub enum SyncStatus {
    UpToDate(String),
    Synced(String),
    Created(String),
}

impl Display for SyncStatus {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            SyncStatus::UpToDate(bucket_name) => write!(f, "{bucket_name} is already up-to-date"),
            SyncStatus::Created(bucket_name) => write!(f, "Created new {bucket_name}"),
            SyncStatus::Synced(bucket_name) => write!(f, "Synced {bucket_name} to the remote"),
        }
    }
}

impl Bucket {
    fn count(&self) -> usize {
        self.0.iter().count()
    }

    fn write_to(&self, path: &PathBuf) {
        let json = json!(self.0).to_string();

        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(path)
            .unwrap();

        file.write_all(json.as_bytes()).unwrap();
    }
}

pub trait SyncAll {
    type Error;
    fn sync() -> Result<Vec<SyncStatus>, Self::Error>;
}

impl SyncAll for Buckets {
    type Error = ScoopieError;
    fn sync() -> Result<Vec<SyncStatus>, Self::Error> {
        Config::read()?
            .known_buckets()
            .par_iter()
            .map(|v| Bucket::sync(v.0, v.1))
            .collect()
    }
}

trait Sync: ReadFromRepo {
    type Error;
    fn sync(name: &str, url: &str) -> Result<SyncStatus, <Self as Sync>::Error>;
}

impl Sync for Bucket {
    type Error = ScoopieError;

    fn sync(name: &str, url: &str) -> Result<SyncStatus, <Self as Sync>::Error> {
        let temp_dir = tempdir()
            .map_err(|_| ScoopieError::Sync(SyncError::UnableToMkTmpDir))?
            .into_path();

        let repo = RepoBuilder::new()
            .clone(url, &temp_dir)
            .map_err(|_| ScoopieError::Sync(SyncError::UnableToFetchRepo))?;

        let head = repo
            .head()
            .map_err(|_| ScoopieError::Sync(SyncError::UnableToGetHead))?;

        let commit_id = head
            .peel_to_commit()
            .map_err(|_| ScoopieError::Sync(SyncError::UnableToGetCommit))?
            .id()
            .to_string();

        let bucket_dir = Config::buckets_dir()?;
        let bucket_path = bucket_dir.join(name);

        let mut metadata = MetaData::read()?;

        let st = match (
            bucket_path.exists(),
            metadata.get(name).commit_id == commit_id,
        ) {
            (true, true) => SyncStatus::UpToDate(name.into()),
            (true, false) => {
                Self::read(&temp_dir)?.write_to(&bucket_path);
                metadata.write(name, url, &commit_id)?;
                SyncStatus::Synced(name.into())
            }
            (false, _) => {
                Self::read(&temp_dir)?.write_to(&bucket_path);
                metadata.write(name, url, &commit_id)?;
                SyncStatus::Created(name.into())
            }
        };

        Ok(st)
    }
}

trait ReadFromRepo {
    type Error;
    fn read(path: &PathBuf) -> Result<Self, Self::Error>
    where
        Self: Sized;
}

impl ReadFromRepo for Bucket {
    type Error = ScoopieError;

    fn read(path: &PathBuf) -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
        let bucket_path = path.join("bucket");

        match (bucket_path.is_dir(), bucket_path.exists()) {
            (true, true) => {
                let manifests: HashMap<String, Manifest> = fs::read_dir(bucket_path)
                    .map_err(|_| ScoopieError::Bucket(BucketError::BucketsNotFound))?
                    .par_bridge()
                    .filter_map(|entry| {
                        let file_path = entry.ok()?.path();

                        match file_path.extension().and_then(|e| e.to_str()) {
                            Some("json") => {
                                let app_name = file_path
                                    .file_stem()
                                    .unwrap_or_default()
                                    .to_string_lossy()
                                    .to_string();

                                let buff = fs::read_to_string(file_path).ok()?;
                                let manifest: Manifest = serde_json::from_str(&buff).ok()?;

                                Some((app_name, manifest))
                            }
                            _ => None,
                        }
                    })
                    .collect();
                Ok(Bucket(manifests))
            }
            _ => Err(ScoopieError::Bucket(BucketError::NotFound)),
        }
    }
}
