use crate::error::{BucketError, ScoopieError};
use rayon::prelude::*;
use rusqlite::Row;

use super::manifest::Manifest;

use std::{
    collections::HashMap,
    fmt,
    fs::{self, DirEntry},
    io::{Error, ErrorKind},
    write,
};

#[derive(Debug)]
pub struct Entry {
    pub app_name: String,
    pub manifest: Manifest,
}

pub trait TryMaybeFrom<T> {
    type Error;
    fn try_maybe_from(value: T) -> Result<Option<Self>, Self::Error>
    where
        Self: Sized;
}

impl TryMaybeFrom<Result<DirEntry, Error>> for Entry {
    type Error = ScoopieError;

    fn try_maybe_from(value: Result<DirEntry, Error>) -> Result<Option<Self>, Self::Error> {
        let file_path = value
            .map_err(|e| match e.kind() {
                ErrorKind::NotFound => ScoopieError::Bucket(BucketError::NotFound),
                ErrorKind::PermissionDenied => ScoopieError::PermissionDenied,
                _ => ScoopieError::Unknown,
            })?
            .path();

        Ok(match file_path.extension().unwrap_or_default() == "json" {
            true => Some(Entry {
                app_name: file_path
                    .file_stem()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string(),
                manifest: {
                    let buff = fs::read_to_string(&file_path)
                        .map_err(|_| ScoopieError::Bucket(BucketError::MainfestRead))?;
                    serde_json::from_str(&buff)
                        .map_err(|_| ScoopieError::Bucket(BucketError::InvalidManifest))?
                },
            }),
            false => None,
        })
    }
}

impl TryFrom<&Row<'_>> for Entry {
    type Error = rusqlite::Error;

    fn try_from(value: &Row) -> Result<Self, Self::Error> {
        let app_name = value.get(0)?;

        let manifest: Manifest = {
            let m: String = value.get(1)?;
            serde_json::from_str(&m).unwrap()
        };

        Ok(Entry { app_name, manifest })
    }
}

#[derive(Default, Debug)]
pub struct BucketData(HashMap<String, Vec<Entry>>);

impl BucketData {
    pub fn entries(self) -> HashMap<String, Vec<Entry>> {
        self.0
    }
}

impl From<HashMap<String, Vec<Entry>>> for BucketData {
    fn from(value: HashMap<String, Vec<Entry>>) -> Self {
        Self(value)
    }
}

impl fmt::Display for BucketData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let output: String = self
            .0
            .par_iter()
            .flat_map(|(bucket_name, entries)| {
                entries.par_iter().map(move |entry| {
                    format!(
                        "\n{}/{}  {}\n  {}",
                        &entry.app_name,
                        &bucket_name,
                        &entry.manifest.version,
                        &entry.manifest.description
                    )
                })
            })
            .collect();

        write!(f, "{output}")
    }
}
