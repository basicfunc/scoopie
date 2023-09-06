use std::{
    fs::{metadata, File},
    io::{BufWriter, Read, Write},
    iter::zip,
};

use indicatif::{ProgressBar, ProgressStyle};
use regex_lite::Regex;
use url::Url;

use super::Hash;

use {
    crate::core::{buckets::*, config::*},
    crate::error::*,
    crate::utils::*,
};

const TEMPLATE: &'static str =
    " {spinner:.black} {wide_msg} |{bar:>.green}| ({percent:<}%, {binary_bytes_per_sec:<.black})";
const TICK_CHARS: &'static str = "⠁⠂⠄⡀⢀⠠⠐⠈ ";

#[derive(Debug)]
pub enum DownloadStatus {
    Downloaded(String),
    DownloadedAndVerified(String),
    AlreadyInCache(String),
}

pub struct Downloader;

impl Downloader {
    pub fn download(app_name: &str, verify: bool) -> Result<Vec<DownloadStatus>, ScoopieError> {
        let query = app_name.trim().to_lowercase();

        let (app_name, manifest) = match query.split_once('/') {
            Some((bucket, app)) => {
                let manifest = Buckets::query_app(&app)?
                    .get_app_from(&app, &bucket)
                    .ok_or(ScoopieError::NoAppFoundInBucket(app.into(), bucket.into()))?;

                (app, manifest)
            }
            None => {
                let app_name = &query;
                let manifest = Buckets::query_app(&app_name)?
                    .get_app(&query)
                    .ok_or(ScoopieError::NoAppFound(app_name.into()))?;

                (app_name.as_str(), manifest)
            }
        };

        let version = &manifest.version;
        let urls = manifest.url();
        let hashes = manifest.hash();

        zip(urls, hashes)
            .map(|(url, hash)| {
                let (pkg_name, file) = extract_names(&app_name, version, &url);
                let hash = if verify { Some(&hash) } else { None };
                dwnld(&pkg_name, url.as_str(), &file, hash)
            })
            .collect::<Result<Vec<_>, _>>()
    }
}

fn dwnld(
    pkg_name: &str,
    url: &str,
    file_name: &str,
    verify: Option<&Hash>,
) -> Result<DownloadStatus, ScoopieError> {
    let file_path = Config::cache_dir()?.join(file_name);

    let request = minreq::get(url);

    let mut response = request
        .send_lazy()
        .map_err(|_| ScoopieError::FailedToSendReq)?;

    if response.status_code >= 200 && response.status_code <= 299 {
        let total_size = match response.headers.get("content-length") {
            Some(size) => size.parse::<u64>().unwrap_or(0),
            None => 0,
        };

        let mut downloader = || {
            let mut file = BufWriter::new(
                File::create(&file_path)
                    .map_err(|_| ScoopieError::UnableToCreateFile(file_name.into()))?,
            );

            let mut chunk = [0; 4096];

            let style = ProgressStyle::with_template(TEMPLATE)
                .unwrap()
                .tick_chars(TICK_CHARS);

            let pb = ProgressBar::new(total_size);
            pb.set_style(style);
            pb.set_message(format!("Collecting {pkg_name}"));

            loop {
                let bytes_read = response
                    .read(&mut chunk)
                    .map_err(|_| ScoopieError::UnableToGetChunk(file_name.into()))?;

                if bytes_read == 0 {
                    break;
                }

                file.write_all(&chunk[..bytes_read])
                    .map_err(|_| ScoopieError::ChunkWrite(file_path.to_path_buf()))?;

                pb.inc(bytes_read as u64);
            }

            file.flush()
                .map_err(|_| ScoopieError::FlushFile(file_path.to_path_buf()))?;

            match verify {
                Some(hash) => match hash.verify(&file_path)? {
                    true => Ok(DownloadStatus::DownloadedAndVerified(file_name.into())),
                    false => Err(ScoopieError::WrongDigest(file_name.into())),
                },
                None => Ok(DownloadStatus::Downloaded(file_name.into())),
            }
        };

        if file_path.exists() {
            let file_metadata = metadata(&file_path)
                .map_err(|_| (ScoopieError::FailedToGetMetadata(file_path.to_path_buf())))?;

            match file_metadata.len().eq(&total_size) {
                true => Ok(DownloadStatus::AlreadyInCache(file_name.into())),
                false => {
                    file_path.rm()?;
                    downloader()
                }
            }
        } else {
            downloader()
        }
    } else {
        Err(ScoopieError::RequestFailed(
            file_name.into(),
            response.reason_phrase,
        ))
    }
}

