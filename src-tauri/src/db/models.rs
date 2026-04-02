use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexedRoot {
    pub id: i64,
    pub path: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExcludedPath {
    pub id: i64,
    pub pattern: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileRecord {
    pub id: i64,
    pub root_id: i64,
    pub path: String,
    pub filename: String,
    pub extension: Option<String>,
    pub size_bytes: i64,
    pub created_at_fs: Option<String>,
    pub modified_at_fs: Option<String>,
    pub fingerprint: Option<String>,
    pub parser_type: String,
    pub parser_status: String,
    pub is_deleted: bool,
    pub last_indexed_at: Option<String>,
    pub last_seen_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub file_id: i64,
    pub path: String,
    pub filename: String,
    pub extension: Option<String>,
    pub size_bytes: i64,
    pub modified_at_fs: Option<String>,
    pub snippet: String,
    pub rank: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResults {
    pub results: Vec<SearchResult>,
    pub total_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct IndexStatus {
    pub is_running: bool,
    pub total_files: u64,
    pub processed_files: u64,
    pub current_file: Option<String>,
    pub failed_files: u64,
    pub errors: Vec<String>,
}
