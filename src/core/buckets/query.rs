use std::collections::HashMap;
use std::fs::read_to_string;

use super::{Bucket, Buckets, Manifest};

use crate::core::config::*;
use crate::error::*;

use rayon::prelude::*;
use regex_lite::Regex;
use serde_json::from_str;

pub trait Query<T>: Sized {
    type Error;
    fn query_fts(query: T) -> Result<Self, Self::Error>;
    fn query_app(query: T) -> Result<Self, Self::Error>;
}

impl Query<&str> for Buckets {
    type Error = ScoopieError;

    fn query_fts(query: &str) -> Result<Self, Self::Error> {
        let buckets_dir = Config::buckets_dir()?;
        let buckets = Config::read()?.list_buckets();

        let query = match query.contains(" ") {
            true => query
                .split_whitespace()
                .map(regex_lite::escape)
                .collect::<Vec<_>>()
                .join("|"),
            false => query.into(),
        };

        let predicate = |bucket_name: String| -> Result<(String, Bucket), ScoopieError> {
            let bucket_path = buckets_dir.join(&bucket_name);
            let content = read_to_string(&bucket_path)
                .map_err(|_| ScoopieError::FailedToReadFile(bucket_path))?;

            let bucket: Bucket = from_str(&content).map_err(|_| {
                ScoopieError::Bucket(BucketError::FailedToReadBucket(bucket_name.clone()))
            })?;

            let bucket: Bucket = bucket.query_fts(&query);

            Ok((bucket_name, bucket))
        };

        let buckets = buckets
            .into_par_iter()
            .map(predicate)
            .collect::<Result<HashMap<_, _>, _>>()?;

        Ok(Buckets(buckets))
    }

    fn query_app(query: &str) -> Result<Self, Self::Error> {
        let buckets_dir = Config::buckets_dir()?;
        let buckets = Config::read()?.list_buckets();

        let predicate = |bucket_name: String| -> Option<(String, Bucket)> {
            let content = read_to_string(buckets_dir.join(&bucket_name)).unwrap();
            let bucket: Bucket = from_str(&content).unwrap();
            let bucket: Bucket = bucket.query_app(&query);

            match !bucket.0.is_empty() {
                true => Some((bucket_name, bucket)),
                false => None,
            }
        };

        Ok(Buckets(
            buckets.into_par_iter().filter_map(predicate).collect(),
        ))
    }
}

trait QueryBucket<T>: Sized {
    fn query_fts(self, q: &str) -> Self;
    fn query_app(self, q: &str) -> Self;
}

impl QueryBucket<&str> for Bucket {
    fn query_fts(self, q: &str) -> Self {
        let predicate = |(app_name, manifest): &(String, Manifest)| -> bool {
            let re = Regex::new(q).unwrap();
            re.is_match(&app_name) || re.is_match(&manifest.description)
        };

        Bucket(self.0.into_par_iter().filter(predicate).collect())
    }

    fn query_app(self, q: &str) -> Self {
        let predicate = |(app_name, _): &(String, _)| -> bool { app_name == q };

        Bucket(self.0.into_par_iter().filter(predicate).collect())
    }
}
