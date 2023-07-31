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
        let download_cfg = cfg.download();
        let download_dir = Config::cache_dir()?;

        let entries: Vec<(String, Url, Hash)> = self
            .manifest
            .url()
            .into_iter()
            .zip(self.manifest.hash().into_iter())
            .map(|(url, hash)| {
                let u = Url::parse(&url).unwrap();
                let f = format!("{}{}{}", &self.app_name, u.path(), u.fragment().unwrap_or_default()).replace("/", "_");
                let h = Hash::from(hash);

                (f, u, h)
            })
            .collect();

        println!("{:?}", entries);

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
