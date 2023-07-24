use std::{path::PathBuf, vec};

use lazy_static::lazy_static;
use rayon::prelude::*;
use serde_json::Value;
use tokio::runtime::Runtime;
use trauma::{download::Download as Downloader, downloader::DownloaderBuilder};

use crate::{
    bucket::{
        manifest::{Links, Manifest},
        *,
    },
    config::*,
    error::*,
};

lazy_static! {
    static ref APP_QUERY: &'static str =
        "SELECT app_name, manifest FROM manifests WHERE app_name LIKE ?";
}

#[derive(Debug)]
pub struct DownloadEntry {
    app_name: String,
    arch: Arch,
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

        match raw_data.par_iter().find_map_any(|data| {
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
            Some((bucket_name, entry)) => Ok(DownloadEntry {
                bucket_name: bucket_name.into(),
                app_name: entry.app_name.clone(),
                manifest: entry.manifest.clone(),
                arch,
                download_dir,
            }),
            None => Err(bucket.map_or(
                ScoopieError::Download(DownloadError::NoAppFound(value.into())),
                |b| {
                    ScoopieError::Download(DownloadError::NoAppFoundInBucket(
                        app_name.into(),
                        b.into(),
                    ))
                },
            )),
        }
    }
}

pub trait Download {
    type Error;
    fn download(&self, verify: bool) -> Result<(), Self::Error>;
}

impl Download for DownloadEntry {
    type Error = ScoopieError;

    fn download(&self, verify: bool) -> Result<(), ScoopieError> {
        let entry = self
            .manifest
            .architecture
            .as_ref()
            .map_or_else(
                || serde_json::to_value(&self.manifest.url),
                |v| match self.arch {
                    Arch::Bit64 => serde_json::to_value(&v.bit_64),
                    Arch::Bit32 => serde_json::to_value(&v.bit_32),
                    Arch::Arm64 => serde_json::to_value(&v.arm64),
                },
            )
            .map_err(|_| {
                ScoopieError::Download(DownloadError::UnableToGetUrl(self.app_name.to_owned()))
            })?;

        let urls: Vec<String> = match entry {
            Value::Object(v) => {
                let links = serde_json::from_value::<Links>(Value::Object(v.clone()))
                    .map_err(|_| {
                        ScoopieError::Download(DownloadError::InvalidUrlFormat(
                            self.app_name.to_owned(),
                        ))
                    })?
                    .url
                    .ok_or(ScoopieError::Download(DownloadError::InvalidUrlFormat(
                        self.app_name.to_owned(),
                    )))?;

                match links {
                    Value::String(s) => vec![s],
                    Value::Array(arr) => arr
                        .iter()
                        .map(|a| a.as_str().unwrap().to_string())
                        .collect(),
                    _ => vec![],
                }
            }
            Value::String(url) => vec![url],
            _ => vec![],
        };

        let downloads: Vec<_> = if !urls.is_empty() {
            urls.iter()
                .map(|u| Downloader::try_from(u.as_str()).unwrap())
                .collect()
        } else {
            vec![]
        };

        let dm = DownloaderBuilder::new()
            .directory(PathBuf::from(&self.download_dir))
            .build();

        let rt = Runtime::new().unwrap();

        let s = rt.block_on(dm.download(&downloads));

        println!("{:?}", s);

        Ok(())
    }
}
