use std::{
    error::Error,
    fmt::{Display, Formatter, Result},
    path::PathBuf,
};

#[derive(Debug)]
pub enum ScoopieError {
    // Sync related Errors
    SyncUnableToFetchRepo,
    SyncUnableToGetHead,
    SyncUnableToGetCommit,

    // Bucket related errors
    BucketsNotFound,
    FailedToReadBucket(String),
    InvalidManifestInBucket,

    // Init related errors
    ConfigWriteWhileInit,
    DirAlreadyExists(PathBuf),

    // Config related errors
    ConfigNotFound,
    ConfigInvalidData,
    InterruptedConfig,
    UnexpectedEofInConfig,
    InvalidConfig,

    // Download related errors
    NoAppFound(String),
    FailedToSendReq,
    RequestFailed(String, String),
    NoAppFoundInBucket(String, String),
    FlushFile(PathBuf),
    ChunkWrite(PathBuf),
    UnableToGetChunk(String),
    UnableToCreateFile(String),
    WrongDigest(String),

    // Query Errors
    InvalidRegex(String),

    // Command Errors
    UnableToExecuteCmd,

    // Common Errors
    UnableToMkTmpDir,
    UserDirUnavailable,
    HomeDirUnavailable,
    CacheDirUnavailable,
    BucketsDirUnavailable,
    AppsDirUnavailable,
    ShimsDirUnavailable,
    PersistDirUnavailable,
    AbsoultePathResolve,
    EnvResolve,
    EnvRemove,
    EnvSet,
    FailedToMkdir(PathBuf),
    PermissionDenied,
    FileNotExist(PathBuf),
    FailedToOpenFile(PathBuf),
    FailedToReadFile(PathBuf),
    FailedToGetMetadata(PathBuf),
    UnableToOpenEnvRegistry,
    UnsupportedArch,
    UnableToGetEnvVar(String),
    NonUTF8Bytes,
    Unknown,
}

impl Error for ScoopieError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }

    fn cause(&self) -> Option<&dyn Error> {
        None
    }
}

