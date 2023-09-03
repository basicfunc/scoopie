use std::collections::HashMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Deserializer, Serialize};
use serde_json::{json, Value};
use url::Url;

use crate::core::config::*;
use crate::core::download::{deserialize_hash, Hash};
use crate::error::ScoopieError;

#[derive(Clone, Deserialize, Debug, Serialize)]
/// This strictly follows Scoop's convention for app manifests, which could be found at: https://github.com/ScoopInstaller/Scoop/wiki/App-Manifests
pub struct Manifest {
    // Required Properties
    pub version: String,
    pub description: String,
    pub homepage: String,
    pub license: Value,
    // Optional Properties
    bin: Option<Value>,
    extract_dir: Option<Value>,
    #[serde(rename = "##")]
    comments: Option<Value>,
    architecture: Option<Architecture>,
    autoupdate: Option<Value>, // It is used by scoop to check for autoupdates, currrently out-of-scope for Scoopie.
    checkver: Option<Value>, // It is used by scoop to check for updated versions, currrently out-of-scope for Scoopie.
    depends: Option<Value>,
    suggest: Option<Value>,
    env_add_path: Option<Value>,
    env_set: Option<HashMap<String, String>>,
    extract_to: Option<Value>,
    #[serde(default, deserialize_with = "deserialize_hash")]
    hash: Option<Vec<Hash>>,
    innosetup: Option<bool>,
    installer: Option<Value>, // TODO: implement it as individual struct so that it contains related properties.
    notes: Option<Value>,
    persist: Option<Value>,
    post_install: Option<Value>,
    post_uninstall: Option<Value>,
    pre_install: Option<Value>,
    pre_uninstall: Option<Value>,
    psmodule: Option<HashMap<String, String>>,
    shortcuts: Option<Vec<Vec<String>>>,
    uninstaller: Option<Value>, // TODO: Same options as installer, but the file/script is run to uninstall the application.
    #[serde(default, deserialize_with = "deserialize_url")]
    url: Option<Vec<Url>>,
    // Undocumented Properties
    cookie: Option<Value>,
    // Deprecated Properties
    _comment: Option<Vec<String>>,
    msi: Option<String>,
}

fn deserialize_url<'de, D>(deserializer: D) -> Result<Option<Vec<Url>>, D::Error>
where
    D: Deserializer<'de>,
{
    let value: Option<Value> = Deserialize::deserialize(deserializer)?;

    match value {
        Some(Value::String(s)) => Ok(Some(
            vec![Url::parse(&s).map_err(serde::de::Error::custom)?],
        )),
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

impl ToString for Manifest {
    fn to_string(&self) -> String {
        json!(self).to_string()
    }
}

impl TryFrom<PathBuf> for Manifest {
    type Error = ScoopieError;

    fn try_from(value: PathBuf) -> Result<Self, Self::Error> {
        let buff =
            std::fs::read_to_string(&value).map_err(|_| ScoopieError::FailedToReadFile(value))?;

        serde_json::from_str::<Manifest>(&buff).map_err(|_| ScoopieError::InvalidManifestInBucket)
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
struct Architecture {
    #[serde(rename = "64bit")]
    bit_64: Option<Attrs>,
    #[serde(rename = "32bit")]
    bit_32: Option<Attrs>,
    arm64: Option<Attrs>,
}

impl Architecture {
    fn get(&self) -> Attrs {
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
struct Attrs {
    #[serde(default, deserialize_with = "deserialize_url")]
    url: Option<Vec<Url>>,
    #[serde(default, deserialize_with = "deserialize_hash")]
    hash: Option<Vec<Hash>>,
    extract_dir: Option<Value>,
    bin: Option<Value>,
    shortcuts: Option<Value>,
    env_add_path: Option<Value>,
}

impl Attrs {
    fn url(self) -> Vec<Url> {
        self.url.unwrap_or_default()
    }

    fn hash(self) -> Vec<Hash> {
        self.hash.unwrap_or_default()
    }
}
