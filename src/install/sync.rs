use std::path::PathBuf;

use git2::Repository;
use rayon::prelude::{IntoParallelRefIterator, ParallelIterator};
use tempfile::tempdir;

use super::db::{Bucket, DB};
use crate::config::Config;

pub type SyncResult = Result<(), SyncError>;

#[derive(Debug)]
pub enum SyncError {
    UnableToMkTmpDir,
    UnableToFetchRepo,
    UnableToGetHead,
    UnableToGetCommit,
}

#[derive(Debug, Default)]
pub struct Repo {
    pub name: String,
    pub commit_id: String,
    pub path: PathBuf,
}

impl Repo {
    pub fn fetch() -> SyncResult {
        let config = Config::read().unwrap();
        let repos = config.repos().unwrap();
        let temp_dir = tempdir().map_err(|_| SyncError::UnableToMkTmpDir)?;
        let temp_dir = temp_dir.path();

        repos.par_iter().for_each(|(name, url)| {
            let c = Repo::clone(name, url, &temp_dir.join(name)).unwrap();
            let c = Bucket::fetch_from(c).unwrap();
            let c = DB::create_from(c).unwrap();

            println!("{}", c.display());
        });

        Ok(())
    }

    fn clone(name: &String, url: &String, path: &PathBuf) -> Result<Repo, SyncError> {
        let repo = Repository::clone(url, &path).map_err(|_| SyncError::UnableToFetchRepo)?;
        let head = repo.head().map_err(|_| SyncError::UnableToGetHead)?;
        let commit_id = head
            .peel_to_commit()
            .map_err(|_| SyncError::UnableToGetCommit)?
            .id();

        Ok(Repo {
            name: name.into(),
            commit_id: commit_id.to_string(),
            path: path.to_path_buf(),
        })
    }
}
