use crate::db::models::{SearchResult, SearchResults};
use rusqlite::Connection;

/// Execute a search query against the FTS5 index.
/// Returns ranked results with snippets.
pub fn execute_search(
    conn: &Connection,
    query: &str,
    extensions: &Option<Vec<String>>,
    root_id: &Option<i64>,
    limit: i64,
    offset: i64,
) -> Result<SearchResults, String> {
    if query.trim().is_empty() {
        return Ok(SearchResults {
            results: vec![],
            total_count: 0,
        });
    }

    // Sanitize query for FTS5: escape double quotes and wrap terms
    let fts_query = sanitize_fts_query(query);

    // Build dynamic WHERE clauses for filters
    let mut extra_where = String::new();
    let mut filter_params: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();

    filter_params.push(Box::new(fts_query.clone()));

    if let Some(exts) = extensions {
        if !exts.is_empty() {
            let placeholders: Vec<String> = exts
                .iter()
                .enumerate()
                .map(|(i, _)| format!("?{}", i + 2))
                .collect();
            extra_where.push_str(&format!(
                " AND f.extension IN ({})",
                placeholders.join(", ")
            ));
            for ext in exts {
                filter_params.push(Box::new(ext.clone()));
            }
        }
    }

    let root_param_idx = filter_params.len() + 1;
    if let Some(rid) = root_id {
        extra_where.push_str(&format!(" AND f.root_id = ?{}", root_param_idx));
        filter_params.push(Box::new(*rid));
    }

    let total_sql = format!(
        "SELECT COUNT(*)
         FROM files_fts
         JOIN files f ON f.id = files_fts.file_id
         WHERE files_fts MATCH ?1
           AND f.is_deleted = 0
           {}",
        extra_where
    );

    let filter_param_refs: Vec<&dyn rusqlite::types::ToSql> =
        filter_params.iter().map(|p| p.as_ref()).collect();

    let total_count: i64 = conn
        .query_row(&total_sql, filter_param_refs.as_slice(), |row| row.get(0))
        .map_err(|e| format!("Count query error: {}", e))?;

    let mut result_params = filter_params;
    let limit_idx = result_params.len() + 1;
    let offset_idx = result_params.len() + 2;
    result_params.push(Box::new(limit));
    result_params.push(Box::new(offset));

    let sql = format!(
        "SELECT
            f.id,
            f.path,
            f.filename,
            f.extension,
            f.size_bytes,
            f.modified_at_fs,
            snippet(files_fts, 2, '<mark>', '</mark>', '...', 48) as snippet,
            bm25(files_fts, 10.0, 5.0, 1.0) as rank
        FROM files_fts
        JOIN files f ON f.id = files_fts.file_id
        WHERE files_fts MATCH ?1
            AND f.is_deleted = 0
            {}
        ORDER BY rank
        LIMIT ?{} OFFSET ?{}",
        extra_where, limit_idx, offset_idx
    );

    let param_refs: Vec<&dyn rusqlite::types::ToSql> =
        result_params.iter().map(|p| p.as_ref()).collect();

    let mut stmt = conn.prepare(&sql).map_err(|e| format!("Query prepare error: {}", e))?;

    let results: Vec<SearchResult> = stmt
        .query_map(param_refs.as_slice(), |row| {
            Ok(SearchResult {
                file_id: row.get(0)?,
                path: row.get(1)?,
                filename: row.get(2)?,
                extension: row.get(3)?,
                size_bytes: row.get(4)?,
                modified_at_fs: row.get(5)?,
                snippet: row.get::<_, String>(6).unwrap_or_default(),
                rank: row.get(7)?,
            })
        })
        .map_err(|e| format!("Query error: {}", e))?
        .filter_map(|r| r.ok())
        .collect();

    Ok(SearchResults {
        results,
        total_count,
    })
}

/// Sanitize a user query string for FTS5.
/// Handles basic cases: if the query contains quotes, pass through as phrase search.
/// Otherwise, split into terms and join with implicit AND.
fn sanitize_fts_query(query: &str) -> String {
    let trimmed = query.trim();

    // If it looks like the user is doing a phrase search, pass through
    if trimmed.starts_with('"') && trimmed.ends_with('"') {
        return trimmed.to_string();
    }

    // Split into words and create FTS5-safe query
    let terms: Vec<&str> = trimmed.split_whitespace().collect();

    if terms.is_empty() {
        return String::new();
    }

    if terms.len() == 1 {
        // Single term: use prefix matching for better UX
        return format!("{}*", escape_fts_term(terms[0]));
    }

    // Multiple terms: AND them together with prefix on last term
    let mut parts: Vec<String> = terms[..terms.len() - 1]
        .iter()
        .map(|t| escape_fts_term(t))
        .collect();
    // Last term gets prefix matching for type-ahead feel
    parts.push(format!("{}*", escape_fts_term(terms[terms.len() - 1])));

    parts.join(" ")
}

fn escape_fts_term(term: &str) -> String {
    // Remove characters that are special in FTS5 syntax
    term.chars()
        .filter(|c| c.is_alphanumeric() || *c == '_' || *c == '-' || *c == '.')
        .collect()
}
