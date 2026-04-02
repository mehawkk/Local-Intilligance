use crate::config::MAX_TEXT_SIZE;
use std::path::Path;

/// Extract text content from a PDF file using pdf-extract.
/// Wrapped in catch_unwind because pdf-extract can panic on malformed PDFs.
pub fn extract(path: &Path) -> Result<String, String> {
    let path_buf = path.to_path_buf();

    let result = std::panic::catch_unwind(move || {
        pdf_extract::extract_text(&path_buf)
    });

    match result {
        Ok(Ok(text)) => {
            let truncated = if text.len() > MAX_TEXT_SIZE {
                text[..MAX_TEXT_SIZE].to_string()
            } else {
                text
            };
            Ok(truncated)
        }
        Ok(Err(e)) => Err(format!("PDF extraction error: {}", e)),
        Err(_) => Err("PDF extraction panicked on malformed file".to_string()),
    }
}
