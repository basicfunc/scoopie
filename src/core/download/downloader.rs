use std::fs::{self, File};
use std::io::{BufWriter, Write};
use std::{cmp::min, format};

use futures_util::StreamExt;
use indicatif::{ProgressBar, ProgressStyle};
use tokio::runtime::Runtime;
use url::Url;

use super::Hash;

use {
    crate::core::{buckets::*, config::*},
    crate::error::*,
    crate::utils::*,
};

const TEMPLATE: &'static str  = "{spinner:.blue} {msg} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec}, {eta})";
const PROGRESS_CHARS: &'static str = "=>-";

#[derive(Debug)]
pub enum DownloadStatus {
    Downloaded(String),
    DownloadedAndVerified(String),
    AlreadyInCache(String),
}

pub struct Downloader;

impl Downloader {
    pub fn download(app_name: &str, verify: bool) -> Result<Vec<DownloadStatus>, ScoopieError> {
        let query = app_name.trim().to_lowercase();

        let (app_name, manifest) =
            match query.split_once('/') {
                Some((bucket, app)) => {
                    let manifest = Buckets::query_app(app)?.get_app_from(app, bucket).ok_or(
                        ScoopieError::Download(DownloadError::NoAppFoundInBucket(
                            app.into(),
                            bucket.into(),
                        )),
                    )?;

                    (app, manifest)
                }
                None => {
                    let manifest = Buckets::query_app(&query)?.get_app(&query).ok_or(
                        ScoopieError::Download(DownloadError::NoAppFound(query.into())),
                    )?;

                    (app_name, manifest)
                }
            };

        let entries = manifest.download_entry(&app_name);

        entries
            .iter()
            .map(|(url, hash, file)| {
                let hash = if verify { Some(hash) } else { None };
                download(&app_name, url, file, hash)
            })
            .collect::<Result<Vec<_>, _>>()
    }
}

fn download(
    app_name: &str,
    url: &Url,
    file_name: &str,
    verify: Option<&Hash>,
) -> Result<DownloadStatus, ScoopieError> {
    let rt =
        Runtime::new().map_err(|_| ScoopieError::Download(DownloadError::FailedToCreateRunTime))?;

    rt.block_on(async {
        let file_path = Config::cache_dir()?.join(file_name);

        let client = reqwest::ClientBuilder::new()
            .build()
            .map_err(|_| ScoopieError::Download(DownloadError::UnableToGetClient))?;

        let res = client.get(url.as_str()).send().await.unwrap();
        let total_size = res.content_length().unwrap_or_default();

        let downloader = || async {
            let style = ProgressStyle::with_template(TEMPLATE)
                .unwrap()
                .progress_chars(PROGRESS_CHARS);

            let pb = ProgressBar::new(total_size);
            pb.set_style(style);
            pb.set_message(format!("Downloading {app_name}"));

            let mut file = BufWriter::new(File::create(&file_path).map_err(|_| {
                ScoopieError::Download(DownloadError::UnableToCreateFile(file_name.to_owned()))
            })?);
            let mut downloaded = 0u64;
            let mut stream = res.bytes_stream();

            while let Some(item) = stream.next().await {
                let chunk = item.map_err(|_| {
                    ScoopieError::Download(DownloadError::UnableToGetChunk(app_name.into()))
                })?;

                file.write_all(&chunk).map_err(|_| {
                    ScoopieError::Download(DownloadError::ChunkWrite(file_path.to_path_buf()))
                })?;

                let new = min(downloaded + (chunk.len() as u64), total_size);
                downloaded = new;
                pb.set_position(new);
            }

            file.flush().map_err(|_| {
                ScoopieError::Download(DownloadError::FlushFile(file_path.to_path_buf()))
            })?;

            if let Some(hash) = verify {
                if hash.verify(&file_path)? {
                    pb.finish_with_message(format!("Downloaded and verified {app_name}"));
                    Ok(DownloadStatus::DownloadedAndVerified(file_name.into()))
                } else {
                    Err(ScoopieError::Download(DownloadError::WrongDigest(
                        app_name.into(),
                    )))
                }
            } else {
                pb.finish_with_message(format!("Downloaded {app_name}"));
                Ok(DownloadStatus::Downloaded(file_name.into()))
            }
        };

        match file_path.exists() {
            true => match fs::metadata(&file_path)
                .map_err(|_| ScoopieError::FailedToGetMetadata(file_path.to_path_buf()))?
                .len()
                .eq(&total_size)
            {
                true => Ok(DownloadStatus::AlreadyInCache(file_name.into())),
                false => {
                    file_path.rm()?;
                    downloader().await
                }
            },
            false => downloader().await,
        }
    })
}
