use std::collections::HashMap;
use std::fs::read_to_string;

use super::{Bucket, Buckets};

use crate::core::config::*;
use crate::error::*;

use regex::Regex;
use serde_json::from_str;

pub trait Query {
    type Error;
    fn query(q: &str) -> Result<Self, Self::Error>
    where
        Self: Sized;
}

impl Query for Buckets {
    type Error = ScoopieError;

    fn query(q: &str) -> Result<Self, Self::Error> {
        let buckets_dir = Config::buckets_dir()?;
        let buckets = Config::read()?.known_buckets();

        let query = match q.contains(" ") {
            true => q
                .split_whitespace()
                .map(|phrases| regex::escape(phrases))
                .collect::<Vec<_>>()
                .join("|"),
            false => q.into(),
        };

        let buckets: HashMap<_, _> = buckets
            .iter()
            .filter_map(|(name, _)| {
                let content = read_to_string(buckets_dir.join(name)).unwrap();
                let bucket: Bucket = from_str(&content).unwrap();
                let bucket = QueryBucket::<Rgx>::query(&bucket, &query);

                match !bucket.0.is_empty() {
                    true => Some((name.to_string(), bucket)),
                    false => None,
                }
            })
            .collect();

        Ok(Buckets(buckets))
    }
}

struct App;
struct Rgx;

trait QueryBucket<T> {
    fn query(&self, q: &str) -> Self
    where
        Self: Sized;
}

impl QueryBucket<App> for Bucket {
    fn query(&self, q: &str) -> Self {
        Bucket(
            self.0
                .iter()
                .filter_map(|(app_name, manifest)| match app_name == q {
                    true => Some((app_name.clone(), manifest.clone())),
                    false => None,
                })
                .collect(),
        )
    }
}

impl QueryBucket<Rgx> for Bucket {
    fn query(&self, q: &str) -> Self {
        let re = Regex::new(q).unwrap();
        Bucket(
            self.0
                .iter()
                .filter_map(|(app_name, manifest)| {
                    match re.is_match(&app_name) || re.is_match(&manifest.to_string()) {
                        true => Some((app_name.clone(), manifest.clone())),
                        false => None,
                    }
                })
                .collect(),
        )
    }
}
