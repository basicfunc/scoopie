use std::collections::HashMap;
use std::vec;

use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;
use url::Url;

use crate::error::*;

use super::config::*;
use super::verify::{deserialize_hash, Hash};

#[derive(Clone, Deserialize, Debug, Serialize)]
/// This strictly follows Scoop's convention for app manifests, which could be found at: https://github.com/ScoopInstaller/Scoop/wiki/App-Manifests
pub struct Manifest {
    // Required Properties
    pub version: String,
    pub description: String,
    pub homepage: String,
    pub license: Value,
    // Optional Properties
    pub bin: Option<Value>,
    pub extract_dir: Option<Value>,
    #[serde(rename = "##")]
    pub comments: Option<Value>,
    pub architecture: Option<Architecture>,
    pub autoupdate: Option<Value>, // It is used by scoop to check for autoupdates, currrently out-of-scope for Scoopie.
    pub checkver: Option<Value>,   // It is used by scoop to check for updated versions, currrently out-of-scope for Scoopie.
    pub depends: Option<Value>,
    pub suggest: Option<Value>,
    pub env_add_path: Option<Value>,
    pub env_set: Option<HashMap<String, String>>,
    pub extract_to: Option<Value>,
    #[serde(default, deserialize_with = "deserialize_hash")]
    pub hash: Option<Vec<Hash>>,
    pub innosetup: Option<bool>,
    pub installer: Option<Value>, // TODO: implement it as individual struct so that it contains related properties.
    pub notes: Option<Value>,
    pub persist: Option<Value>,
    pub post_install: Option<Value>,
    pub post_uninstall: Option<Value>,
    pub pre_install: Option<Value>,
    pub pre_uninstall: Option<Value>,
    pub psmodule: Option<HashMap<String, String>>,
    pub shortcuts: Option<Vec<Vec<String>>>,
    pub uninstaller: Option<Value>, // TODO: Same options as installer, but the file/script is run to uninstall the application.
    #[serde(default, deserialize_with = "deserialize_url")]
    pub url: Option<Vec<Url>>,
    // Undocumented Properties
    pub cookie: Option<Value>,
    // Deprecated Properties
    pub _comment: Option<Vec<String>>,
    pub msi: Option<String>,
}

fn deserialize_url<'de, D>(deserializer: D) -> Result<Option<Vec<Url>>, D::Error>
where
    D: Deserializer<'de>,
{
    let value: Option<Value> = Deserialize::deserialize(deserializer)?;

    match value {
        Some(Value::String(s)) => Ok(Some(vec![Url::parse(&s).map_err(serde::de::Error::custom)?])),
        Some(Value::Array(arr)) => arr
            .iter()
            .map(|url| match url {
                Value::String(s) => Url::parse(&s).map_err(serde::de::Error::custom),
                _ => Err(serde::de::Error::custom("Invalid Url Format")),
            })
            .collect::<Result<Vec<_>, _>>()
            .map(Some),
        _ => Ok(None),
    }
}

impl TryInto<String> for Manifest {
    type Error = ScoopieError;

    fn try_into(self) -> Result<String, Self::Error> {
        serde_json::to_string(&self).map_err(|_| ScoopieError::Bucket(BucketError::InvalidManifest))
    }
}

impl Manifest {
    pub fn url(&self) -> Vec<Url> {
        match &self.architecture {
            Some(arch) => arch.get().url(),
            None => self.url.clone().unwrap_or_default(),
        }
    }

    pub fn hash(&self) -> Vec<Hash> {
        match &self.architecture {
            Some(arch) => arch.get().hash(),
            None => self.hash.clone().unwrap_or_default(),
        }
    }
}

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct Architecture {
    #[serde(rename = "64bit")]
    pub bit_64: Option<Links>,
    #[serde(rename = "32bit")]
    pub bit_32: Option<Links>,
    pub arm64: Option<Links>,
}

impl Architecture {
    fn get(&self) -> Links {
        let arch = Config::arch().unwrap();

        match arch {
            Arch::Bit64 => &self.bit_64,
            Arch::Bit32 => &self.bit_32,
            Arch::Arm64 => &self.arm64,
        }
        .clone()
        .unwrap_or_default()
    }
}

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct Links {
    #[serde(default, deserialize_with = "deserialize_url")]
    pub url: Option<Vec<Url>>,
    #[serde(default, deserialize_with = "deserialize_hash")]
    pub hash: Option<Vec<Hash>>,
    pub extract_dir: Option<Value>,
    pub bin: Option<Value>,
    pub shortcuts: Option<Value>,
    pub env_add_path: Option<Value>,
}

impl Links {
    fn url(self) -> Vec<Url> {
        self.url.unwrap_or_default()
    }

    fn hash(self) -> Vec<Hash> {
        self.hash.unwrap_or_default()
    }
}
