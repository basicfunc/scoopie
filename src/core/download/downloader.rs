use std::format;
use std::fs::{self, File};
use std::io::{BufWriter, Write};
use std::iter::zip;

use colorized::*;
use spinoff::{spinners::Dots, Color::Blue, Spinner};
use url::Url;

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

fn download(url: &str, fname: &str, verify: Option<&Hash>) -> Result<DownloadStatus, ScoopieError> {
    let file_path = Config::cache_dir()?.join(fname);

    let request = ureq::get(url);

    let response = request.call().map_err(|_| ScoopieError::FailedToSendReq)?;

    let total_size = match response.header("content-length") {
        Some(size) => size.parse::<u64>().unwrap_or(0),
        None => 0,
    };

    let downloader = || -> Result<DownloadStatus, ScoopieError> {
        let mut file = BufWriter::new(
            File::create(&file_path)
                .map_err(|_| ScoopieError::UnableToCreateFile(fname.to_string()))?,
        );

        let mut downloaded = 0;
        let mut chunk = [0; 4096];

        let pkg_name = if fname.len() < 32 {
            fname.to_string()
        } else {
            format!("{}", &fname[..32])
        };

        let mut sp = Spinner::new(Dots, format!("Collecting {pkg_name}"), Blue);

        let mut stream = response.into_reader();

        loop {
            let bytes_read = stream
                .read(&mut chunk)
                .map_err(|_| ScoopieError::UnableToGetChunk(fname.into()))?;

            if bytes_read == 0 {
                break;
            }

            file.write_all(&chunk[..bytes_read])
                .map_err(|_| ScoopieError::ChunkWrite(file_path.to_path_buf()))?;
            downloaded += bytes_read;
            let percentage = (downloaded as f64 / total_size as f64) * 100.0;

            sp.update_text(format!(
                "Collecting {pkg_name}: {percentage:.2}% ({downloaded}/{total_size})"
            ));
        }

        file.flush()
            .map_err(|_| ScoopieError::FlushFile(file_path.to_path_buf()))?;

        match verify {
            Some(hash) => match hash.verify(&file_path)? {
                true => {
                    sp.success(&format!("Successfully downloaded and verified {fname}"));
                    Ok(DownloadStatus::DownloadedAndVerified(fname.into()))
                }
                false => {
                    sp.fail(&format!(
                        "Successfully downloaded but failed to verified {fname}"
                    ));
                    Err(ScoopieError::WrongDigest(fname.into()))
                }
            },
            None => {
                sp.success(&format!("Successfully downloaded {fname}"));
                Ok(DownloadStatus::Downloaded(fname.into()))
            }
        }
    };

    match file_path.exists() {
        true => match fs::metadata(&file_path)
            .or(Err(ScoopieError::FailedToGetMetadata(
                file_path.to_path_buf(),
            )))?
            .len()
            == total_size
        {
            true => Ok(DownloadStatus::AlreadyInCache(fname.into())),
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
