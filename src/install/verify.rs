use digest::{Digest, FixedOutput};
use md5::Md5;
use sha1::Sha1;
use sha2::{Sha256, Sha512};

use std::{
    fs::File,
    io::{self, Read},
    path::PathBuf,
};

#[derive(Debug)]
pub enum Hash {
    SHA256(String),
    SHA512(String),
    SHA1(String),
    MD5(String),
}

impl From<String> for Hash {
    fn from(value: String) -> Self {
        let value = value.trim_matches('"');
        println!("{}", value);

        let (hash_func, digest) = value.split_once(":").unwrap_or(("", value));
        let digest = digest.to_string();

        match hash_func {
            "sha512" => Self::SHA512(digest),
            "sha1" => Self::SHA1(digest),
            "md5" => Self::MD5(digest),
            _ => Self::SHA256(digest),
        }
    }
}

impl Hash {
    pub fn verify(&self, file: &PathBuf) -> Result<bool, io::Error> {
        let mut file = File::open(file)?;
        let mut buff: Vec<u8> = Vec::new();
        file.read_to_end(&mut buff)?;

        let expected_hash = match self {
            Hash::SHA256(x) => x,
            Hash::SHA512(x) => x,
            Hash::SHA1(x) => x,
            Hash::MD5(x) => x,
        }
        .to_lowercase();

        let hash = match self {
            Hash::SHA256(_) => calc_hash::<Sha256>(&buff),
            Hash::SHA512(_) => calc_hash::<Sha512>(&buff),
            Hash::SHA1(_) => calc_hash::<Sha1>(&buff),
            Hash::MD5(_) => calc_hash::<Md5>(&buff),
        }
        .to_lowercase();

        Ok(expected_hash == hash)
    }
}

fn calc_hash<D: Digest + FixedOutput>(data: &[u8]) -> String {
    let mut hasher = D::new();
    Digest::update(&mut hasher, data);
    let res = hasher.finalize_fixed();
    hex::encode(res)
}
