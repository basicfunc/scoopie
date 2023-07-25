use std::{collections::HashMap, format, path::PathBuf};

use lazy_static::lazy_static;
use rayon::prelude::*;
use reqwest::Url;
use serde_json::Value;
use tokio::runtime::Runtime;
use trauma::{download::Download as Downloader, downloader::DownloaderBuilder};

use crate::{
    bucket::{
        data::BucketData,
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

#[derive(Debug, Clone)]
pub struct DownloadEntry {
    app_name: String,
    bucket_name: String,
    manifest: Manifest,
}

trait Fetch<T> {
    type Error;
    fn from_all(&self, app_name: T) -> Result<DownloadEntry, Self::Error>;

    fn from(&self, app_name: T, bucket: T) -> Result<DownloadEntry, Self::Error>;
}

impl Fetch<&str> for BucketData {
    type Error = ScoopieError;

    fn from_all(&self, app_name: &str) -> Result<DownloadEntry, Self::Error> {
        self.0
            .par_iter()
            .find_map_first(|(bucket_name, entries)| {
                entries
                    .par_iter()
                    .find_first(|entry| entry.app_name == app_name)
                    .map(|entry| DownloadEntry {
                        app_name: entry.app_name.to_owned(),
                        bucket_name: bucket_name.to_owned(),
                        manifest: entry.manifest.to_owned(),
                    })
            })
            .ok_or_else(|| ScoopieError::Download(DownloadError::NoAppFound(app_name.into())))
    }

    fn from(&self, app_name: &str, bucket: &str) -> Result<DownloadEntry, Self::Error> {
        self.0
            .get(bucket)
            .map(|entries| entries.par_iter().find_first(|x| x.app_name == app_name))
            .flatten()
            .map(|entry| DownloadEntry {
                app_name: app_name.to_owned(),
                bucket_name: bucket.to_owned(),
                manifest: entry.manifest.to_owned(),
            })
            .ok_or_else(|| {
                ScoopieError::Download(DownloadError::NoAppFoundInBucket(
                    app_name.into(),
                    bucket.into(),
                ))
            })
    }
}

impl TryFrom<&String> for DownloadEntry {
    type Error = ScoopieError;

    fn try_from(value: &String) -> Result<Self, Self::Error> {
        let query = Bucket::build_query(*APP_QUERY)?;

        match value.split_once('/') {
            Some((bucket, app)) => query.execute(app.into())?.from(app, bucket),
            None => query.execute(value.into())?.from_all(&value),
        }
    }
}

pub trait Download {
    type Error;
    fn download(&self, verify: bool) -> Result<(), Self::Error>;
}

impl Download for DownloadEntry {
    type Error = ScoopieError;

    fn download(&self, _verify: bool) -> Result<(), ScoopieError> {
        let cfg = Config::read()?;
        let arch = Config::arch()?;
        let download_cfg = cfg.download();
        let download_dir = Config::cache_dir()?;

        let entry = self
            .manifest
            .architecture
            .as_ref()
            .map_or_else(
                || serde_json::to_value(&self.manifest.url),
                |v| match arch {
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
                        .par_iter()
                        .map(|a| a.as_str().unwrap().to_string())
                        .collect(),
                    _ => vec![],
                }
            }
            Value::String(url) => vec![url],
            _ => vec![],
        };

        let download_entry: HashMap<String, Url> = urls
            .par_iter()
            .map(|url| {
                let u = Url::parse(&url).unwrap();
                let f = format!(
                    "{}{}{}",
                    &self.app_name,
                    u.path(),
                    u.fragment().unwrap_or_default()
                )
                .replace("/", "_");

                (f, u)
            })
            .collect();

        let downloads: Vec<_> = download_entry
            .par_iter()
            .map(|(f, u)| Downloader::new(u, f))
            .collect();

        let dm = DownloaderBuilder::new()
            .directory(PathBuf::from(download_dir))
            .retries(download_cfg.max_retries)
            .concurrent_downloads(download_cfg.concurrent_downloads)
            .build();

        let rt = Runtime::new().unwrap();

        let _s = rt.block_on(dm.download(&downloads));

        // println!("{:?}", _s);

        Ok(())
    }
}