impl Display for ScoopieError {
    fn fmt(&self, f: &mut Formatter) -> Result {
        match self {
            // Sync related Errors
            ScoopieError::SyncUnableToFetchRepo => write!(f, "Unable to fetch repository"),
            ScoopieError::SyncUnableToGetHead => write!(f, "Unable to get HEAD of repository"),
            ScoopieError::SyncUnableToGetCommit => {
                write!(f, "Unable to get latest commit of repository")
            }

            // Bucket related errors
            ScoopieError::BucketsNotFound => write!(f, "No buckets found"),
            ScoopieError::FailedToReadBucket(bucket) => {
                write!(f, "Failed to read bucket: {bucket}")
            }
            ScoopieError::InvalidManifestInBucket => write!(f, "Invalid JSON format"),

            // Init related errors
            ScoopieError::ConfigWriteWhileInit => write!(f, "Config write error"),
            ScoopieError::DirAlreadyExists(dir) => write!(f, "Unable to initialize scoopie as \"{}\" already exists", dir.display()),

            // Config related errors
            ScoopieError::ConfigNotFound => write!(f, "Config file not found"),
            ScoopieError::ConfigInvalidData => write!(f, "Invalid data in config"),
            ScoopieError::InterruptedConfig => write!(f, "Config found to be interrupted"),
            ScoopieError::UnexpectedEofInConfig => write!(f, "Unexpected EOF occured in config"),
            ScoopieError::InvalidConfig => {
                write!(f, "Config found to be not following config specs")
            }

            // Download related errors
            ScoopieError::NoAppFound(app) => {
                write!(f, "Unable to find: \"{app}\" in any configured bucket")
            }
            ScoopieError::NoAppFoundInBucket(app, bucket) => {
                write!(f, "Unable to find: \"{app}\" in \"{bucket}\" bucket")
            }
            ScoopieError::FailedToSendReq => {
                write!(
                    f,
                    "Failed to send request to server. Hint: Check your network settings"
                )
            }
            ScoopieError::RequestFailed(pkg, reason) => {
                write!(f, "Request failed to download: \"{pkg}\" due to {reason}.")
            }
            ScoopieError::FlushFile(file) => {
                write!(f, "Failed to close file: \"{}\" properly", file.display())
            }
            ScoopieError::ChunkWrite(file) => {
                write!(
                    f,
                    "Failed to write downloaded buffer to file: \"{}\"",
                    file.display()
                )
            }
            ScoopieError::UnableToGetChunk(pkg) => write!(
                f,
                "Failed to download buffer from server while downloading package: \"{pkg}\""
            ),
            ScoopieError::UnableToCreateFile(pkg) => write!(
                f,
                "Failed to create file while downloading package: \"{pkg}\""
            ),
            ScoopieError::WrongDigest(pkg) => write!(f, "Failed to verify package: \"{pkg}\" due to wrong digest in manifest. Hint: Please open a issue regarding this"),
            
            // Query Errors            
            ScoopieError::InvalidRegex(pat) => write!(f, "Query failed due to invalid regex pattern: \"{pat}\""),
            

            // Common Errors
            ScoopieError::UnableToMkTmpDir => write!(f, "Failed to make temporary directory"),
            ScoopieError::UserDirUnavailable => write!(f, "Failed to retrieve current user directory"),
            ScoopieError::HomeDirUnavailable => write!(f, "Failed to retrieve scoopie home directory. Hint: Check if $SCOOPIE_HOME is set correctly and it is properly initialized, if not then first run \"scoopie nuke\" and then \"scoopie init <your_desired_directory>\""),
            ScoopieError::CacheDirUnavailable => write!(f, "Failed to retrieve scoopie cache directory. Hint: Check if $SCOOPIE_HOME is set correctly and it is properly initialized, if not then first run \"scoopie nuke\" and then \"scoopie init <your_desired_directory>\""),
            ScoopieError::BucketsDirUnavailable => write!(f, "Failed to retrieve scoopie buckets directory. Hint: Check if $SCOOPIE_HOME is set correctly and it is properly initialized, if not then first run \"scoopie nuke\" and then \"scoopie init <your_desired_directory>\""),
            ScoopieError::AppsDirUnavailable => write!(f, "Failed to retrieve scoopie apps directory. Hint: Check if $SCOOPIE_HOME is set correctly and it is properly initialized, if not then first run \"scoopie nuke\" and then \"scoopie init <your_desired_directory>\""),
            ScoopieError::ShimsDirUnavailable => write!(f, "Failed to retrieve scoopie shims directory. Hint: Check if $SCOOPIE_HOME is set correctly and it is properly initialized, if not then first run \"scoopie nuke\" and then \"scoopie init <your_desired_directory>\""),
            ScoopieError::PersistDirUnavailable => write!(f, "Failed to retrieve scoopie persist directory. Hint: Check if $SCOOPIE_HOME is set correctly and it is properly initialized, if not then first run \"scoopie nuke\" and then \"scoopie init <your_desired_directory>\""),
            ScoopieError::AbsoultePathResolve => write!(f, "Failed to resolve path"),
            ScoopieError::EnvResolve => write!(f, "Failed to resolve environment variables"),
            ScoopieError::EnvRemove => write!(f, "Failed to remove environment variables"),
            ScoopieError::EnvSet => write!(f, "Failed to set environment variable"),
            ScoopieError::FailedToMkdir(dir) => write!(f, "Failed to create directory: \"{}\"", dir.display()),
            ScoopieError::PermissionDenied => write!(f, "Failed due to permission denial"),
            ScoopieError::FileNotExist(file) => write!(f, "Failed as file: \"{}\" not found", file.display()),
            ScoopieError::FailedToOpenFile(file) => write!(f, "Failed to open file: \"{}\"", file.display()),
            ScoopieError::FailedToReadFile(file) => write!(f, "Failed to read file: \"{}\"", file.display()),
            ScoopieError::FailedToGetMetadata(file) => write!(f, "Failed to get metadata of file: \"{}\"", file.display()),
            ScoopieError::UnableToOpenEnvRegistry => write!(f, "Failed to open Environment Registry to perform environment variable function"),
            ScoopieError::UnsupportedArch => write!(f, "Failed as current architecture is not supported."),
            ScoopieError::UnableToGetEnvVar(var) => write!(f, "Failed to get environment variable: \"{var}\" from current user's registry"),
            ScoopieError::Unknown => write!(f, "Unknow error occured"),
            ScoopieError::NonUTF8Bytes => write!(f, "Failed to convert to string due to invalid bytes"),
            ScoopieError::UnableToExecuteCmd => write!(f, "Unable to execute command")
        }
    }
}
