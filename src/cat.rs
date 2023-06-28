use std::{
    fs::File,
    io::{self, BufRead, BufReader, Write},
    write,
};

use argh::FromArgs;

#[derive(FromArgs, PartialEq, Debug)]
/// Show content of specified manifest
#[argh(subcommand, name = "cat")]
pub struct CatCommand {
    #[argh(positional)]
    app: String,
}

#[derive(Debug)]
pub enum CatError {
    Read(ReadError),
    Write(WriteError),
}

impl std::error::Error for CatError {}

impl std::fmt::Display for CatError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Write(err) => write!(f, "{err} while writing."),
            Self::Read(err) => write!(f, "{err} while reading."),
        }
    }
}

#[derive(Debug)]
pub enum ReadError {
    UnknownError,
    FileNotFound,
    PermissionDenied,
    NonUTF8,
    TimeOut,
    UnexpectedEof,
    OutOfMemory,
}

impl std::fmt::Display for ReadError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ReadError::UnknownError => write!(f, "Unknown error occured"),
            ReadError::FileNotFound => write!(f, "File not found"),
            ReadError::PermissionDenied => write!(f, "Permission denied"),
            ReadError::NonUTF8 => write!(f, "Enoding error occured"),
            ReadError::TimeOut => write!(f, "Timeout"),
            ReadError::UnexpectedEof => write!(f, "Early EOF found"),
            ReadError::OutOfMemory => write!(f, "Ran out of memory"),
        }
    }
}

#[derive(Debug)]
pub enum WriteError {
    InvalidInput,
    PermissionDenied,
    OutOfMemory,
    NonUTF8,
    Interrupted,
    UnexpectedEof,
    UnknownError,
}

impl std::fmt::Display for WriteError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            WriteError::InvalidInput => write!(f, "Invalid input found"),
            WriteError::PermissionDenied => write!(f, "Permission denied"),
            WriteError::OutOfMemory => write!(f, "Ran out of memory"),
            WriteError::NonUTF8 => write!(f, "Enoding error occured"),
            WriteError::Interrupted => write!(f, "Interrupted"),
            WriteError::UnexpectedEof => write!(f, "Unexpected EOF found"),
            WriteError::UnknownError => write!(f, "Unknown error occured"),
        }
    }
}

impl CatCommand {
    pub fn run(config: &CatCommand) -> Result<(), CatError> {
        let file = File::open(&config.app).map_err(|err| match err.kind() {
            io::ErrorKind::NotFound => CatError::Read(ReadError::FileNotFound),
            io::ErrorKind::PermissionDenied => CatError::Read(ReadError::PermissionDenied),
            io::ErrorKind::InvalidData => CatError::Read(ReadError::NonUTF8),
            io::ErrorKind::TimedOut => CatError::Read(ReadError::TimeOut),
            io::ErrorKind::UnexpectedEof => CatError::Read(ReadError::UnexpectedEof),
            io::ErrorKind::OutOfMemory => CatError::Read(ReadError::OutOfMemory),
            _ => CatError::Read(ReadError::UnknownError),
        })?;

        let reader = BufReader::new(file);

        for line in reader.lines() {
            let line = line.map_err(|_| CatError::Read(ReadError::UnknownError))?;
            writeln!(io::stdout(), "{}", line).map_err(|err| match err.kind() {
                io::ErrorKind::PermissionDenied => CatError::Write(WriteError::PermissionDenied),
                io::ErrorKind::InvalidInput => CatError::Write(WriteError::InvalidInput),
                io::ErrorKind::InvalidData => CatError::Write(WriteError::NonUTF8),
                io::ErrorKind::Interrupted => CatError::Write(WriteError::Interrupted),
                io::ErrorKind::UnexpectedEof => CatError::Write(WriteError::UnexpectedEof),
                io::ErrorKind::OutOfMemory => CatError::Write(WriteError::OutOfMemory),
                _ => CatError::Write(WriteError::UnknownError),
            })?;
        }

        Ok(())
    }
}
