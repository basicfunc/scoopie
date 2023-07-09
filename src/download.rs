use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use serde_json::{from_str, Value};

use crate::{config::Config, error::ScoopieError, query::Query};

#[derive(Debug, Serialize, Deserialize)]
struct DownloadEntry {
    url: String,
    hash: String,
}

pub struct Downloader {
    download_dir: PathBuf,
    arch: &'static str,
}

impl Downloader {
    pub fn from() -> Result<Self, ScoopieError> {
        let download_dir = Config::cache_dir()?;
        let arch = Config::arch()?;

        Ok(Self { download_dir, arch })
    }

    pub fn download(&self, app: &str) -> Result<(), ScoopieError> {
        let raw = Query::builder("SELECT app_name, mainfest FROM mainfests WHERE app_name LIKE ?")?
            .run(&format!("{app}"))?;

        for app in raw {
            let mainfest: Value = from_str(&app.mainfest.mainfest)
                .map_err(|_| ScoopieError::Bucket(crate::error::BucketError::InvalidJSON))?;

            let entry = mainfest
                .get("architecture")
                .unwrap_or(&Value::Null)
                .get(self.arch)
                .unwrap_or(&Value::Null);

            let entry = if !entry.is_null() {
                entry
            } else {
                mainfest.get("url").unwrap_or(&Value::Null)
            };

            let entry: DownloadEntry = serde_json::from_value(entry.clone()).unwrap();

            println!("Name: {}\n{:?}", app.mainfest.app_name, entry);
        }

        Ok(())
    }
}
