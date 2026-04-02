use crate::db::models::IndexStatus;
use crate::indexer;
use crate::AppState;
use tauri::State;

#[tauri::command]
pub fn start_scan(
    state: State<'_, AppState>,
    app_handle: tauri::AppHandle,
    root_id: i64,
) -> Result<(), String> {
    // Check if already running
    {
        let status = state.index_status.lock().map_err(|e| e.to_string())?;
        if status.is_running {
            return Err("Indexing is already in progress".to_string());
        }
    }

    // Get root path
    let root_path: String = {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        db.query_row(
            "SELECT path FROM indexed_roots WHERE id = ?1",
            [root_id],
            |row| row.get(0),
        )
        .map_err(|e| format!("Root not found: {}", e))?
    };

    // Get exclusions
    let exclusions = get_exclusions(&state)?;

    // Spawn indexing in background thread
    let db = state.db_arc.clone();
    let index_status = state.index_status_arc.clone();

    std::thread::spawn(move || {
        indexer::run_full_scan(&db, &index_status, &app_handle, root_id, &root_path, &exclusions);
    });

    Ok(())
}

#[tauri::command]
pub fn trigger_rescan(
    state: State<'_, AppState>,
    app_handle: tauri::AppHandle,
) -> Result<(), String> {
    // Check if already running
    {
        let status = state.index_status.lock().map_err(|e| e.to_string())?;
        if status.is_running {
            return Err("Indexing is already in progress".to_string());
        }
    }

    // Get all roots
    let roots = get_roots(&state)?;

    if roots.is_empty() {
        return Err("No indexed folders to scan".to_string());
    }

    // Get exclusions
    let exclusions = get_exclusions(&state)?;

    let db = state.db_arc.clone();
    let index_status = state.index_status_arc.clone();

    std::thread::spawn(move || {
        for (root_id, root_path) in roots {
            indexer::run_full_scan(&db, &index_status, &app_handle, root_id, &root_path, &exclusions);
        }
    });

    Ok(())
}

#[tauri::command]
pub fn get_index_status(state: State<'_, AppState>) -> Result<IndexStatus, String> {
    let status = state.index_status.lock().map_err(|e| e.to_string())?;
    Ok(status.clone())
}

fn get_exclusions(state: &State<'_, AppState>) -> Result<Vec<String>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let mut stmt = db
        .prepare("SELECT pattern FROM excluded_paths")
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([], |row| row.get(0))
        .map_err(|e| e.to_string())?;
    let mut result = Vec::new();
    for row in rows {
        if let Ok(val) = row {
            result.push(val);
        }
    }
    Ok(result)
}

fn get_roots(state: &State<'_, AppState>) -> Result<Vec<(i64, String)>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let mut stmt = db
        .prepare("SELECT id, path FROM indexed_roots")
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))
        .map_err(|e| e.to_string())?;
    let mut result = Vec::new();
    for row in rows {
        if let Ok(val) = row {
            result.push(val);
        }
    }
    Ok(result)
}
