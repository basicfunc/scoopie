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
    #[error("{0}")]
    Download(DownloadError),
    #[error("Home directory unavailable")]
    HomeDirUnavailable,
    #[error("Config directory unavailable")]
    ConfigDirUnavailable,
    #[error("Cache directory unavailable")]
    CacheDirUnavailable,
    #[error("Repos directory unavailable")]
    BucketsDirUnavailable,
    #[error("Apps directory unavailable")]
    AppsDirUnavailable,
    #[error("Shims directory unavailable")]
    ShimsDirUnavailable,
    #[error("Persist directory unavailable")]
    PersistDirUnavailable,
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
pub enum DownloadError {
    #[error("App: \"{0}\" is not available in configured repos.")]
    NoAppFound(String),
    #[error("App: \"{0}\" is not available in {1} repo.")]
    NoAppFoundInBucket(String, String),
    #[error("Unable to find url for app: {0}.")]
    UnableToGetUrl(String),
    #[error("Invalid URL format in manifest: \"{0}.json\".")]
    InvalidUrlFormat(String),
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
    InvalidManifest,
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
    #[error("No buckets configured in config")]
    NoBucketsConfigured,
}

#[derive(Debug, Error)]
pub enum QueryError {
    #[error("Failed to retrieve data")]
    FailedToRetrieveData,
    // #[error("Found invalid JSON while retrieving data")]
    // InavlidJSONData,
    // #[error("Failed to query")]
    // FailedToQuery,
}
