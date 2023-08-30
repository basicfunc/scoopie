use std::{fs::File, io::Read, path::PathBuf};

#[derive(Debug)]
pub enum FileKind {
    ZipArchive,
    SevenZipArchive,
    TarArchive,
    GZipArchive,
    LZMAArchive,
    LZHArchive,
    Other,
}

impl FileKind {
    pub fn infer(path: &PathBuf) -> Self {
        // println!("{:?}", path);

        // let kind = infer::get_from_path(path).unwrap();
        // println!("{:?}", kind);

        // match kind {
        //     Some(k) => match k.mime_type() {
        //         "application/zip" => Self::ZipArchive,
        //         "application/x-7z-compressed" => Self::SevenZipArchive,
        //         "application/x-tar" => Self::TarArchive,
        //         "application/gzip" => Self::GZipArchive,
        //         "application/x-xz" => Self::LZMAArchive,
        //         "application/x-lzip" => Self::LZMAArchive,
        //         _ => Self::Other,
        //     },
        //     None => Self::Other,
        // }

        Self::Other
    }
}
