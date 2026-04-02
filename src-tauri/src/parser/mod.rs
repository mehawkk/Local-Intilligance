mod docx_parser;
mod pdf_parser;
mod text_parser;

use std::path::Path;

/// Extract text content from a file based on its parser type.
/// Returns Ok(String) with the extracted text, or Err(String) with an error message.
/// Errors are non-fatal: the file will still be indexed with metadata only.
pub fn extract_content(path: &Path, parser_type: &str) -> Result<String, String> {
    match parser_type {
        "text" => text_parser::extract(path),
        "pdf" => pdf_parser::extract(path),
        "docx" => docx_parser::extract(path),
        _ => Ok(String::new()),
    }
}
