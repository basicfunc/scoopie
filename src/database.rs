use std::path::PathBuf;

use rusqlite::Connection;

use crate::{
    bucket::Bucket,
    config::Config,
    error::{DatabaseError, ScoopieError},
};

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
    pub fn create(bucket: Bucket) -> Result<Database, ScoopieError> {
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

        conn.execute(
            "CREATE TABLE IF NOT EXISTS mainfests (
                 app_name TEXT NOT NULL PRIMARY KEY,
                 mainfest TEXT
             )",
            [],
        )
        .map_err(|_| ScoopieError::Database(DatabaseError::FailedToCreateTable))?;

        let mut stmt = conn
            .prepare("INSERT INTO mainfests (app_name, mainfest) VALUES (?, ?)")
            .map_err(|_| ScoopieError::Database(DatabaseError::FailedToMkStmt))?;

        for mainfest in bucket.mainfests {
            stmt.execute(&[&mainfest.app_name, &mainfest.mainfest])
                .map_err(|_| ScoopieError::Database(DatabaseError::FailedInsertion))?;
        }

        conn.execute(
            "CREATE VIRTUAL TABLE IF NOT EXISTS mainfests_fts USING FTS5(app_name, mainfest)",
            [],
        )
        .map_err(|_| ScoopieError::Database(DatabaseError::FailedToCreateTable))?;

        conn.execute(
            "INSERT INTO mainfests_fts(app_name, mainfest) SELECT app_name, mainfest FROM mainfests",
            [],
        )
        .map_err(|_| ScoopieError::Database(DatabaseError::FailedInsertion))?;

        Ok(Database {
            name,
            path: db,
            status: DatabaseState::Created,
        })
    }
}
