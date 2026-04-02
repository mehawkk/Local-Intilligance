pub mod models;

use rusqlite::{Connection, Result};
use std::path::Path;

/// Open or create the SQLite database and run migrations.
pub fn init_db(db_path: &Path) -> Result<Connection> {
    let conn = Connection::open(db_path)?;

    // Performance pragmas - all may return rows in newer SQLite, so use query_row
    let _: String = conn.query_row("PRAGMA journal_mode=WAL", [], |row| row.get(0))
        .unwrap_or_else(|_| "wal".to_string());
    let _ = conn.query_row("PRAGMA synchronous=NORMAL", [], |row| row.get::<_, i64>(0)).ok();
    let _ = conn.query_row("PRAGMA foreign_keys=ON", [], |row| row.get::<_, i64>(0)).ok();
    let _ = conn.query_row("PRAGMA cache_size=-8000", [], |row| row.get::<_, i64>(0)).ok();
    let _ = conn.query_row("PRAGMA busy_timeout=5000", [], |row| row.get::<_, i64>(0)).ok();
    // Use memory for temp tables instead of disk
    let _ = conn.query_row("PRAGMA temp_store=MEMORY", [], |row| row.get::<_, i64>(0)).ok();
    // Memory-mapped I/O: 256MB for faster reads
    let _ = conn.query_row("PRAGMA mmap_size=268435456", [], |row| row.get::<_, i64>(0)).ok();

    run_migrations(&conn)?;
    Ok(conn)
}

fn run_migrations(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS indexed_roots (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            path TEXT NOT NULL UNIQUE,
            created_at TEXT NOT NULL DEFAULT (datetime('now'))
        )",
        [],
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS excluded_paths (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            pattern TEXT NOT NULL UNIQUE,
            created_at TEXT NOT NULL DEFAULT (datetime('now'))
        )",
        [],
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS files (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            root_id INTEGER NOT NULL REFERENCES indexed_roots(id) ON DELETE CASCADE,
            path TEXT NOT NULL UNIQUE,
            filename TEXT NOT NULL,
            extension TEXT,
            size_bytes INTEGER NOT NULL DEFAULT 0,
            created_at_fs TEXT,
            modified_at_fs TEXT,
            fingerprint TEXT,
            parser_type TEXT NOT NULL DEFAULT 'none',
            parser_status TEXT NOT NULL DEFAULT 'pending',
            is_deleted INTEGER NOT NULL DEFAULT 0,
            last_indexed_at TEXT,
            last_seen_at TEXT
        )",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_files_path ON files(path)",
        [],
    )?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_files_root_id ON files(root_id)",
        [],
    )?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_files_fingerprint ON files(fingerprint)",
        [],
    )?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_files_deleted ON files(is_deleted)",
        [],
    )?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_files_extension ON files(extension)",
        [],
    )?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_files_root_deleted ON files(root_id, is_deleted)",
        [],
    )?;

    // FTS5 virtual table - create only if it doesn't exist.
    let fts_exists: bool = conn
        .prepare("SELECT 1 FROM sqlite_master WHERE type='table' AND name='files_fts'")
        .and_then(|mut stmt| stmt.exists([]))
        .unwrap_or(false);

    if !fts_exists {
        conn.execute_batch(
            "CREATE VIRTUAL TABLE files_fts USING fts5(
                filename,
                path,
                content,
                file_id UNINDEXED
            );",
        )?;
    }

    // Insert default exclusion patterns if table is empty
    let count: i64 = conn.query_row("SELECT COUNT(*) FROM excluded_paths", [], |row| {
        row.get(0)
    })?;

    if count == 0 {
        let defaults = [
            "node_modules",
            ".git",
            "__pycache__",
            ".svn",
            "target",
            "dist",
            ".cache",
            ".tmp",
            "AppData\\Local\\Temp",
            "Windows\\System32",
            "Windows\\SysWOW64",
            "$Recycle.Bin",
            ".DS_Store",
        ];
        let mut stmt =
            conn.prepare("INSERT OR IGNORE INTO excluded_paths (pattern) VALUES (?1)")?;
        for pattern in &defaults {
            stmt.execute([pattern])?;
        }
    }

    Ok(())
}
