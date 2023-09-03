use std::path::PathBuf;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum ScoopieError {
    // Sync Errors
    #[error("Unable to fetch repository")]
    SyncUnableToFetchRepo,
    #[error("Unable to get HEAD of repositoy")]
    SyncUnableToGetHead,
    #[error("Unable to get latest commit of repositoy")]
    SyncUnableToGetCommit,

    // Bucket related errors
    #[error("No buckets found")]
    BucketsNotFound,
    #[error("Failed to read bucket: {0}")]
    FailedToReadBucket(String),
    #[error("Invalid JSON format")]
    InvalidManifestInBucket,

    // Init related errors
    #[error("Config write error")]
    ConfigWriteWhileInit,

    // Config related errors
    #[error("Config file not found")]
    ConfigNotFound,
    #[error("Invalid data")]
    ConfigInvalidData,
    #[error("Interrupted")]
    InterruptedConfig,
    #[error("Unexpected end of file")]
    UnexpectedEofInConfig,
    #[error("Invalid TOML")]
    InvalidConfig,

    // Download related errors
    #[error("App: \"{0}\" is not available in configured repos.")]
    NoAppFound(String),
    #[error("App: \"{0}\" is not available in {1} repo.")]
    NoAppFoundInBucket(String, String),
    #[error("Unable to write to file: {0:?}")]
    FlushFile(PathBuf),
    #[error("Unable to write downloaded chunk to file: {0:?}")]
    ChunkWrite(PathBuf),
    #[error("Unable to download chunk while downloading app: {0}")]
    UnableToGetChunk(String),
    #[error("Unable to create file while downloading app: {0}")]
    UnableToCreateFile(String),
    #[error("Found wrong digest for {0}")]
    WrongDigest(String),

    // Query Errors
    #[error("Invalid Regex: {0}")]
    InvalidRegex(String),

    // Common Errors
    #[error("Unable to make temporary directory")]
    UnableToMkTmpDir,
    #[error("User directory unavailable")]
    UserDirUnavailable,
    #[error("Home directory unavailable")]
    HomeDirUnavailable,
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
    #[error("While setting value for environment variable")]
    EnvSet,
    #[error("Failed to create directory: {0:?}")]
    FailedToMkdir(PathBuf),
    #[error("Permission Denied")]
    PermissionDenied,
    #[error("{0:?} file not found")]
    FileNotExist(PathBuf),
    #[error("Failed to open file: {0:?}")]
    FailedToOpenFile(PathBuf),
    #[error("Failed to read file: {0:?}")]
    FailedToReadFile(PathBuf),
    #[error("Failed to get metadata for file: {0:?}")]
    FailedToGetMetadata(PathBuf),
    #[error("Unable to open environment in current user registry")]
    UnableToOpenEnvRegistry,
    #[error("Your CPU architecture is not supported")]
    UnsupportedArch,
    #[error("Unable to get environment variable: {0}")]
    UnableToGetEnvVar(String),
    #[error("Unknown")]
    Unknown,
}
