use std::fs::{self, File};
use std::io::{BufWriter, Write};
use std::path::PathBuf;
use std::{cmp::min, format};

use futures_util::StreamExt;
use indicatif::{ProgressBar, ProgressStyle};
use tokio::runtime::Runtime;
use url::Url;

use {
    super::verify::Hash,
    crate::core::{buckets::*, config::*},
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
struct DownloadEntry {
    app_name: String,
    version: String,
    urls: Vec<Url>,
    hashes: Vec<Hash>,
}

impl DownloadEntry {
    fn new(app_name: String, manifest: Manifest) -> Self {
        let urls = manifest.url();
        let hashes = manifest.hash();

        Self {
            app_name,
            version: manifest.version,
            urls,
            hashes,
        }
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

        Ok(Self {
            item,
            max_retries,
            concurrent,
            download_dir,
        })
    }
}

impl Downloader {
    pub fn download(&self, verify: bool) -> Result<Vec<DownloadStatus>, ScoopieError> {
        let app_name = &self.item.app_name;
        let version = &self.item.version;

        let rt = Runtime::new().unwrap();

        self.item
            .urls
            .iter()
            .map(|url| rt.block_on(download(&app_name, &version, url, &self.download_dir)))
            .collect::<Result<Vec<_>, _>>()
    }
}

fn extract_file_name(app_name: &str, version: &str, url: &Url) -> String {
    const INVALID_CHARS: [&str; 5] = [":", "#", "?", "&", "="];
    let url = url.as_str().to_string();

    let sanitized_fname = INVALID_CHARS
        .iter()
        .fold(url, |fname, &c| fname.replace(c, ""))
        .replace("//", "_")
        .replace("/", "_");

    format!("{app_name}#{version}#{sanitized_fname}")
}

pub async fn download(
    app_name: &str,
    version: &str,
    url: &Url,
    parent_dir: &PathBuf,
) -> Result<DownloadStatus, ScoopieError> {
    let file_name = extract_file_name(app_name, version, url);
    let filepath = parent_dir.join(&file_name);

    let client = reqwest::ClientBuilder::new()
        .build()
        .map_err(|_| ScoopieError::Download(DownloadError::UnableToGetClient))?;

    let res = client.get(url.as_str()).send().await.unwrap();
    let total_size = res.content_length().unwrap_or_default();

    if filepath.exists() && fs::metadata(&filepath).unwrap().len() == total_size {
        println!("Found in cache {app_name} (v{version})");
        Ok(DownloadStatus::AlreadyInCache)
    } else {
        // Indicatif setup
        let pb = ProgressBar::new(total_size);
        pb.set_style(ProgressStyle::default_bar()
        .template("{spinner:.blue} {msg} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec}, {eta})")
        .unwrap()
        .progress_chars("=>-"));
        pb.set_message(format!("Downloading {app_name} (v{version})"));

        // download chunks
        let mut file = BufWriter::new(File::create(&filepath).map_err(|_| {
            ScoopieError::Download(DownloadError::UnableToCreateFile(file_name.to_owned()))
        })?);
        let mut downloaded = 0u64;
        let mut stream = res.bytes_stream();

        while let Some(item) = stream.next().await {
            let chunk = item.map_err(|_| {
                ScoopieError::Download(DownloadError::UnableToGetChunk(app_name.into()))
            })?;

            file.write_all(&chunk).map_err(|_| {
                ScoopieError::Download(DownloadError::ChunkWrite(filepath.to_path_buf()))
            })?;

            let new = min(downloaded + (chunk.len() as u64), total_size);
            downloaded = new;
            pb.set_position(new);
        }

        file.flush().map_err(|_| {
            ScoopieError::Download(DownloadError::FlushFile(filepath.to_path_buf()))
        })?;

        pb.finish_with_message(format!("Downloaded {app_name} (v{version})"));

        Ok(DownloadStatus::Downloaded)
    }
}
