use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::error::*;

#[derive(Clone, Deserialize, Debug, Default, Serialize)]
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
    pub hash: Option<Value>,
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
    pub url: Option<Value>,
    // Undocumented Properties
    pub cookie: Option<Value>,
    // Deprecated Properties
    pub _comment: Option<Vec<String>>,
    pub msi: Option<String>,
}

impl TryInto<String> for Manifest {
    type Error = ScoopieError;

    fn try_into(self) -> Result<String, Self::Error> {
        serde_json::to_string(&self).map_err(|_| ScoopieError::Bucket(BucketError::InvalidManifest))
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

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct Links {
    pub url: Option<Value>,
    pub hash: Option<Value>,
    pub extract_dir: Option<Value>,
    pub bin: Option<Value>,
    pub shortcuts: Option<Value>,
    pub env_add_path: Option<Value>,
}
