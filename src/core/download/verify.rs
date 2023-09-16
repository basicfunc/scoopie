use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::Value;

use crate::error::ScoopieError;

use std::{fs::File, io::Read, path::PathBuf};

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
        let hash = {
            let hash: String = Deserialize::deserialize(deserializer)?;
            hash.trim_matches('"').to_owned()
        };

        match hash.split_once(':') {
            Some(("sha512", digest)) => Ok(Hash::SHA512(digest.to_lowercase())),
            Some(("sha1", digest)) => Ok(Hash::SHA1(digest.to_lowercase())),
            Some(("md5", digest)) => Ok(Hash::MD5(digest.to_lowercase())),
            Some((func, _)) => Err(serde::de::Error::custom(format!(
                "unsupported digest function: {func}"
            ))),
            None => Ok(Hash::SHA256(hash)),
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
    pub fn verify(&self, path: &PathBuf) -> Result<bool, ScoopieError> {
        let mut file =
            File::open(path).map_err(|_| ScoopieError::FailedToOpenFile(path.to_path_buf()))?;
        let mut buff: Vec<u8> = Vec::new();
        file.read_to_end(&mut buff)
            .map_err(|_| ScoopieError::FailedToReadFile(path.to_path_buf()))?;

        let (expected_hash, computed_hash) = match self {
            Hash::SHA256(hash) => {
                use sha2::{Digest, Sha256};
                (
                    hash.to_lowercase(),
                    hex::encode(Sha256::digest(buff)).to_lowercase(),
                )
            }
            Hash::SHA512(hash) => {
                use sha2::{Digest, Sha512};

                (
                    hash.to_lowercase(),
                    hex::encode(Sha512::digest(buff)).to_lowercase(),
                )
            }
            Hash::SHA1(hash) => {
                use sha1::{Digest, Sha1};

                (
                    hash.to_lowercase(),
                    hex::encode(Sha1::digest(buff)).to_lowercase(),
                )
            }
            Hash::MD5(hash) => {
                use md5::{Digest, Md5};

                (
                    hash.to_lowercase(),
                    hex::encode(Md5::digest(buff)).to_lowercase(),
                )
            }
        };

        Ok(expected_hash == computed_hash)
    }
}
