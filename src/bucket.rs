use argh::FromArgs;
use chrono::{DateTime, Local, NaiveDateTime, Offset};
use git2::Repository;
use json_to_table::{json_to_table, JsonTable};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{
    fs::{self, remove_dir_all, OpenOptions},
    io::Write,
    path::PathBuf,
};

#[derive(FromArgs, PartialEq, Debug)]
/// Manage Scoopie buckets
#[argh(subcommand, name = "bucket")]
pub struct BucketCommand {
    #[argh(option)]
    /// add new bucket
    add: Option<String>,
    #[argh(option)]
    /// remove a bucket
    rm: Option<String>,
    #[argh(switch)]
    /// list all buckets
    list: bool,
}

#[derive(Debug)]
enum AddError {
    AlreadyExists,
    NameResolution,
    PeelCommit,
    InvalidUrl,
    NoMem,
    Os,
    BadRepoState,
    InvalidIndex,
    Object,
    Net,
    Ssl,
    Submodule,
    Thread,
    FetchHead,
    Fs,
    Hash,
    Http,
    Unknown,
    TimeConversion,
    Serialization,
    OpenFile(PathBuf),
    Write,
}

#[derive(Debug)]
enum RemoveError {
    Failed(PathBuf, Box<dyn std::error::Error>),
    DirNotExist(PathBuf),
    LackOfPermission,
}

#[derive(Debug)]
enum ListError {
    NotFound(PathBuf),
    PermissionDenied,
    InvalidInput,
    InvalidData,
    Interrupted,
    Io,
    Syntax,
    Data,
    UnexpectedEof,
    Unknown,
}

#[derive(Debug)]
enum BucketError {
    Add(AddError),
    Remove(RemoveError),
    List(ListError),
}

#[derive(Serialize, Deserialize, Debug)]
struct Bucket {
    name: String,
    url: String,
    mainfests: usize,
    time: String,
}

impl BucketCommand {
    pub fn run(config: &Self, buckets_dir: &PathBuf) {
        if let Some(url) = &config.add {
            println!("{:?}", Self::add(url, buckets_dir));
        } else if let Some(bucket_name) = &config.rm {
            println!("{:?}", Self::remove(bucket_name, buckets_dir));
        } else if config.list {
            let json = Self::list(buckets_dir).unwrap();
            let buckets = json.get("buckets").unwrap();

            for bucket in buckets.as_array().unwrap() {
                println!("{}", json_to_table(bucket));
            }
        } else {
            println!("No command specified");
        }
    }

    fn add(url: &String, buckets_dir: &PathBuf) -> Result<(), AddError> {
        let bucket_name = url
            .rfind('/')
            .map(|idx| &url[idx + 1..])
            .ok_or(AddError::NameResolution)?;
        let bucket_name = bucket_name.to_lowercase();

        let path = buckets_dir.join(&bucket_name);

        if path.exists() {
            return Err(AddError::AlreadyExists);
        }

        // TODO: Shallow Clone
        let repo = Repository::clone_recurse(url, &path).map_err(|err| match err.class() {
            git2::ErrorClass::NoMemory => AddError::NoMem,
            git2::ErrorClass::Os => AddError::Os,
            git2::ErrorClass::Repository => AddError::BadRepoState,
            git2::ErrorClass::Index => AddError::InvalidIndex,
            git2::ErrorClass::Object => AddError::Object,
            git2::ErrorClass::Net => AddError::Net,
            git2::ErrorClass::Ssl => AddError::Ssl,
            git2::ErrorClass::Submodule => AddError::Submodule,
            git2::ErrorClass::Thread => AddError::Thread,
            git2::ErrorClass::FetchHead => AddError::FetchHead,
            git2::ErrorClass::Filesystem => AddError::Fs,
            git2::ErrorClass::Sha1 => AddError::Hash,
            git2::ErrorClass::Http => AddError::Http,
            _ => AddError::Unknown,
        })?;

        let head = repo.head().map_err(|_| AddError::FetchHead)?;
        let commit = head.peel_to_commit().map_err(|_| AddError::PeelCommit)?;
        let timestamp = commit.time();
        let naive_datetime = NaiveDateTime::from_timestamp_opt(timestamp.seconds(), 0)
            .ok_or(AddError::TimeConversion)?;
        let datetime = DateTime::<Local>::from_utc(naive_datetime, Local::now().offset().fix());
        let time = datetime.format("%Y-%m-%d %I:%M:%S %p").to_string();

        let path = path.join("bucket");
        let mainfests = fs::read_dir(path)
            .map_err(|_| AddError::Fs)?
            .par_bridge()
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

        let bucket = Bucket {
            name: bucket_name,
            url: url.to_owned(),
            mainfests,
            time,
        };

        write_bucket(bucket, &buckets_dir)?;

        Ok(())
    }

    fn remove(bucket_name: &String, buckets_dir: &PathBuf) -> Result<(), RemoveError> {
        let bucket_path = buckets_dir.join(bucket_name);

        remove_dir_all(&bucket_path).map_err(|err| match err.kind() {
            std::io::ErrorKind::NotFound => RemoveError::DirNotExist(bucket_path),
            std::io::ErrorKind::PermissionDenied => RemoveError::LackOfPermission,
            _ => RemoveError::Failed(bucket_path, Box::new(err)),
        })?;

        Ok(())
    }

    fn list(buckets_dir: &PathBuf) -> Result<Value, ListError> {
        let bucket_json = buckets_dir.join("buckets.json");

        let json = fs::read_to_string(&bucket_json).map_err(|e| match e.kind() {
            std::io::ErrorKind::NotFound => ListError::NotFound(bucket_json),
            std::io::ErrorKind::PermissionDenied => ListError::PermissionDenied,
            std::io::ErrorKind::InvalidInput => ListError::InvalidInput,
            std::io::ErrorKind::InvalidData => ListError::InvalidData,
            std::io::ErrorKind::Interrupted => ListError::Interrupted,
            std::io::ErrorKind::UnexpectedEof => ListError::UnexpectedEof,
            _ => ListError::Unknown,
        })?;

        let json: Value = serde_json::from_str(&json).map_err(|err| match err.classify() {
            serde_json::error::Category::Io => ListError::Io,
            serde_json::error::Category::Syntax => ListError::Syntax,
            serde_json::error::Category::Data => ListError::Data,
            serde_json::error::Category::Eof => ListError::UnexpectedEof,
        })?;

        Ok(json)
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct BucketData {
    buckets: Vec<Bucket>,
}

fn write_bucket(bucket: Bucket, buckets_dir: &PathBuf) -> Result<(), AddError> {
    let bucket_json = buckets_dir.join("buckets.json");

    let mut bucket_data = BucketData { buckets: vec![] };

    // Read existing data from the JSON file if it exists
    if let Ok(file_content) = fs::read_to_string(&bucket_json) {
        if let Ok(existing_data) = serde_json::from_str::<BucketData>(&file_content) {
            bucket_data.buckets = existing_data.buckets;
        }
    }

    bucket_data.buckets.push(bucket);

    let json = serde_json::to_string_pretty(&bucket_data).map_err(|_| AddError::Serialization)?;

    let mut file = OpenOptions::new()
        .write(true)
        .append(true)
        .create(true)
        .open(&bucket_json)
        .map_err(|_| AddError::OpenFile(bucket_json))?;

    writeln!(file, "{}", json).map_err(|_| AddError::Write)?;

    Ok(())
}
