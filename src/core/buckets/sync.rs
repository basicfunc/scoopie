use std::{
    collections::HashMap,
    ffi::OsStr,
    fmt::{self, Display, Formatter},
    fs::{self, OpenOptions},
    io::Write,
    path::PathBuf,
    time::Duration,
};

use crate::core::config::*;
use crate::error::*;
use crate::utils::*;

use super::manifest::Manifest;
use super::metadata::MetaData;
use super::{Bucket, Buckets};

use console::style;
use git2::{build::RepoBuilder, FetchOptions};
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use rayon::prelude::*;
use serde_json::json;

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
        let mb = MultiProgress::new();

        Config::read()?
            .known_buckets()
            .par_iter()
            .map(|v| Bucket::sync(&mb, v.0, v.1))
            .collect()
    }
}

trait Sync: ReadFromRepo {
    type Error;
    fn sync(pb: &MultiProgress, name: &str, url: &str)
        -> Result<SyncStatus, <Self as Sync>::Error>;
}

impl Sync for Bucket {
    type Error = ScoopieError;

    fn sync(
        mb: &MultiProgress,
        name: &str,
        url: &str,
    ) -> Result<SyncStatus, <Self as Sync>::Error> {
        let temp_dir_builder = TempDir::build()?;
        let temp_dir = temp_dir_builder.path();

        let pb = mb.add(ProgressBar::new_spinner());
        pb.enable_steady_tick(Duration::from_millis(3));

        pb.set_message(
            style(format!("Fetching bucket {name} from remote..."))
                .bold()
                .to_string(),
        );

        let mut fo = FetchOptions::new();
        fo.depth(1);

        let repo = RepoBuilder::new()
            .bare(true)
            .fetch_options(fo)
            .clone(url, &temp_dir)
            .map_err(|_| ScoopieError::SyncUnableToFetchRepo)?;

        let head = repo.head().map_err(|_| ScoopieError::SyncUnableToGetHead)?;

        let commit_id = head
            .peel_to_commit()
            .map_err(|_| ScoopieError::SyncUnableToGetCommit)?
            .id()
            .to_string();

        let bucket_dir = Config::buckets_dir()?;
        let bucket_path = bucket_dir.join(name);

        pb.set_message(
            style(format!("Reading metadata for bucket {name}..."))
                .bold()
                .to_string(),
        );

        let mut metadata = MetaData::read()?;

        let st = match (
            bucket_path.exists(),
            metadata.get(name).commit_id == commit_id,
        ) {
            (true, true) => {
                pb.finish_with_message(
                    style(format!("Bucket: {name} is already synced to the remote."))
                        .bold()
                        .to_string(),
                );
                SyncStatus::UpToDate(name.into())
            }
            (true, false) => {
                pb.set_message(
                    style(format!("Fetching manifests from bucket {name}..."))
                        .bold()
                        .to_string(),
                );
                Self::read(&temp_dir)?.write_to(&bucket_path);
                pb.set_message(
                    style(format!("Writing metadata for bucket {name}..."))
                        .bold()
                        .to_string(),
                );
                metadata.write(name, url, &commit_id)?;
                pb.finish_with_message(
                    style(format!("Bucket: {name} is now synced to the remote."))
                        .bold()
                        .to_string(),
                );
                SyncStatus::Synced(name.into())
            }

            (false, _) => {
                pb.set_message(
                    style(format!("Fetching manifests from bucket {name}..."))
                        .bold()
                        .to_string(),
                );
                Self::read(&temp_dir)?.write_to(&bucket_path);
                pb.set_message(
                    style(format!("Writing metadata for bucket {name}..."))
                        .bold()
                        .to_string(),
                );
                metadata.write(name, url, &commit_id)?;
                pb.finish_with_message(
                    style(format!(
                        "Bucket: {name} is created and synced to the remote."
                    ))
                    .bold()
                    .to_string(),
                );
                SyncStatus::Created(name.into())
            }
        };

        Ok(st)
    }
}

trait ReadFromRepo: Sized {
    type Error;
    fn read(path: &PathBuf) -> Result<Self, Self::Error>;
}

impl ReadFromRepo for Bucket {
    type Error = ScoopieError;

    fn read(path: &PathBuf) -> Result<Self, Self::Error> {
        let bucket_path = path.join("bucket");

        match (bucket_path.is_dir(), bucket_path.exists()) {
            (true, true) => {
                let manifests = fs::read_dir(bucket_path)
                    .map_err(|_| ScoopieError::BucketsNotFound)?
                    .filter_map(Result::ok)
                    .filter(|entry| entry.path().extension() == Some(OsStr::new("json")))
                    .par_bridge()
                    .map(|entry| -> Result<(String, Manifest), ScoopieError> {
                        let file_path = entry.path();
                        let app_name = file_path
                            .file_stem()
                            .unwrap_or_default()
                            .to_string_lossy()
                            .to_string();

                        let manifest = Manifest::try_from(file_path)?;
                        Ok((app_name, manifest))
                    })
                    .collect::<Result<HashMap<_, _>, _>>()?;
                Ok(Bucket(manifests))
            }
            _ => Err(ScoopieError::BucketsNotFound),
        }
    }
}
