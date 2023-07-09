use std::path::PathBuf;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum ScoopieError {
    #[error("While syncing repositories, {0}")]
    Sync(SyncError),
    #[error("{0}")]
    Database(DatabaseError),
    #[error("{0}")]
    Bucket(BucketError),
    #[error("{0}")]
    Init(InitError),
    #[error("While reading config, {0}")]
    Config(ConfigError),
    #[error("{0}")]
    Query(QueryError),
    #[error("Home directory unavailable")]
    HomeDirUnavailable,
    #[error("Config directory unavailable")]
    ConfigDirUnavailable,
    #[error("Cache directory unavailable")]
    CacheDirUnavailable,
    #[error("Data directory unavailable")]
    DataDirUnavailable,
    #[error("Repos directory unavailable")]
    ReposDirUnavailable,
    #[error("Directory already exists: {0:?}")]
    DirAlreadyExists(PathBuf),
    #[error("While resolving absolute path")]
    AbsoultePathResolve,
    #[error("While resolving environment variable")]
    EnvResolve,
    #[error("While removing environment variable")]
    EnvRemove,
    #[error("Failed to create directory: {0:?}")]
    FailedToMkdir(PathBuf),
    #[error("Failed to create file: {0:?}")]
    FailedToTouch(PathBuf),
    #[error("Permission Denied")]
    PermissionDenied,
    #[error("{0:?} file not found")]
    FileNotExist(PathBuf),
    #[error("Your CPU architecture is not supported")]
    UnknownArch,
    #[error("Unknown")]
    Unknown,
}

#[derive(Debug, Error)]
pub enum SyncError {
    #[error("Unable to make temporary directory")]
    UnableToMkTmpDir,
    #[error("Unable to fetch repository")]
    UnableToFetchRepo,
    #[error("Unable to get HEAD of repositoy")]
    UnableToGetHead,
    #[error("Unable to get latest commit of repositoy")]
    UnableToGetCommit,
}

#[derive(Debug, Error)]
pub enum DatabaseError {
    #[error("Unable to open database")]
    UnableToOpen,
    #[error("Failed to create database")]
    FailedToCreateTable,
    #[error("Failed to make statement for database")]
    FailedToMkStmt,
    #[error("Failed to insert mainfest to database")]
    FailedInsertion,
    #[error("Failed to set pragmas for database optimizations")]
    FailedToSetPragma,
    #[error("Failed to begin transction")]
    FailedToBeginTransaction,
    #[error("Failed to commit transction")]
    FailedToCommitTransaction,
}

#[derive(Debug, Error)]
pub enum BucketError {
    #[error("No buckets found")]
    BucketsNotFound,
    #[error("Not Found")]
    NotFound,
    #[error("Uanble to read mainfest")]
    MainfestRead,
    #[error("Invalid JSON format")]
    InvalidJSON,
}

#[derive(Debug, Error)]
pub enum InitError {
    #[error("TOML parse error")]
    TOMLParse,
    #[error("Config write error")]
    ConfigWrite,
    #[error("Unable to set environment variable")]
    UnableToSetEnvVar,
}

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("Config file not found")]
    ConfigNotFound,
    #[error("Invalid data")]
    InvalidData,
    #[error("Interrupted")]
    Interrupted,
    #[error("Unexpected end of file")]
    UnexpectedEof,
    #[error("Invalid TOML")]
    InvalidToml,
    #[error("No repo")]
    NoRepo,
    #[error("Unable to get entry in repos dir")]
    UnableToGetEntry,
    #[error("Unable to get metadata of database files")]
    UnableToGetMetadata,
}

#[derive(Debug, Error)]
pub enum QueryError {
    #[error("Scoopie working directory unavailable")]
    ScoopieWorkingDirUnavailable,
    #[error("Invalid Query")]
    InvalidQuery,
    #[error("Failed to retrieve data")]
    FailedToRetrieveData,
    #[error("Found invalid JSON while retrieving data")]
    InavlidJSONData,
    #[error("Failed to query")]
    FailedToQuery,
}
