use sha2::{Digest, Sha256};
use std::io::Read;
use std::path::Path;
use std::time::UNIX_EPOCH;

/// Compute a lightweight fingerprint for change detection.
/// Format: "{size}:{mtime_secs}:{sha256_prefix}"
/// For files <= 64KB, hashes the entire file.
/// For larger files, hashes only the first 64KB.
pub fn compute_fingerprint(path: &Path) -> Result<String, String> {
    let metadata = std::fs::metadata(path).map_err(|e| format!("metadata error: {}", e))?;

    let size = metadata.len();
    let modified = metadata
        .modified()
        .map_err(|e| format!("mtime error: {}", e))?
        .duration_since(UNIX_EPOCH)
        .map_err(|e| format!("time error: {}", e))?
        .as_secs();

    let mut hasher = Sha256::new();
    let file = std::fs::File::open(path).map_err(|e| format!("open error: {}", e))?;
    let mut reader = std::io::BufReader::new(file);

    // Read up to 64KB for hashing
    let mut buffer = vec![0u8; 65536];
    let bytes_read = reader
        .read(&mut buffer)
        .map_err(|e| format!("read error: {}", e))?;
    hasher.update(&buffer[..bytes_read]);

    let hash = format!("{:x}", hasher.finalize());
    // Use first 16 chars of the hash for compactness
    Ok(format!("{}:{}:{}", size, modified, &hash[..16]))
}
