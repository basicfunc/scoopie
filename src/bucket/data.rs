use rusqlite::Row;

use super::manifest::Manifest;

use std::{collections::HashMap, fmt, write};

#[derive(Default, Debug, Clone)]
pub struct Entry {
    pub app_name: String,
    pub manifest: Manifest,
}

impl TryFrom<&Row<'_>> for Entry {
    type Error = rusqlite::Error;

    fn try_from(value: &Row) -> Result<Self, Self::Error> {
        let app_name = value.get(0)?;
        let manifest: String = value.get(1)?;

        let manifest: Manifest = serde_json::from_str(&manifest).unwrap();

        Ok(Entry { app_name, manifest })
    }
}

#[derive(Default, Debug)]
pub struct BucketData(pub HashMap<String, Vec<Entry>>);

impl fmt::Display for BucketData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut output = String::new();
        self.0.iter().for_each(|(bucket_name, entries)| {
            entries.iter().for_each(|entry| {
                let app_name = &entry.app_name;
                let version = &entry.manifest.version;
                let description = &entry.manifest.description;
                output.push_str(&format!(
                    "\n{}/{}  {}\n  {}",
                    app_name, bucket_name, version, description
                ));
            })
        });

        write!(f, "{output}")
    }
}
