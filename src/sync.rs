use rayon::prelude::*;
use tempfile::tempdir;

use crate::{
    bucket::Bucket,
    config::Config,
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
            .map(|(name, url)| Database::create(Bucket::fetch(name, url, &temp_dir.join(name))?))
            .collect()
    }
}
