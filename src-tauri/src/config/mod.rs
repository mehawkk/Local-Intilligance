/// Maximum text content to extract from a single file (1 MB).
pub const MAX_TEXT_SIZE: usize = 1_048_576;

/// Determine the parser type for a given file extension.
pub fn parser_type_for_extension(ext: &str) -> &'static str {
    match ext.to_lowercase().as_str() {
        // Plain text and code files
        "txt" | "md" | "markdown" | "rst" => "text",
        "rs" | "py" | "js" | "ts" | "tsx" | "jsx" | "java" | "c" | "cpp" | "h" | "hpp"
        | "cs" | "go" | "rb" | "php" | "swift" | "kt" | "scala" | "r" | "lua" | "pl"
        | "pm" => "text",
        // Config and data files
        "json" | "yaml" | "yml" | "toml" | "xml" | "html" | "htm" | "css" | "scss" | "sass"
        | "less" | "svg" => "text",
        // Shell and scripts
        "sh" | "bash" | "zsh" | "fish" | "bat" | "cmd" | "ps1" | "psm1" => "text",
        // SQL and database
        "sql" | "graphql" | "gql" => "text",
        // Other text formats
        "log" | "ini" | "cfg" | "conf" | "env" | "gitignore" | "dockerignore"
        | "editorconfig" | "properties" => "text",
        "makefile" | "cmake" | "dockerfile" => "text",
        // CSV/TSV
        "csv" | "tsv" => "text",
        // PDF
        "pdf" => "pdf",
        // Word documents
        "docx" => "docx",
        // Everything else: metadata only
        _ => "none",
    }
}
