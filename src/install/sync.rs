use rayon::prelude::*;
use tempfile::tempdir;

use crate::{
    bucket::*,
    config::*,
    error::{ScoopieError, SyncError},
};

pub struct Sync {}

impl Sync {
    pub fn now() -> Result<Vec<Bucket>, ScoopieError> {
        let buckets = Config::read()?.known_buckets();
        let temp_dir = tempdir().map_err(|_| ScoopieError::Sync(SyncError::UnableToMkTmpDir))?;
        let temp_dir = temp_dir.path();

        buckets
            .par_iter()
            .map(|(name, url)| Bucket::create(name, url, &temp_dir.join(name)))
            .collect::<Result<Vec<Bucket>, _>>()
    }
}
