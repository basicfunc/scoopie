use std::path::PathBuf;

use rusqlite::Connection;

use crate::{
    bucket::Bucket,
    config::*,
    error::{DatabaseError, ScoopieError},
};

const TABLE_CREATE_STMT: &'static str = "CREATE TABLE IF NOT EXISTS manifests (
                 app_name TEXT NOT NULL PRIMARY KEY,
                 manifest TEXT
             )";

const FTS_TABLE_CREATE_STMT: &'static str =
    "CREATE VIRTUAL TABLE IF NOT EXISTS manifests_fts USING FTS5(app_name, manifest)";

const INSERT_STMT: &'static str = "INSERT INTO manifests (app_name, manifest) VALUES (?, ?)";

const FTS_INSERT_STMT: &'static str =
    "INSERT INTO manifests_fts(app_name, manifest) SELECT app_name, manifest FROM manifests";

#[derive(Debug, PartialEq, PartialOrd)]
pub enum DatabaseState {
    UpToDate,
    Created,
}

#[derive(Debug)]
pub struct Database {
    name: String,
    path: PathBuf,
    status: DatabaseState,
}

impl Database {
    pub fn create(bucket: &Bucket) -> Result<Database, ScoopieError> {
        let name = &bucket.id.split("-").next().unwrap_or_default();
        let name = name.to_string();

        let mut repo = PathBuf::from(&bucket.id);
        repo.set_extension("db");

        let db = Config::repos_dir()?.join(&repo);

        if db.exists() {
            return Ok(Database {
                name,
                path: db,
                status: DatabaseState::UpToDate,
            });
        }

        let conn = Connection::open(&db)
            .map_err(|_| ScoopieError::Database(DatabaseError::UnableToOpen))?;

        conn.execute(TABLE_CREATE_STMT, [])
            .map_err(|_| ScoopieError::Database(DatabaseError::FailedToCreateTable))?;

        let mut stmt = conn
            .prepare(INSERT_STMT)
            .map_err(|_| ScoopieError::Database(DatabaseError::FailedToMkStmt))?;

        for manifest in &bucket.manifests {
            stmt.execute(&[&manifest.0, &manifest.1])
                .map_err(|_| ScoopieError::Database(DatabaseError::FailedInsertion))?;
        }

        conn.execute(FTS_TABLE_CREATE_STMT, [])
            .map_err(|_| ScoopieError::Database(DatabaseError::FailedToCreateTable))?;

        conn.execute(FTS_INSERT_STMT, [])
            .map_err(|_| ScoopieError::Database(DatabaseError::FailedInsertion))?;

        Ok(Database {
            name,
            path: db,
            status: DatabaseState::Created,
        })
    }
}
