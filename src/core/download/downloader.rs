use std::path::PathBuf;

use rayon::prelude::*;
use tokio::runtime::Runtime;
use trauma::{download::Download, downloader::DownloaderBuilder};
use url::Url;

use {
    super::verify::Hash,
    crate::core::{buckets::*, config::*},
    crate::error::*,
    crate::utils::*,
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

impl DownloadEntry {
    fn new(app_name: String, manifest: Manifest) -> Self {
        let urls = manifest.url();
        let hashes = manifest.hash();

        let metadata: Vec<Metadata> = Zipper::zip(urls.iter(), hashes.iter())
            .map(|(url, hash)| -> Metadata {
                let file = format!(
                    "{}_{}{}{}",
                    app_name,
                    &manifest.version,
                    url.path(),
                    url.fragment().unwrap_or("")
                )
                .replace("/", "_");

                Metadata(file, hash.clone(), url.clone())
            })
            .collect();

        Self {
            app_name,
            version: manifest.version,
            metadata,
        }
    }

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

        let item = match query.split_once('/') {
            Some((bucket, app)) => {
                let manifest = Buckets::query_app(app)?.get_app_from(app, bucket).ok_or(
                    ScoopieError::Download(DownloadError::NoAppFoundInBucket(
                        app.into(),
                        bucket.into(),
                    )),
                )?;

                DownloadEntry::new(app.into(), manifest)
            }
            None => {
                let manifest = Buckets::query_app(app_name)?.get_app(app_name).ok_or(
                    ScoopieError::Download(DownloadError::NoAppFound(app_name.into())),
                )?;

                DownloadEntry::new(app_name.into(), manifest)
            }
        };

        let download_cfg = Config::read()?.download();
        let max_retries = download_cfg.max_retries;
        let concurrent = download_cfg.concurrent_downloads;

        let download_dir = Config::cache_dir()?;

        println!("Found: {}  (v{})", item.app_name, item.version);

        let total_size: u64 = item.metadata.iter().map(|m| get_package_size(&m.2)).sum();

        println!("Total download size: {total_size} bytes");

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

fn get_package_size(url: &Url) -> u64 {
    // Create a reqwest Client
    let client = reqwest::blocking::Client::new();

    // Create a HEAD request using reqwest
    let response = client.get(url.clone()).send().unwrap();

    response.content_length().unwrap_or_default()
}
