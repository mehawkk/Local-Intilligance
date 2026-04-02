use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;
use walkdir::WalkDir;

#[derive(Debug, Clone)]
pub struct CrawledFile {
    pub path: PathBuf,
    pub filename: String,
    pub extension: Option<String>,
    pub size_bytes: u64,
    pub created_at: Option<String>,
    pub modified_at: Option<String>,
}

/// Recursively crawl a directory, skipping excluded paths.
/// Returns a Vec of CrawledFile for all regular files found.
/// Uses HashSet for O(1) exclusion lookups on path components.
pub fn crawl_directory(root: &Path, exclusions: &[String]) -> Vec<CrawledFile> {
    let exclusion_set: HashSet<&str> = exclusions.iter().map(|s| s.as_str()).collect();

    WalkDir::new(root)
        .follow_links(false)
        .into_iter()
        .filter_entry(|entry| !is_excluded(entry.path(), &exclusion_set))
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.file_type().is_file())
        .filter_map(|entry| {
            let path = entry.path().to_path_buf();
            let filename = entry.file_name().to_string_lossy().into_owned();
            let extension = path
                .extension()
                .map(|ext| ext.to_string_lossy().to_lowercase());
            let metadata = entry.metadata().ok()?;
            let size_bytes = metadata.len();

            let created_at = metadata
                .created()
                .ok()
                .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
                .map(|d| {
                    chrono::DateTime::from_timestamp(d.as_secs() as i64, 0)
                        .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
                        .unwrap_or_default()
                });

            let modified_at = metadata
                .modified()
                .ok()
                .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
                .map(|d| {
                    chrono::DateTime::from_timestamp(d.as_secs() as i64, 0)
                        .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
                        .unwrap_or_default()
                });

            Some(CrawledFile {
                path,
                filename,
                extension,
                size_bytes,
                created_at,
                modified_at,
            })
        })
        .collect()
}

/// Check if a path should be excluded based on exclusion patterns.
/// Uses HashSet for O(1) component lookup instead of O(n) string-contains.
/// Falls back to substring matching for patterns containing path separators.
fn is_excluded(path: &Path, exclusion_set: &HashSet<&str>) -> bool {
    for component in path.components() {
        if let Some(s) = component.as_os_str().to_str() {
            if exclusion_set.contains(s) {
                return true;
            }
        }
    }
    // Fallback: check patterns with path separators (e.g., "AppData\Local\Temp")
    let path_str = path.to_string_lossy();
    for exclusion in exclusion_set {
        if exclusion.contains('\\') || exclusion.contains('/') {
            if path_str.contains(exclusion.as_ref() as &str) {
                return true;
            }
        }
    }
    false
}
