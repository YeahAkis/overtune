use std::fs;
use std::io;
use std::path::Path;
use std::time::Duration;

use rusqlite::Connection;
use thiserror::Error;

use crate::schema::{OVERTUNE_TABLES, VITUNE_TABLES};

const SCHEMA_VERSION: i64 = 1;

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    Sqlite(#[from] rusqlite::Error),
    #[error(transparent)]
    Io(#[from] io::Error),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    #[error("database schema {0} is newer than this version of overtune")]
    TooNew(i64),
}

pub type Result<T> = std::result::Result<T, Error>;

pub struct Library {
    pub(crate) conn: Connection,
}

impl Library {
    pub fn open(path: &Path) -> Result<Library> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let conn = Connection::open(path)?;
        conn.pragma_update(None, "journal_mode", "WAL")?;
        Library::init(conn)
    }

    pub fn in_memory() -> Result<Library> {
        Library::init(Connection::open_in_memory()?)
    }

    fn init(conn: Connection) -> Result<Library> {
        conn.pragma_update(None, "foreign_keys", true)?;
        conn.busy_timeout(Duration::from_secs(5))?;
        let lib = Library { conn };
        lib.migrate()?;
        Ok(lib)
    }

    fn migrate(&self) -> Result<()> {
        let version: i64 = self
            .conn
            .pragma_query_value(None, "user_version", |r| r.get(0))?;
        if version > SCHEMA_VERSION {
            return Err(Error::TooNew(version));
        }
        if version < 1 {
            let tx = self.conn.unchecked_transaction()?;
            for sql in VITUNE_TABLES.iter().chain(OVERTUNE_TABLES) {
                tx.execute(sql, [])?;
            }
            tx.pragma_update(None, "user_version", SCHEMA_VERSION)?;
            tx.commit()?;
        }
        Ok(())
    }

    pub fn table_columns(&self, table: &str) -> Result<Vec<(String, String, bool)>> {
        let mut stmt = self
            .conn
            .prepare("SELECT name, type, \"notnull\" FROM pragma_table_info(?1) ORDER BY cid")?;
        let rows = stmt.query_map([table], |r| {
            Ok((r.get(0)?, r.get(1)?, r.get::<_, i64>(2)? != 0))
        })?;
        Ok(rows.collect::<rusqlite::Result<_>>()?)
    }
}
