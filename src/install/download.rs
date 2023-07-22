use std::path::PathBuf;

use lazy_static::lazy_static;

use crate::{bucket::*, config::*, error::ScoopieError};

lazy_static! {
    static ref APP_QUERY: &'static str =
        "SELECT app_name, manifest FROM manifests WHERE app_name LIKE ?";
}

#[allow(dead_code)]
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
        let _raw = Bucket::build_query(*APP_QUERY)?.execute(app.into())?;

        // for app in raw {
        //     let mainfest: Value = from_str(&app.manifest)
        //         .map_err(|_| ScoopieError::Bucket(crate::error::BucketError::InvalidManifest))?;

        //     let entry = mainfest
        //         .get("architecture")
        //         .unwrap_or(&Value::Null)
        //         .get(self.arch)
        //         .unwrap_or(&Value::Null);

        //     let entry = if !entry.is_null() {
        //         entry
        //     } else {
        //         mainfest.get("url").unwrap_or(&Value::Null)
        //     };

        //     let entry: DownloadEntry = serde_json::from_value(entry.clone()).unwrap();

        //     println!("Name: {}\n{:?}", app.app_name, entry);
        // }

        Ok(())
    }
}
