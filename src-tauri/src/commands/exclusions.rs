use crate::db::models::ExcludedPath;
use crate::AppState;
use tauri::State;

#[tauri::command]
pub fn add_exclusion(state: State<'_, AppState>, pattern: String) -> Result<ExcludedPath, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;

    db.execute(
        "INSERT OR IGNORE INTO excluded_paths (pattern) VALUES (?1)",
        [&pattern],
    )
    .map_err(|e| format!("Failed to add exclusion: {}", e))?;

    let exclusion: ExcludedPath = db
        .query_row(
            "SELECT id, pattern, created_at FROM excluded_paths WHERE pattern = ?1",
            [&pattern],
            |row| {
                Ok(ExcludedPath {
                    id: row.get(0)?,
                    pattern: row.get(1)?,
                    created_at: row.get(2)?,
                })
            },
        )
        .map_err(|e| format!("Failed to retrieve exclusion: {}", e))?;

    Ok(exclusion)
}

#[tauri::command]
pub fn remove_exclusion(state: State<'_, AppState>, id: i64) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;

    db.execute("DELETE FROM excluded_paths WHERE id = ?1", [id])
        .map_err(|e| format!("Failed to delete exclusion: {}", e))?;

    Ok(())
}

#[tauri::command]
pub fn list_exclusions(state: State<'_, AppState>) -> Result<Vec<ExcludedPath>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;

    let mut stmt = db
        .prepare("SELECT id, pattern, created_at FROM excluded_paths ORDER BY pattern ASC")
        .map_err(|e| e.to_string())?;

    let exclusions = stmt
        .query_map([], |row| {
            Ok(ExcludedPath {
                id: row.get(0)?,
                pattern: row.get(1)?,
                created_at: row.get(2)?,
            })
        })
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();

    Ok(exclusions)
}
