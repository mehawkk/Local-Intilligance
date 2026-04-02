use crate::indexer;
use notify::RecommendedWatcher;
use notify_debouncer_full::{new_debouncer, DebounceEventResult, Debouncer, FileIdMap};
use rusqlite::Connection;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::Duration;
use tauri::Emitter;

pub type WatcherHandle = Debouncer<RecommendedWatcher, FileIdMap>;

/// Start watching the given paths for file changes.
/// Returns a Debouncer handle that must be kept alive.
/// Uses HashMap for O(1) root_id lookups instead of linear search.
pub fn start_watching(
    app_handle: tauri::AppHandle,
    db: std::sync::Arc<Mutex<Connection>>,
    paths: Vec<(i64, PathBuf)>, // (root_id, path) pairs
) -> Result<WatcherHandle, String> {
    let db_clone = db.clone();
    let handle_clone = app_handle.clone();

    // Build HashMap for O(1) root lookups (path -> root_id)
    let root_map: HashMap<PathBuf, i64> = paths.iter().map(|(id, p)| (p.clone(), *id)).collect();

    let mut debouncer = new_debouncer(
        Duration::from_secs(2),
        None,
        move |result: DebounceEventResult| {
            match result {
                Ok(events) => {
                    for event in events {
                        process_event(&db_clone, &handle_clone, &root_map, &event);
                    }
                }
                Err(errors) => {
                    for e in errors {
                        log::error!("Watch error: {:?}", e);
                    }
                }
            }
        },
    )
    .map_err(|e| format!("Failed to create debouncer: {}", e))?;

    for (_root_id, path) in &paths {
        debouncer
            .watch(path, notify::RecursiveMode::Recursive)
            .map_err(|e| format!("Failed to watch {}: {}", path.display(), e))?;
    }

    Ok(debouncer)
}

fn process_event(
    db: &std::sync::Arc<Mutex<Connection>>,
    app_handle: &tauri::AppHandle,
    root_map: &HashMap<PathBuf, i64>,
    event: &notify_debouncer_full::DebouncedEvent,
) {
    use notify::EventKind;

    for path in &event.paths {
        // Skip directories
        if path.is_dir() {
            continue;
        }

        // Find the root_id for this path using HashMap prefix matching
        let root_id = find_root_id(root_map, path);

        let root_id = match root_id {
            Some(id) => id,
            None => continue,
        };

        match event.kind {
            EventKind::Create(_) | EventKind::Modify(_) => {
                if path.exists() {
                    // index_single_file already includes fingerprint dedup
                    if let Err(e) = indexer::index_single_file(db, root_id, path) {
                        log::error!("Failed to index {}: {}", path.display(), e);
                    } else {
                        let _ = app_handle
                            .emit("file-changed", path.to_string_lossy().to_string());
                    }
                }
            }
            EventKind::Remove(_) => {
                let path_str = path.to_string_lossy().to_string();
                if let Err(e) = indexer::remove_file_from_index(db, &path_str) {
                    log::error!("Failed to remove {}: {}", path_str, e);
                } else {
                    let _ = app_handle.emit("file-changed", path_str);
                }
            }
            _ => {}
        }
    }
}

/// Find root_id for a file path by checking which root is a prefix.
/// O(n) on number of roots (typically 1-5), but each check is O(1) via starts_with.
fn find_root_id(root_map: &HashMap<PathBuf, i64>, path: &PathBuf) -> Option<i64> {
    for (root_path, id) in root_map {
        if path.starts_with(root_path) {
            return Some(*id);
        }
    }
    None
}
