use crate::db::models::SearchResults;
use crate::search;
use crate::AppState;
use tauri::State;

#[tauri::command]
pub fn search_files(
    state: State<'_, AppState>,
    query: String,
    extensions: Option<Vec<String>>,
    root_id: Option<i64>,
    limit: Option<i64>,
    offset: Option<i64>,
) -> Result<SearchResults, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;

    search::execute_search(
        &db,
        &query,
        &extensions,
        &root_id,
        limit.unwrap_or(50),
        offset.unwrap_or(0),
    )
}
