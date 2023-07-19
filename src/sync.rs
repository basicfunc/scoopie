use rayon::prelude::*;
use tempfile::tempdir;

use crate::{
    bucket::Bucket,
    config::*,
    database::Database,
    error::{ScoopieError, SyncError},
};

pub struct Sync {}

impl Sync {
    pub fn now() -> Result<Vec<Database>, ScoopieError> {
        let config = Config::read()?;
        let repos = config.repos()?;
        let temp_dir = tempdir().map_err(|_| ScoopieError::Sync(SyncError::UnableToMkTmpDir))?;
        let temp_dir = temp_dir.path();

        repos
            .par_iter()
            .map(|(name, url)| Bucket::fetch(name, url, &temp_dir.join(name)))
            .collect::<Result<Vec<_>, _>>()?
            .par_iter()
            .map(|bucket| Database::create(bucket))
            .collect()
    }
}
