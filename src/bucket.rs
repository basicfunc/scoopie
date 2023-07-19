use std::{ffi::OsStr, fs, path::PathBuf};

use git2::build::RepoBuilder;
use serde_json::Value;

use crate::error::{BucketError, ScoopieError, SyncError};

pub type BucketResult = Result<Bucket, ScoopieError>;

#[derive(Debug, Default)]
pub struct Bucket {
    pub id: String,
    pub manifests: Vec<(String, String)>,
}

impl Bucket {
    pub fn fetch<N, U, P>(name: N, url: U, path: P) -> BucketResult
    where
        N: Into<String>,
        U: Into<String>,
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

        let id = format!("{}-{}", name.into(), commit_id);
        let mut bucket = Bucket {
            id,
            manifests: vec![],
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

                    let mainfest = json.to_string();

                    bucket.manifests.push((app_name.to_owned(), mainfest));
                }
            }
        }

        Ok(bucket)
    }
}
