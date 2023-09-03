use std::format;
use std::fs::{self, File};
use std::io::{BufWriter, Write};
use std::iter::zip;

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

        let (app_name, manifest) = match query.split_once('/') {
            Some((bucket, app)) => {
                let manifest = Buckets::query_app(&app)?
                    .get_app_from(&app, &bucket)
                    .ok_or(ScoopieError::NoAppFoundInBucket(app.into(), bucket.into()))?;

                (app, manifest)
            }
            None => {
                let app_name = &query;
                let manifest = Buckets::query_app(&app_name)?
                    .get_app(&query)
                    .ok_or(ScoopieError::NoAppFound(app_name.into()))?;

                (app_name.as_str(), manifest)
            }
        };

        let version = &manifest.version;
        let urls = manifest.url();
        let hashes = manifest.hash();

        zip(urls, hashes)
            .map(|(url, hash)| {
                let file = extract_file_name(&app_name, version, &url);
                let hash = if verify { Some(&hash) } else { None };
                download(url.as_str(), &file, hash)
            })
            .collect::<Result<Vec<_>, _>>()
    }
}

fn download(
    url: &str,
    file_name: &str,
    verify: Option<&Hash>,
) -> Result<DownloadStatus, ScoopieError> {
    let file_path = Config::cache_dir()?.join(file_name);

    let res = ureq::get(url).call().unwrap();

    let total_size = match res.header("content-length") {
        Some(size) => size.parse::<usize>().unwrap_or(0),
        None => 0,
    };

    let downloader = || -> Result<DownloadStatus, ScoopieError> {
        let mut file = BufWriter::new(
            File::create(&file_path)
                .map_err(|_| ScoopieError::UnableToCreateFile(file_name.to_string()))?,
        );

        let mut downloaded = 0;
        let mut stream = res.into_reader();
        let mut chunk = [0; 1024 * 1024];

        let mut sp = Spinner::new(
            spinners::Dots,
            format!("Downloading {file_name}"),
            Color::Blue,
        );

        loop {
            match stream
                .read(&mut chunk)
                .map_err(|_| ScoopieError::UnableToGetChunk(file_name.into()))?
            {
                0 => break,
                bytes => {
                    file.write_all(&chunk[..bytes])
                        .map_err(|_| ScoopieError::ChunkWrite(file_path.to_path_buf()))?;
                    downloaded += bytes;
                    let percentage = (downloaded as f64 / total_size as f64) * 100.0;
                    sp.update_text(format!("Downloading {file_name}: {:.2}%", percentage));
                }
            }
        }

        file.flush()
            .map_err(|_| ScoopieError::FlushFile(file_path.to_path_buf()))?;

        match verify {
            Some(hash) => match hash.verify(&file_path)? {
                true => {
                    sp.success(&format!("Downloaded and verified {file_name}"));
                    Ok(DownloadStatus::DownloadedAndVerified(file_name.into()))
                }
                false => Err(ScoopieError::WrongDigest(file_name.into())),
            },
            None => {
                sp.success(&format!("Downloaded {file_name}"));
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

fn extract_file_name(app_name: &str, version: &str, url: &Url) -> String {
    const TO_BE_REMOVED_CHARS: &[&str] = &[":", "#", "?", "&", "="];
    const TO_BE_REPLACED_CHARS: &[&str] = &["//", "/", "+"];

    let url = url.path().to_string();

    let sanitized_fname = TO_BE_REMOVED_CHARS
        .iter()
        .fold(url, |fname, &c| fname.replace(c, ""));

    let sanitized_fname = TO_BE_REPLACED_CHARS
        .iter()
        .fold(sanitized_fname, |fname, &c| fname.replace(c, "_"));

    format!("{app_name}#{version}#{sanitized_fname}")
}
