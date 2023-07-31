use std::format;

use rayon::prelude::*;
use reqwest::Url;
use serde_json::Value;
use tokio::runtime::Runtime;
use trauma::{download::Download as Downloader, downloader::DownloaderBuilder};

use {
    super::{
        bucket::*,
        config::*,
        manifest::{Links, Manifest},
        verify::Hash,
    },
    crate::error::*,
};

pub struct DownloadEntry {
    app_name: String,
    manifest: Manifest,
}

trait Fetch<T> {
    type Error;
    fn fetch(app_name: T) -> Result<Self, Self::Error>
    where
        Self: Sized;
    fn fetch_from(app_name: T, bucket: T) -> Result<Self, Self::Error>
    where
        Self: Sized;
}

impl Fetch<&str> for DownloadEntry {
    type Error = ScoopieError;

    fn fetch(app_name: &str) -> Result<Self, Self::Error> {
        let res = Bucket::query(QueryKind::APP, app_name.into())?;

        res.entries()
            .par_iter()
            .find_map_first(|(_, entries)| {
                entries.par_iter().find_first(|entry| entry.app_name == app_name).map(|entry| DownloadEntry {
                    app_name: entry.app_name.to_owned(),
                    manifest: entry.manifest.to_owned(),
                })
            })
            .ok_or_else(|| ScoopieError::Download(DownloadError::NoAppFound(app_name.into())))
    }

    fn fetch_from(app_name: &str, bucket: &str) -> Result<Self, Self::Error> {
        let res = Bucket::query(QueryKind::APP, app_name.into())?;

        res.entries()
            .get(bucket)
            .map(|entries| entries.par_iter().find_first(|x| x.app_name == app_name))
            .flatten()
            .map(|entry| DownloadEntry {
                app_name: app_name.to_owned(),
                manifest: entry.manifest.to_owned(),
            })
            .ok_or_else(|| ScoopieError::Download(DownloadError::NoAppFoundInBucket(app_name.into(), bucket.into())))
    }
}

impl TryFrom<&String> for DownloadEntry {
    type Error = ScoopieError;

    fn try_from(value: &String) -> Result<Self, Self::Error> {
        let app = value.trim();
        match app.split_once('/') {
            Some((bucket, app)) => DownloadEntry::fetch_from(app, bucket),
            None => DownloadEntry::fetch(&app),
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
            .map_err(|_| ScoopieError::Download(DownloadError::UnableToGetUrl(self.app_name.to_owned())))?;

        let urls: Vec<String> = match entry {
            Value::Object(v) => {
                let links = serde_json::from_value::<Links>(Value::Object(v.clone()))
                    .map_err(|_| ScoopieError::Download(DownloadError::InvalidUrlFormat(self.app_name.to_owned())))?
                    .url
                    .ok_or(ScoopieError::Download(DownloadError::InvalidUrlFormat(self.app_name.to_owned())))?;

                match links {
                    Value::String(s) => vec![s],
                    Value::Array(arr) => arr.par_iter().map(|a| a.as_str().unwrap_or_default().to_string()).collect(),
                    _ => vec![],
                }
            }
            Value::Array(v) => v.par_iter().map(|a| a.as_str().unwrap_or_default().to_string()).collect(),
            Value::String(url) => vec![url],
            _ => vec![],
        };

        let hash_entry = match &self.manifest.architecture {
            Some(x) => match arch {
                Arch::Bit64 => &x.bit_64.as_ref().unwrap().hash,
                Arch::Bit32 => &x.bit_32.as_ref().unwrap().hash,
                Arch::Arm64 => &x.arm64.as_ref().unwrap().hash,
            }
            .clone()
            .unwrap(),
            None => serde_json::to_value(&self.manifest.hash).unwrap(),
        };

        let hashes: Vec<String> = match hash_entry {
            Value::Object(v) => {
                let links = serde_json::from_value::<Links>(Value::Object(v.clone()))
                    .map_err(|_| ScoopieError::Download(DownloadError::InvalidUrlFormat(self.app_name.to_owned())))?
                    .hash
                    .ok_or(ScoopieError::Download(DownloadError::InvalidUrlFormat(self.app_name.to_owned())))?;

                match links {
                    Value::String(s) => vec![s],
                    Value::Array(arr) => arr.par_iter().map(|a| a.as_str().unwrap_or_default().to_string()).collect(),
                    _ => vec![],
                }
            }
            Value::Array(v) => v.par_iter().map(|a| a.as_str().unwrap_or_default().to_string()).collect(),
            Value::String(hash) => vec![hash],
            _ => vec![],
        };

        let entry: Vec<(String, String)> = urls.into_iter().zip(hashes.into_iter()).collect();

        type Entry = (String, Url, Hash);

        let entries: Vec<Entry> = entry
            .into_par_iter()
            .map(|(url, hash)| {
                let u = Url::parse(&url).unwrap();
                let f = format!("{}{}{}", &self.app_name, u.path(), u.fragment().unwrap_or_default()).replace("/", "_");
                let h = Hash::from(hash);

                (f, u, h)
            })
            .collect();

        let downloads: Vec<_> = entries.par_iter().map(|(f, u, _)| Downloader::new(u, f)).collect();

        let dm = DownloaderBuilder::new()
            .directory(download_dir.to_path_buf())
            .retries(download_cfg.max_retries)
            .concurrent_downloads(download_cfg.concurrent_downloads)
            .build();

        let rt = Runtime::new().unwrap();
        let s = rt.block_on(dm.download(&downloads));

        println!("{:?}", s);

        if verify {
            let v: Vec<_> = entries.par_iter().map(|(f, _, h)| h.verify(&download_dir.join(f))).collect();
            println!("{:?}", v);
        }

        Ok(())
    }
}
