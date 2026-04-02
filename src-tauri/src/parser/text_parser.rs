use crate::config::MAX_TEXT_SIZE;
use std::io::Read;
use std::path::Path;

/// Extract text content from a plain text or code file.
/// Uses BufReader with take() to read at most MAX_TEXT_SIZE bytes,
/// avoiding loading entire large files into memory.
pub fn extract(path: &Path) -> Result<String, String> {
    let file = std::fs::File::open(path).map_err(|e| format!("Failed to open file: {}", e))?;
    let mut reader = std::io::BufReader::new(file.take(MAX_TEXT_SIZE as u64));
    let mut buf = Vec::with_capacity(MAX_TEXT_SIZE.min(8192));
    reader
        .read_to_end(&mut buf)
        .map_err(|e| format!("Failed to read file: {}", e))?;

    Ok(String::from_utf8_lossy(&buf).into_owned())
}
