use std::{
    collections::HashMap,
    fs::{self, OpenOptions},
    io::Write,
};

use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::{core::config::*, error::*};

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct MetaData(HashMap<String, MetaDataEntry>);

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct MetaDataEntry {
    source: String,
    pub commit_id: String,
}

impl MetaData {
    pub fn read() -> Result<Self, ScoopieError> {
        let metadata_path = Config::buckets_dir()?.join("metadata.json");
        let content = fs::read_to_string(metadata_path).unwrap();
        Ok(serde_json::from_str(&content).unwrap())
    }

    pub fn write(&mut self, name: &str, url: &str, commit_id: &str) -> Result<(), ScoopieError> {
        self.0.insert(
            name.into(),
            MetaDataEntry {
                source: url.into(),
                commit_id: commit_id.into(),
            },
        );

        let metadata_path = Config::buckets_dir()?.join("metadata.json");
        let mut file = OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(metadata_path)
            .unwrap();

        let content = json!(self.0).to_string();

        file.write_all(content.as_bytes()).unwrap();

        Ok(())
    }

    pub fn get(&self, name: &str) -> MetaDataEntry {
        self.0.get(name).cloned().unwrap_or_default()
    }
}

pub fn write_default_metadata() -> Result<(), ScoopieError> {
    let metadata_path = Config::buckets_dir()?.join("metadata.json");
    let mut file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(metadata_path)
        .unwrap();

    let content = json!(MetaData::default()).to_string();

    file.write_all(content.as_bytes()).unwrap();

    Ok(())
}
