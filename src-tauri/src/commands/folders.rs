use crate::db::models::IndexedRoot;
use crate::AppState;
use tauri::State;

#[tauri::command]
pub fn add_indexed_folder(state: State<'_, AppState>, path: String) -> Result<IndexedRoot, String> {
    // Validate the path exists and is a directory
    let p = std::path::Path::new(&path);
    if !p.exists() {
        return Err("Path does not exist".to_string());
    }
    if !p.is_dir() {
        return Err("Path is not a directory".to_string());
    }

    let db = state.db.lock().map_err(|e| e.to_string())?;

    db.execute(
        "INSERT OR IGNORE INTO indexed_roots (path) VALUES (?1)",
        [&path],
    )
    .map_err(|e| format!("Failed to add folder: {}", e))?;

    let root: IndexedRoot = db
        .query_row(
            "SELECT id, path, created_at FROM indexed_roots WHERE path = ?1",
            [&path],
            |row| {
                Ok(IndexedRoot {
                    id: row.get(0)?,
                    path: row.get(1)?,
                    created_at: row.get(2)?,
                })
            },
        )
        .map_err(|e| format!("Failed to retrieve folder: {}", e))?;

    Ok(root)
}

#[tauri::command]
pub fn remove_indexed_folder(state: State<'_, AppState>, id: i64) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;

    // Delete FTS entries for files in this root
    db.execute(
        "DELETE FROM files_fts WHERE file_id IN (SELECT id FROM files WHERE root_id = ?1)",
        [id],
    )
    .map_err(|e| format!("Failed to clean FTS: {}", e))?;

    // Delete files in this root
    db.execute("DELETE FROM files WHERE root_id = ?1", [id])
        .map_err(|e| format!("Failed to delete files: {}", e))?;

    // Delete the root itself
    db.execute("DELETE FROM indexed_roots WHERE id = ?1", [id])
        .map_err(|e| format!("Failed to delete folder: {}", e))?;

    Ok(())
}

#[tauri::command]
pub fn list_indexed_folders(state: State<'_, AppState>) -> Result<Vec<IndexedRoot>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;

    let mut stmt = db
        .prepare("SELECT id, path, created_at FROM indexed_roots ORDER BY created_at DESC")
        .map_err(|e| e.to_string())?;

    let roots = stmt
        .query_map([], |row| {
            Ok(IndexedRoot {
                id: row.get(0)?,
                path: row.get(1)?,
                created_at: row.get(2)?,
            })
        })
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();

    Ok(roots)
}
