use std::format;
use std::fs::{self, File};
use std::io::{BufWriter, Write};

use url::Url;

use spinoff::{spinners, Color, Spinner};

use super::Hash;

use {
    crate::core::{buckets::*, config::*},
    crate::error::*,
    crate::utils::*,
};

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
    let file_path = Config::cache_dir()?.join(file_name);

    let res = ureq::get(url.as_str()).call().unwrap();

    let total_size = match res.header("content-length") {
        Some(size) => size.parse::<usize>().unwrap_or(0),
        None => 0,
    };

    let downloader = || -> Result<DownloadStatus, ScoopieError> {
        let mut file = BufWriter::new(File::create(&file_path).map_err(|_| {
            ScoopieError::Download(DownloadError::UnableToCreateFile(file_name.to_owned()))
        })?);

        let mut downloaded = 0;
        let mut stream = res.into_reader();
        let mut chunk = [0; 4096];

        let mut sp = Spinner::new(
            spinners::BouncingBall,
            format!("Downloading {app_name}"),
            Color::Blue,
        );

        loop {
            match stream.read(&mut chunk) {
                Ok(0) => break,

                Ok(bytes) => {
                    file.write_all(&chunk).map_err(|_| {
                        ScoopieError::Download(DownloadError::ChunkWrite(file_path.to_path_buf()))
                    })?;
                    downloaded += bytes;
                    let percentage = (downloaded as f64 / total_size as f64) * 100.0;
                    sp.update_text(format!("Downloading {app_name}: {:.2}%", percentage));
                }

                Err(_) => {
                    return Err(ScoopieError::Download(DownloadError::UnableToGetChunk(
                        app_name.into(),
                    )))
                }
            }
        }

        file.flush().map_err(|_| {
            ScoopieError::Download(DownloadError::FlushFile(file_path.to_path_buf()))
        })?;

        match verify {
            Some(hash) => match hash.verify(&file_path)? {
                true => {
                    sp.success("Downloaded and verified {app_name}");
                    Ok(DownloadStatus::DownloadedAndVerified(file_name.into()))
                }
                false => Err(ScoopieError::Download(DownloadError::WrongDigest(
                    app_name.into(),
                ))),
            },
            None => {
                sp.success("Downloaded {app_name}");
                Ok(DownloadStatus::Downloaded(file_name.into()))
            }
        }
    };

    match file_path.exists() {
        true => match fs::metadata(&file_path)
            .or(Err(ScoopieError::FailedToGetMetadata(
                file_path.to_path_buf(),
            )))?
            .len() as usize
            == total_size
        {
            true => Ok(DownloadStatus::AlreadyInCache(file_name.into())),
            false => {
                file_path.rm()?;
                downloader()
            }
        },
        false => downloader(),
    }
}
