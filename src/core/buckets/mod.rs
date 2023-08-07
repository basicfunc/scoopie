mod manifest;
mod metadata;
mod query;
mod sync;

pub use manifest::Manifest;
pub use metadata::write_default_metadata;
pub use query::*;
pub use sync::*;

use std::{collections::HashMap, fmt, format, write};

use serde::{Deserialize, Serialize};

pub type BucketName = String;
#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct Buckets(HashMap<BucketName, Bucket>);

impl Buckets {
    pub fn get(&self, bucket_name: &str) -> Option<Bucket> {
        self.0.get(bucket_name).cloned()
    }

    pub fn get_app(&self, app_name: &str) -> Option<Manifest> {
        self.0
            .values()
            .flat_map(|bucket| bucket.get(app_name))
            .next()
    }
}

pub type AppName = String;
#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct Bucket(HashMap<AppName, Manifest>);

impl Bucket {
    pub fn get(&self, app_name: &str) -> Option<Manifest> {
        self.0.get(app_name).cloned()
    }
}

impl fmt::Display for Buckets {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            self.0
                .iter()
                .flat_map(|(bucket_name, bucket)| {
                    bucket.0.iter().map(move |(app_name, manifest)| {
                        format!(
                            "\n{app_name}/{bucket_name}  (v{})\n  {}",
                            manifest.version, manifest.description
                        )
                    })
                })
                .collect::<String>()
        )
    }
}
