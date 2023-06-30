use std::{collections::HashMap, fs, fs::File, io::Read, path::PathBuf};

use chrono::prelude::*;
use dirs::{data_dir, home_dir};
use git2::Repository;
use rayon::prelude::{IntoParallelRefIterator, ParallelIterator};
use serde::{Deserialize, Serialize};
use tempfile::tempdir;
use toml::Value;

pub type SyncResult = Result<Repo, SyncError>;
type RepoList = HashMap<String, String>;

#[derive(Debug)]
pub enum SyncError {
    HomeDirUnavailable,
    ConfigDirUnavailable,
    ConfigNotFound,
    PermissionDenied,
    InvalidData,
    Interrupted,
    UnexpectedEof,
    Unknown,
    InvalidToml,
    NoRepo,
    DataDirUnavailable,
    UnableToMkTmpDir,
    UnableToFetchRepo,
    UnableToGetHead,
    UnableToGetCommit,
    NoMainfest,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Repo {
    name: String,
    commit_id: String,
    mainfests: usize,
    fetch_time: String,
}

impl Repo {
    pub fn fetch() -> SyncResult {
        let repos = repo_list()?;
        let repos_dir = get_repos_dir()?;
        let temp_dir = tempdir().map_err(|_| SyncError::UnableToMkTmpDir)?;
        let temp_dir = temp_dir.path();

        repos.par_iter().for_each(|(name, url)| {
            let c = clone(name, url, &temp_dir.join(name));
            println!("{:?}", c);
        });

        Ok(Default::default())
    }
}

fn repo_list() -> Result<RepoList, SyncError> {
    let home_dir = home_dir().ok_or(SyncError::HomeDirUnavailable)?;
    let config_dir = home_dir.join(".config");

    if !config_dir.exists() {
        return Err(SyncError::ConfigDirUnavailable);
    }

    let scoopie_config = config_dir.join("scoopie.toml");

    if !scoopie_config.exists() {
        return Err(SyncError::ConfigNotFound);
    }

    let mut file = File::open(&scoopie_config).map_err(|e| match e.kind() {
        std::io::ErrorKind::NotFound => SyncError::ConfigNotFound,
        std::io::ErrorKind::PermissionDenied => SyncError::PermissionDenied,
        _ => SyncError::Unknown,
    })?;

    let mut toml = String::new();
    file.read_to_string(&mut toml).map_err(|e| match e.kind() {
        std::io::ErrorKind::InvalidData => SyncError::InvalidData,
        std::io::ErrorKind::Interrupted => SyncError::Interrupted,
        std::io::ErrorKind::UnexpectedEof => SyncError::UnexpectedEof,
        _ => SyncError::Unknown,
    })?;

    let toml: Value = toml::from_str(&toml).map_err(|_| SyncError::InvalidToml)?;
    let repos = toml.get("repos").ok_or(SyncError::NoRepo)?;

    let mut repo_list = RepoList::new();

    if let Value::Table(table) = repos {
        for (key, value) in table.iter() {
            if let Value::String(str_val) = value {
                repo_list.insert(key.clone(), str_val.clone());
            }
        }
    }

    Ok(repo_list)
}

fn get_repos_dir() -> Result<PathBuf, SyncError> {
    Ok(data_dir()
        .ok_or(SyncError::DataDirUnavailable)?
        .join(r"scoopie\repos"))
}

fn clone(name: &String, url: &String, path: &PathBuf) -> SyncResult {
    let repo = Repository::clone(url, path).map_err(|_| SyncError::UnableToFetchRepo)?;
    let head = repo.head().map_err(|_| SyncError::UnableToGetHead)?;
    let commit = head
        .peel_to_commit()
        .map_err(|_| SyncError::UnableToGetCommit)?
        .id();
    let time = Local::now();
    let path = path.join("bucket");
    let mainfests = fs::read_dir(path)
        .map_err(|_| SyncError::NoMainfest)?
        .filter_map(|entry| {
            if let Ok(entry) = entry {
                let file_name = entry.file_name();
                if let Some(file_name) = file_name.to_str() {
                    if file_name.ends_with(".json") {
                        return Some(());
                    }
                }
            }
            None
        })
        .count();

    let mut res = Repo::default();
    res.name = name.into();
    res.commit_id = format!("{commit}");
    res.fetch_time = time.to_string();
    res.mainfests = mainfests;

    Ok(res)
}
