# Local File Intelligence

A Windows desktop app that indexes local folders and provides instant file/content search. Built with Tauri v2 + Rust + React + TypeScript + SQLite FTS5.

## Features

- **Fast local search**: Searches an indexed database, never scans the filesystem at query time
- **Content extraction**: Indexes text inside txt, md, code files, CSV, PDF, and DOCX
- **File watching**: Detects file changes and updates the index incrementally
- **Exclusion patterns**: Skip node_modules, .git, and other configurable patterns
- **Keyboard-driven**: Ctrl+K to focus search, phrase search with "quotes"
- **Fully offline**: All data stays local, no cloud APIs

## Prerequisites

### 1. Install Rust
Download and run the installer from https://rustup.rs

When prompted, accept the defaults (stable toolchain, MSVC target). You also need **Visual Studio C++ Build Tools** - the Rust installer will prompt you if they're missing.

After installation, restart your terminal and verify:
```bash
rustc --version
cargo --version
```

### 2. Node.js
Node.js 18+ and npm must be installed. Verify:
```bash
node --version
npm --version
```

## Getting Started

```bash
# Install dependencies
cd "E:\workspace\local intilligance"
npm install

# Run in development mode
npm run tauri dev
```

The first run will compile all Rust dependencies (~3-5 minutes). Subsequent runs are fast thanks to incremental compilation.

## Usage

1. **Add folders**: Go to Settings, click "Add Folder" to select directories to index
2. **Wait for indexing**: The status bar shows progress as files are indexed
3. **Search**: Go to Search, type keywords to find files by name, path, or content
4. **Open files**: Click a result to open it, or use the folder icon to reveal in Explorer
5. **Filter**: Use the Filters button to narrow by file type or indexed folder

## Architecture

```
src-tauri/src/
├── lib.rs          # App setup, state management, command registration
├── db/             # SQLite initialization, schema, models
├── config/         # Extension-to-parser mapping
├── parser/         # Content extraction (text, PDF, DOCX)
├── crawler/        # Directory walking with exclusion filtering
├── indexer/        # Full scan pipeline with batch transactions
├── watcher/        # File system monitoring (notify crate)
├── search/         # FTS5 query execution with BM25 ranking
└── commands/       # Tauri command handlers (API layer)
```

### How Indexing Works

1. **Initial scan**: Recursively walks selected folders, skipping excluded paths
2. **Fingerprinting**: Computes `size:mtime:sha256_prefix` to detect changes
3. **Content extraction**: Extracts text from supported file types (txt, code, PDF, DOCX)
4. **FTS5 indexing**: Stores searchable text in SQLite FTS5 virtual table
5. **Batch commits**: Processes 100 files per transaction for performance
6. **File watching**: Uses Windows ReadDirectoryChangesW (via notify crate) for incremental updates

### How Search Works

- Queries the FTS5 index with BM25 ranking
- Weights: filename matches (10x) > path matches (5x) > content matches (1x)
- Supports prefix matching on the last term for type-ahead feel
- Supports phrase search with "double quotes"
- Returns highlighted snippets from matched content

## Production Build

```bash
npm run tauri build
```

Output: `src-tauri/target/release/bundle/nsis/` (Windows installer)

## Known Limitations (v1)

- Single DB connection with Mutex (may bottleneck on very large indexes)
- PDF extraction is basic (pdf-extract struggles with some complex PDFs)
- Exclusions use simple string matching, not glob patterns
- No system tray / background mode
- No search history or saved searches
- No file content preview panel
- No pause/resume for indexing

## Tech Stack

| Component | Technology |
|-----------|-----------|
| Desktop framework | Tauri v2 |
| Backend | Rust |
| Frontend | React 18 + TypeScript |
| Database | SQLite with FTS5 (rusqlite) |
| File watching | notify crate |
| PDF extraction | pdf-extract |
| DOCX extraction | zip + quick-xml |
| Styling | Tailwind CSS v4 |
| Icons | Lucide React |
