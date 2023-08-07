use std::path::PathBuf;

use rayon::prelude::*;
use tokio::runtime::Runtime;
use trauma::{download::Download, downloader::DownloaderBuilder};
use url::Url;

use {
    super::{buckets::*, config::*, verify::Hash},
    crate::error::*,
};

#[derive(Debug)]
pub enum DownloadStatus {
    DownloadFailed,
    Downloaded,
    DownloadedAndVerified,
    DownloadedAndVerifyFailed,
    AlreadyInCache,
}

#[derive(Debug)]
struct Metadata(String, Hash, Url);

impl Metadata {
    #[inline]
    fn exists(&self) -> Result<bool, ScoopieError> {
        Ok(Config::cache_dir()?.join(&self.0).exists())
    }
}

#[derive(Debug)]
struct DownloadEntry {
    app_name: String,
    version: String,
    metadata: Vec<Metadata>,
}

trait FetchFromBucket<T> {
    type Error;

    fn fetch(app_name: T) -> Result<Self, Self::Error>
    where
        Self: Sized;

    fn fetch_from(app_name: T, bucket_name: T) -> Result<Self, Self::Error>
    where
        Self: Sized;
}

impl FetchFromBucket<&str> for DownloadEntry {
    type Error = ScoopieError;

    fn fetch(app_name: &str) -> Result<Self, Self::Error> {
        todo!()
    }

    fn fetch_from(app_name: &str, bucket_name: &str) -> Result<Self, Self::Error> {
        todo!()
    }
}

impl DownloadEntry {
    fn get(&self) -> Result<Vec<Download>, ScoopieError> {
        self.metadata
            .par_iter()
            .filter_map(|m| match m.exists() {
                Ok(true) => None,
                Ok(false) => Some(Ok(Download::new(&m.2, &m.0))),
                Err(err) => Some(Err(err)),
            })
            .collect()
    }

    fn verify(&self) -> Result<bool, ScoopieError> {
        let download_dir = Config::cache_dir()?;
        Ok(self
            .metadata
            .par_iter()
            .map(|m| m.1.verify(&PathBuf::from(download_dir.join(&m.0))).unwrap())
            .all(|v| v == true))
    }
}

pub struct Downloader {
    item: DownloadEntry,
    max_retries: u32,
    concurrent: usize,
    download_dir: PathBuf,
}

pub trait Builder<T> {
    type Error;
    fn build_for(app_name: T) -> Result<Self, Self::Error>
    where
        Self: Sized;
}

impl Builder<&str> for Downloader {
    type Error = ScoopieError;

    fn build_for(app_name: &str) -> Result<Self, Self::Error> {
        let query = app_name.trim().to_lowercase();

        let item = match app_name.split_once('/') {
            Some((bucket, app)) => DownloadEntry::fetch_from(app, bucket),
            None => DownloadEntry::fetch(&query),
        }?;

        let download_cfg = Config::read()?.download();
        let max_retries = download_cfg.max_retries;
        let concurrent = download_cfg.concurrent_downloads;

        let download_dir = Config::cache_dir()?;

        println!("Found: {}  v{}", item.app_name, item.version);

        Ok(Self {
            item,
            max_retries,
            concurrent,
            download_dir,
        })
    }
}

impl Downloader {
    pub fn download(&self, verify: bool) -> Result<DownloadStatus, ScoopieError> {
        let dm = DownloaderBuilder::new()
            .concurrent_downloads(self.concurrent)
            .retries(self.max_retries)
            .directory(self.download_dir.to_path_buf())
            .build();

        let downloads = self.item.get()?;

        let status = match downloads.is_empty() {
            true => DownloadStatus::AlreadyInCache,
            false => {
                let s = {
                    let rt = Runtime::new().unwrap();
                    rt.block_on(dm.download(&downloads))
                };

                match s.iter().any(|st| {
                    st.statuscode().is_server_error() || st.statuscode().is_client_error()
                }) {
                    true => DownloadStatus::DownloadFailed,
                    false => match (verify, self.item.verify()?) {
                        (true, true) => DownloadStatus::DownloadedAndVerified,
                        (true, false) => DownloadStatus::DownloadedAndVerifyFailed,
                        (false, _) => DownloadStatus::Downloaded,
                    },
                }
            }
        };

        Ok(status)
    }
}