fn extract_names(app_name: &str, version: &str, url: &Url) -> (String, String) {
    let pkg_name = match url.path_segments() {
        Some(segments) => segments.last().unwrap_or_default(),
        None => "",
    }
    .to_lowercase();

    let file_name = format!("{app_name}#{version}#{}", sanitize(url.path()));

    (pkg_name, file_name)
}

/// Sanitizes a given input string to make it safe for use as a filename across various operating systems.
///
/// This function takes an input string and performs the following operations:
///
/// 1. Replaces reserved characters and control characters with underscores (`_`) to ensure the resulting filename is valid.
///    Reserved characters and control characters include: `<`, `>`, `:`, `"`, `/`, `\`, `|`, `?`, `*`, and certain control characters (from `0x0000` to `0x001F`, `0x007F` to `0x009F`).
/// 2. Handles cases specific to Windows, where certain filenames are reserved for historical reasons. It checks if the resulting filename matches any of the reserved Windows filenames and appends an underscore if needed. Reserved Windows filenames include: `CON`, `PRN`, `AUX`, `NUL`, `COM1` through `COM9`, and `LPT1` through `LPT9`.
///
/// # Arguments
///
/// * `input` - A value that can be converted to a string (e.g., `&str`, `String`) representing the input filename.
///
/// # Returns
///
/// A sanitized string that can be safely used as a filename on various operating systems.
///
/// # Platform-Specific Considerations
///
/// - On Windows, this function performs additional checks for reserved filenames, ensuring compliance with historical constraints.
///
/// # Examples
///
/// ```rust
/// let input = "my<file>:name.txt";
/// let sanitized = sanitize(input);
/// assert_eq!(sanitized, "my_file_name.txt");
/// ```
///
/// ```rust
/// // On Windows, this will append an underscore to match the reserved filename "CON".
/// let input = "CON";
/// let sanitized = sanitize(input);
/// assert_eq!(sanitized, "CON_");
/// ```
///
/// # Note
///
/// This function provides a best-effort approach to sanitizing filenames and does not guarantee uniqueness. Filenames should still be used with caution, and additional measures may be necessary to ensure uniqueness in a specific context.
///
/// # References
///
/// For the most up-to-date and detailed information about reserved file name patterns, consult the official documentation for the specific operating system and file system you are working with.
///
/// - **Windows:** Microsoft's documentation, including MSDN and official Windows documentation, often provides information on reserved file names and characters.
/// - **Linux and Unix-like systems:** Documentation for the specific file system you are using (e.g., ext4, XFS) and the Linux Filesystem Hierarchy Standard (FHS) can provide insights into file naming conventions.
/// - **macOS:** macOS follows Unix conventions, so you can refer to Unix and macOS documentation for guidance.
///
/// # Safety
///
/// This function should be safe to use in typical scenarios. However, improper use or reliance on this function as the sole means of ensuring filename safety may still result in issues in specific edge cases or unusual contexts.
///
/// Always consider the specific requirements and constraints of your application when working with filenames.
///
fn sanitize<T: AsRef<str>>(input: T) -> String {
    const REPLACEMENT: &str = "_";

    // Create regex patterns
    let reserved_pattern =
        Regex::new("[<>:\"/\\\\|?*\u{0000}-\u{001F}\u{007F}\u{0080}-\u{009F}]+").unwrap();
    let outer_periods_pattern = Regex::new("^\\.+|\\.+$").unwrap();

    // Apply regex replacements and conversions
    let result = reserved_pattern.replace_all(input.as_ref(), REPLACEMENT);
    let result = outer_periods_pattern
        .replace_all(&result, REPLACEMENT)
        .to_string();

    // Windows-specific checks to match any of the reserved Windows filenames
    #[cfg(windows)]
    {
        let windows_reserved_pattern = Regex::new("^(con|prn|aux|nul|com\\d|lpt\\d)$").unwrap();
        if windows_reserved_pattern.is_match(result.as_str()) {
            return result + REPLACEMENT;
        }
    }

    result
}
