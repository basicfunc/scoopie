use std::path::PathBuf;

use git2::Repository;
use rayon::prelude::*;
use tempfile::tempdir;

use crate::config::Config;
use crate::error::{ScoopieError, SyncError};
use crate::install::db::{Bucket, DB};

#[derive(Debug, Default)]
pub struct Repo {
    pub name: String,
    pub commit_id: String,
    pub path: PathBuf,
}

impl Repo {
    pub fn fetch() -> Result<(), ScoopieError> {
        let config = Config::read().unwrap();
        let repos = config.repos().unwrap();
        let temp_dir = tempdir().map_err(|_| ScoopieError::Sync(SyncError::UnableToMkTmpDir))?;
        let temp_dir = temp_dir.path();

        repos.par_iter().for_each(|(name, url)| {
            let c = Repo::clone(name, url, &temp_dir.join(name)).unwrap();
            let c = Bucket::fetch_from(c).unwrap();
            let c = DB::create_from(c).unwrap();

            println!("{:?}", c);
        });

        Ok(())
    }

    fn clone(name: &String, url: &String, path: &PathBuf) -> Result<Repo, ScoopieError> {
        let repo = Repository::clone(url, &path)
            .map_err(|_| ScoopieError::Sync(SyncError::UnableToFetchRepo))?;
        let head = repo
            .head()
            .map_err(|_| ScoopieError::Sync(SyncError::UnableToGetHead))?;
        let commit_id = head
            .peel_to_commit()
            .map_err(|_| ScoopieError::Sync(SyncError::UnableToGetCommit))?
            .id();

        Ok(Repo {
            name: name.into(),
            commit_id: commit_id.to_string(),
            path: path.to_path_buf(),
        })
    }
}
