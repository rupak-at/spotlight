use crate::{model::Entry, Result};
use rusqlite::{params, Connection};
use std::path::Path;

fn connect(path: &Path) -> Result<Connection> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let db = Connection::open(path).map_err(|e| e.to_string())?;
    db.busy_timeout(std::time::Duration::from_secs(3))
        .map_err(|e| e.to_string())?;
    db.execute_batch("CREATE TABLE IF NOT EXISTS metadata (key TEXT PRIMARY KEY, value TEXT NOT NULL); CREATE TABLE IF NOT EXISTS entries (id TEXT PRIMARY KEY, payload TEXT NOT NULL);").map_err(|e| e.to_string())?;
    Ok(db)
}

pub fn load(path: &Path, key: &str) -> Result<Vec<Entry>> {
    let db = connect(path)?;
    let saved: Option<String> = db
        .query_row(
            "SELECT value FROM metadata WHERE key = 'settings'",
            [],
            |row| row.get(0),
        )
        .optional()
        .map_err(|e| e.to_string())?;
    if saved.as_deref() != Some(key) {
        return Ok(Vec::new());
    }
    let mut stmt = db
        .prepare("SELECT payload FROM entries ORDER BY id")
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([], |row| row.get::<_, String>(0))
        .map_err(|e| e.to_string())?;
    rows.map(|row| {
        serde_json::from_str(&row.map_err(|e| e.to_string())?).map_err(|e| e.to_string())
    })
    .collect()
}

pub fn replace(path: &Path, key: &str, entries: &[Entry]) -> Result<()> {
    let mut db = connect(path)?;
    let tx = db.transaction().map_err(|e| e.to_string())?;
    tx.execute("DELETE FROM entries", [])
        .map_err(|e| e.to_string())?;
    {
        let mut stmt = tx
            .prepare("INSERT INTO entries (id, payload) VALUES (?1, ?2)")
            .map_err(|e| e.to_string())?;
        for entry in entries {
            stmt.execute(params![
                entry.id,
                serde_json::to_string(entry).map_err(|e| e.to_string())?
            ])
            .map_err(|e| e.to_string())?;
        }
    }
    tx.execute(
        "INSERT OR REPLACE INTO metadata (key, value) VALUES ('settings', ?1)",
        [key],
    )
    .map_err(|e| e.to_string())?;
    tx.commit().map_err(|e| e.to_string())
}

use rusqlite::OptionalExtension;
