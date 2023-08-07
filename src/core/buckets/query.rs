use std::collections::HashMap;
use std::fs::read_to_string;

use super::{Bucket, Buckets};

use crate::core::config::*;
use crate::error::*;

use rayon::prelude::*;
use regex::Regex;
use serde_json::from_str;

pub enum QueryTerm {
    App(String),
    Regex(String),
}

pub trait Query<T> {
    type Error;
    fn query(query: T) -> Result<Self, Self::Error>
    where
        Self: Sized;
}

impl Query<QueryTerm> for Buckets {
    type Error = ScoopieError;

    fn query(q: QueryTerm) -> Result<Self, Self::Error> {
        let buckets_dir = Config::buckets_dir()?;
        let buckets = Config::read()?.known_buckets();

        let query_func = match q {
            QueryTerm::App(_) => <Bucket as QueryBucket<App>>::query,
            QueryTerm::Regex(_) => <Bucket as QueryBucket<Rgx>>::query,
        };

        let query = match q {
            QueryTerm::App(s) => s,
            QueryTerm::Regex(s) => match s.contains(" ") {
                true => s
                    .split_whitespace()
                    .map(regex::escape)
                    .collect::<Vec<_>>()
                    .join("|"),
                false => s,
            },
        };

        let buckets: HashMap<_, _> = buckets
            .into_par_iter()
            .filter_map(|(name, _)| {
                let content = read_to_string(buckets_dir.join(&name)).unwrap();
                let bucket: Bucket = from_str(&content).unwrap();
                let bucket = query_func(bucket, &query);

                match !bucket.0.is_empty() {
                    true => Some((name, bucket)),
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
    fn query(self, q: &str) -> Self
    where
        Self: Sized;
}

impl QueryBucket<App> for Bucket {
    fn query(self, q: &str) -> Self {
        Bucket(
            self.0
                .into_par_iter()
                .filter_map(|(app_name, manifest)| match app_name == q {
                    true => Some((app_name, manifest)),
                    false => None,
                })
                .collect(),
        )
    }
}

impl QueryBucket<Rgx> for Bucket {
    fn query(self, q: &str) -> Self {
        let re = Regex::new(q).unwrap();
        Bucket(
            self.0
                .iter()
                .filter_map(|(app_name, manifest)| {
                    match re.is_match(&app_name) || re.is_match(&manifest.description) {
                        true => Some((app_name.clone(), manifest.clone())),
                        false => None,
                    }
                })
                .collect(),
        )
    }
}
