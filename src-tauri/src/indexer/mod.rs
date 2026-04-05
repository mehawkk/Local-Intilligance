pub mod fingerprint;

use crate::config;
use crate::crawler;
use crate::db::models::IndexStatus;
use crate::parser;
use rusqlite::Connection;
use std::path::Path;
use std::sync::Mutex;
use tauri::Emitter;

/// Run a full scan of the given root path, indexing all files.
/// Updates index_status as it progresses and emits Tauri events.
pub fn run_full_scan(
    conn: &Mutex<Connection>,
    index_status: &Mutex<IndexStatus>,
    app_handle: &tauri::AppHandle,
    root_id: i64,
    root_path: &str,
    exclusions: &[String],
) {
    let scan_started_at = chrono::Utc::now()
        .format("%Y-%m-%d %H:%M:%S")
        .to_string();

    // Set status to running
    {
        let mut status = index_status.lock().unwrap();
        status.is_running = true;
        status.total_files = 0;
        status.processed_files = 0;
        status.failed_files = 0;
        status.current_file = None;
        status.errors.clear();
    }

    let _ = app_handle.emit("index-progress", get_status_snapshot(index_status));

    // Crawl the directory
    let files = crawler::crawl_directory(Path::new(root_path), exclusions);

    {
        let mut status = index_status.lock().unwrap();
        status.total_files = files.len() as u64;
    }

    let _ = app_handle.emit("index-progress", get_status_snapshot(index_status));

    // Process files in batches
    let batch_size = 100;
    for chunk in files.chunks(batch_size) {
        // Track batch-level counters to minimize mutex locks
        let mut batch_processed: u64 = 0;
        let mut batch_failed: u64 = 0;
        let mut batch_errors: Vec<String> = Vec::new();

        let db = conn.lock().unwrap();

        // Begin transaction for this batch
        let tx = match db.unchecked_transaction() {
            Ok(tx) => tx,
            Err(e) => {
                log::error!("Failed to begin transaction: {}", e);
                continue;
            }
        };

        for file in chunk {
            let path_str = file.path.to_string_lossy().to_string();

            // === FAST PATH: Check size + mtime before any I/O ===
            let existing: Option<(i64, Option<String>, Option<String>)> = tx
                .query_row(
                    "SELECT size_bytes, modified_at_fs, fingerprint FROM files WHERE path = ?1 AND is_deleted = 0",
                    [&path_str],
                    |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
                )
                .ok();

            if let Some((db_size, db_mtime, _db_fp)) = &existing {
                // If size and mtime match, skip entirely — no hash, no content extraction
                if *db_size == file.size_bytes as i64
                    && db_mtime.as_deref() == file.modified_at.as_deref()
                {
                    let _ = tx.execute(
                        "UPDATE files SET last_seen_at = datetime('now') WHERE path = ?1",
                        [&path_str],
                    );
                    batch_processed += 1;
                    continue;
                }
            }

            // File is new or changed — compute fingerprint
            let fp = fingerprint::compute_fingerprint(&file.path).unwrap_or_default();

            // Check fingerprint match (handles edge case: size/mtime changed but content didn't)
            if let Some((_, _, Some(ref db_fp))) = existing {
                if db_fp == &fp {
                    let _ = tx.execute(
                        "UPDATE files SET modified_at_fs = ?1, size_bytes = ?2, last_seen_at = datetime('now') WHERE path = ?3",
                        rusqlite::params![file.modified_at, file.size_bytes as i64, path_str],
                    );
                    batch_processed += 1;
                    continue;
                }
            }

            // Content has changed — extract it
            let ext = file.extension.as_deref().unwrap_or("");
            let parser_type = config::parser_type_for_extension(ext);
            let (content, parser_status) = match parser::extract_content(&file.path, parser_type) {
                Ok(text) => (text, "ok"),
                Err(e) => {
                    log::warn!("Extraction failed for {}: {}", path_str, e);
                    batch_failed += 1;
                    if batch_errors.len() < 50 {
                        batch_errors.push(format!("{}: {}", file.filename, e));
                    }
                    (String::new(), "error")
                }
            };

            // Upsert into files table
            let result = tx.execute(
                "INSERT INTO files (root_id, path, filename, extension, size_bytes, created_at_fs, modified_at_fs, fingerprint, parser_type, parser_status, is_deleted, last_indexed_at, last_seen_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, 0, datetime('now'), datetime('now'))
                 ON CONFLICT(path) DO UPDATE SET
                    root_id = excluded.root_id,
                    filename = excluded.filename,
                    extension = excluded.extension,
                    size_bytes = excluded.size_bytes,
                    created_at_fs = excluded.created_at_fs,
                    modified_at_fs = excluded.modified_at_fs,
                    fingerprint = excluded.fingerprint,
                    parser_type = excluded.parser_type,
                    parser_status = excluded.parser_status,
                    is_deleted = 0,
                    last_indexed_at = datetime('now'),
                    last_seen_at = datetime('now')",
                rusqlite::params![
                    root_id,
                    path_str,
                    file.filename,
                    file.extension,
                    file.size_bytes as i64,
                    file.created_at,
                    file.modified_at,
                    fp,
                    parser_type,
                    parser_status,
                ],
            );

            if let Err(e) = result {
                log::error!("Failed to insert file {}: {}", path_str, e);
                batch_processed += 1;
                continue;
            }

            // Get file_id: use last_insert_rowid if it was an INSERT, else query
            let changes = tx.changes();
            let file_id: i64 = if changes > 0 && existing.is_none() {
                tx.last_insert_rowid()
            } else {
                match tx.query_row(
                    "SELECT id FROM files WHERE path = ?1",
                    [&path_str],
                    |row| row.get(0),
                ) {
                    Ok(id) => id,
                    Err(e) => {
                        log::error!("Failed to get file id for {}: {}", path_str, e);
                        batch_processed += 1;
                        continue;
                    }
                }
            };

            // Update FTS index: delete old entry then insert new
            let _ = tx.execute("DELETE FROM files_fts WHERE file_id = ?1", [file_id]);
            let _ = tx.execute(
                "INSERT INTO files_fts (file_id, filename, path, content) VALUES (?1, ?2, ?3, ?4)",
                rusqlite::params![file_id, file.filename, path_str, content],
            );

            batch_processed += 1;
        }

        if let Err(e) = tx.commit() {
            log::error!("Failed to commit batch: {}", e);
        }

        // Flush batch counters to shared status (one lock per batch instead of per file)
        {
            let mut status = index_status.lock().unwrap();
            status.processed_files += batch_processed;
            status.failed_files += batch_failed;
            status.current_file = chunk.last().map(|f| f.filename.clone());
            for err in batch_errors {
                if status.errors.len() < 50 {
                    status.errors.push(err);
                }
            }
        }

        // Emit progress after each batch
        let _ = app_handle.emit("index-progress", get_status_snapshot(index_status));
    }

    // Mark files not seen in this scan as deleted, then clean up
    {
        let db = conn.lock().unwrap();
        let _ = db.execute(
            "UPDATE files
             SET is_deleted = 1
             WHERE root_id = ?1
               AND (last_seen_at IS NULL OR last_seen_at < ?2)",
            rusqlite::params![root_id, scan_started_at],
        );
        // Remove FTS entries for deleted files
        let _ = db.execute(
            "DELETE FROM files_fts WHERE file_id IN (SELECT id FROM files WHERE is_deleted = 1 AND root_id = ?1)",
            [root_id],
        );
        // Hard-delete soft-deleted rows to reclaim space
        let _ = db.execute(
            "DELETE FROM files WHERE is_deleted = 1 AND root_id = ?1",
            [root_id],
        );
        // Optimize FTS index to merge segments
        let _ = db.execute_batch("INSERT INTO files_fts(files_fts) VALUES('optimize');");
    }

    // Set status to complete
    {
        let mut status = index_status.lock().unwrap();
        status.is_running = false;
        status.current_file = None;
    }

    let _ = app_handle.emit("index-complete", get_status_snapshot(index_status));
}

