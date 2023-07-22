use std::path::PathBuf;

use lazy_static::lazy_static;
use rayon::prelude::*;

use crate::{
    bucket::{manifest::Manifest, *},
    config::*,
    error::*,
};

lazy_static! {
    static ref APP_QUERY: &'static str =
        "SELECT app_name, manifest FROM manifests WHERE app_name LIKE ?";
}

#[derive(Debug)]
#[allow(dead_code)]
pub struct DownloadEntry {
    app_name: String,
    arch: &'static str,
    bucket_name: String,
    download_dir: PathBuf,
    manifest: Manifest,
}

impl TryFrom<&String> for DownloadEntry {
    type Error = ScoopieError;

    fn try_from(value: &String) -> Result<Self, Self::Error> {
        let download_dir = Config::cache_dir()?;
        let arch = Config::arch()?;

        let query = Bucket::build_query(*APP_QUERY)?;

        let (bucket, app_name) = match value.split_once('/') {
            Some(stmt) => (Some(stmt.0), stmt.1),
            None => (None, value.as_str()),
        };

        let raw_data = query.execute(app_name.into())?;

        if let Some((bucket_name, entry)) = raw_data.par_iter().find_map_any(|data| {
            data.entries.par_iter().find_map_any(|entry| {
                if (bucket.is_none() || Some(data.bucket_name.as_str()) == bucket)
                    && entry.app_name == app_name
                {
                    Some((data.bucket_name.to_owned(), entry))
                } else {
                    None
                }
            })
        }) {
            return Ok(DownloadEntry {
                bucket_name: bucket_name.into(),
                app_name: entry.app_name.clone(),
                manifest: entry.manifest.clone(),
                arch,
                download_dir,
            });
        }

        Err(match bucket {
            Some(bucket) => ScoopieError::Find(FindError::NoAppFoundInBucket(
                app_name.into(),
                bucket.into(),
            )),
            None => ScoopieError::Find(FindError::NoAppFound(value.into())),
        })
    }
}

pub trait Download {
    type Error;
    fn dowanload(&self) -> Result<(), Self::Error>;
}
