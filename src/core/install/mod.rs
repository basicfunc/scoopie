mod installer;

use std::{ffi::OsStr, path::PathBuf};

use crate::core::{
    buckets::{Buckets, Query},
    config::*,
    download::*,
};

use crate::utils::*;

pub fn install(app: &str) {
    let query = app.trim().to_lowercase();

    let (app_name, manifest) = match query.split_once('/') {
        Some((bucket, app_name)) => (
            app_name,
            Buckets::query_app(app_name)
                .unwrap()
                .get_app_from(app_name, bucket),
        ),
        None => (app, Buckets::query_app(app).unwrap().get_app(app)),
    };

    let manifest = manifest.unwrap();
    let file_name = Downloader::download(app_name, true).unwrap();
    let cache_dir = Config::cache_dir().unwrap();

    let srcs = file_name
        .iter()
        .map(|f| match f {
            DownloadStatus::Downloaded(s) => s,
            DownloadStatus::DownloadedAndVerified(s) => s,
            DownloadStatus::AlreadyInCache(s) => s,
        })
        .map(|s| cache_dir.join(s))
        .collect::<Vec<_>>();

    let version = &manifest.version;

    let app_dir = Config::app_dir().unwrap();
    let app_dir = app_dir.join(app_name);
    let app_dir = app_dir.join(version);

    if !app_dir.exists() {
        PathBuf::create(app_dir.clone()).unwrap();
        use sevenz_rust::decompress_file;
        for src in srcs {
            if src
                .extension()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string()
                == "7z"
            {
                decompress_file(src, &app_dir).unwrap();
            }
        }
    }

    println!("{app_name}\n{manifest:#?}\n{file_name:#?} at {app_dir:?}");
}