/// Index a single file (used by the watcher for incremental updates).
/// Includes fingerprint dedup: skips content extraction if file is unchanged.
pub fn index_single_file(
    conn: &Mutex<Connection>,
    root_id: i64,
    file_path: &Path,
) -> Result<(), String> {
    let metadata = std::fs::metadata(file_path).map_err(|e| e.to_string())?;
    if !metadata.is_file() {
        return Ok(());
    }

    let path_str = file_path.to_string_lossy().to_string();
    let filename = file_path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();
    let extension = file_path
        .extension()
        .map(|e| e.to_string_lossy().to_lowercase());

    let size_bytes = metadata.len() as i64;
    let modified_at = metadata
        .modified()
        .ok()
        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|d| {
            chrono::DateTime::from_timestamp(d.as_secs() as i64, 0)
                .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
                .unwrap_or_default()
        });

    // Fast path: check if file is unchanged by fingerprint
    let fp = fingerprint::compute_fingerprint(file_path).unwrap_or_default();

    {
        let db = conn.lock().map_err(|e| e.to_string())?;
        let existing_fp: Option<String> = db
            .query_row(
                "SELECT fingerprint FROM files WHERE path = ?1 AND is_deleted = 0",
                [&path_str],
                |row| row.get(0),
            )
            .ok()
            .flatten();

        if existing_fp.as_deref() == Some(fp.as_str()) {
            // File unchanged, just update timestamps
            let _ = db.execute(
                "UPDATE files SET last_seen_at = datetime('now'), modified_at_fs = ?1 WHERE path = ?2",
                rusqlite::params![modified_at, path_str],
            );
            return Ok(());
        }
    }

    let ext = extension.as_deref().unwrap_or("");
    let parser_type = config::parser_type_for_extension(ext);
    let (content, parser_status) = match parser::extract_content(file_path, parser_type) {
        Ok(text) => (text, "ok"),
        Err(_) => (String::new(), "error"),
    };

    let db = conn.lock().map_err(|e| e.to_string())?;

    db.execute(
        "INSERT INTO files (root_id, path, filename, extension, size_bytes, modified_at_fs, fingerprint, parser_type, parser_status, is_deleted, last_indexed_at, last_seen_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, 0, datetime('now'), datetime('now'))
         ON CONFLICT(path) DO UPDATE SET
            filename = excluded.filename,
            extension = excluded.extension,
            size_bytes = excluded.size_bytes,
            modified_at_fs = excluded.modified_at_fs,
            fingerprint = excluded.fingerprint,
            parser_type = excluded.parser_type,
            parser_status = excluded.parser_status,
            is_deleted = 0,
            last_indexed_at = datetime('now'),
            last_seen_at = datetime('now')",
        rusqlite::params![root_id, path_str, filename, extension, size_bytes, modified_at, fp, parser_type, parser_status],
    )
    .map_err(|e| e.to_string())?;

    let file_id: i64 = db
        .query_row("SELECT id FROM files WHERE path = ?1", [&path_str], |row| {
            row.get(0)
        })
        .map_err(|e| e.to_string())?;

    let _ = db.execute("DELETE FROM files_fts WHERE file_id = ?1", [file_id]);
    db.execute(
        "INSERT INTO files_fts (file_id, filename, path, content) VALUES (?1, ?2, ?3, ?4)",
        rusqlite::params![file_id, filename, path_str, content],
    )
    .map_err(|e| e.to_string())?;

    Ok(())
}

/// Remove a file from the index (used by the watcher on delete events).
pub fn remove_file_from_index(conn: &Mutex<Connection>, file_path: &str) -> Result<(), String> {
    let db = conn.lock().map_err(|e| e.to_string())?;

    // Get file_id before deleting
    let file_id: Option<i64> = db
        .query_row(
            "SELECT id FROM files WHERE path = ?1",
            [file_path],
            |row| row.get(0),
        )
        .ok();

    if let Some(id) = file_id {
        let _ = db.execute("DELETE FROM files_fts WHERE file_id = ?1", [id]);
        let _ = db.execute("DELETE FROM files WHERE id = ?1", [id]);
    }

    Ok(())
}

fn get_status_snapshot(index_status: &Mutex<IndexStatus>) -> IndexStatus {
    index_status.lock().unwrap().clone()
}
