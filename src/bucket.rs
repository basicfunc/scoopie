use std::{ffi::OsStr, fs, path::PathBuf};

use git2::build::RepoBuilder;
use serde_json::Value;

use crate::error::{BucketError, ScoopieError, SyncError};

#[derive(Debug, Default, Clone)]
pub struct MainfestEntry {
    pub app_name: String,
    pub mainfest: String,
}

#[derive(Debug, Default)]
pub struct Bucket {
    pub id: String,
    pub mainfests: Vec<MainfestEntry>,
}

impl Bucket {
    pub fn fetch(name: &str, url: &String, path: &PathBuf) -> Result<Bucket, ScoopieError> {
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

        let id = format!("{}-{}", name, commit_id);
        let mut bucket = Bucket {
            id,
            mainfests: vec![],
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
                let app_name = file_path.file_stem().unwrap_or(file_path.as_os_str());
                let app_name = app_name.to_string_lossy().to_string();

                let file_content = fs::read_to_string(&file_path)
                    .map_err(|_| ScoopieError::Bucket(BucketError::MainfestRead))?;

                let json: Value = serde_json::from_str(&file_content)
                    .map_err(|_| ScoopieError::Bucket(BucketError::InvalidJSON))?;

                let mainfest = json.to_string();

                bucket.mainfests.push(MainfestEntry { app_name, mainfest });
            }
        }

        Ok(bucket)
    }
}
