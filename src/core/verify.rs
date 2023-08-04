use digest::{Digest, FixedOutput};
use md5::Md5;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::Value;
use sha1::Sha1;
use sha2::{Sha256, Sha512};

use std::{
    fs::File,
    io::{self, Read},
    path::PathBuf,
};

#[derive(Debug, Clone)]
pub enum Hash {
    SHA256(String),
    SHA512(String),
    SHA1(String),
    MD5(String),
}

impl Serialize for Hash {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Hash::SHA256(digest) => serializer.serialize_str(&format!("{}", digest)),
            Hash::SHA512(digest) => serializer.serialize_str(&format!("sha512:{}", digest)),
            Hash::SHA1(digest) => serializer.serialize_str(&format!("sha1:{}", digest)),
            Hash::MD5(digest) => serializer.serialize_str(&format!("md5:{}", digest)),
        }
    }
}

impl<'de> Deserialize<'de> for Hash {
    fn deserialize<D>(deserializer: D) -> Result<Hash, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: String = Deserialize::deserialize(deserializer)?;

        let s = s.trim_matches('"');

        let (hash_func, digest) = s.split_once(':').unwrap_or(("", s));
        let digest = digest.to_string();

        match hash_func {
            "sha512" => Ok(Hash::SHA512(digest)),
            "sha1" => Ok(Hash::SHA1(digest)),
            "md5" => Ok(Hash::MD5(digest)),
            _ => Ok(Hash::SHA256(digest)),
        }
    }
}

pub fn deserialize_hash<'de, D>(deserializer: D) -> Result<Option<Vec<Hash>>, D::Error>
where
    D: Deserializer<'de>,
{
    let value: Option<Value> = Deserialize::deserialize(deserializer)?;

    Ok(match value {
        Some(Value::Array(a)) => Some(
            a.iter()
                .map(|s| Hash::deserialize(s).map_err(serde::de::Error::custom))
                .collect::<Result<Vec<_>, _>>()?,
        ),
        Some(Value::String(s)) => Some(vec![
            Hash::deserialize(&Value::String(s)).map_err(serde::de::Error::custom)?
        ]),
        _ => None,
    })
}

impl Hash {
    pub fn verify(&self, file: &PathBuf) -> Result<bool, io::Error> {
        let mut file = File::open(file)?;
        let mut buff: Vec<u8> = Vec::new();
        file.read_to_end(&mut buff)?;

        fn calc_hash<D: Digest + FixedOutput>(data: &[u8]) -> String {
            let mut hasher = D::new();
            Digest::update(&mut hasher, data);
            let res = hasher.finalize_fixed();
            hex::encode(res)
        }

        let (expected_hash, original_hash) = match self {
            Hash::SHA256(x) => (x, calc_hash::<Sha256>(&buff)),
            Hash::SHA512(x) => (x, calc_hash::<Sha512>(&buff)),
            Hash::SHA1(x) => (x, calc_hash::<Sha1>(&buff)),
            Hash::MD5(x) => (x, calc_hash::<Md5>(&buff)),
        };

        Ok(expected_hash.to_lowercase() == original_hash.to_lowercase())
    }
}
