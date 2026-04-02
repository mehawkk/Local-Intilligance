import { invoke } from "@tauri-apps/api/core";

// --- Types ---

export interface IndexedRoot {
  id: number;
  path: string;
  created_at: string;
}

export interface ExcludedPath {
  id: number;
  pattern: string;
  created_at: string;
}

export interface SearchResult {
  file_id: number;
  path: string;
  filename: string;
  extension: string | null;
  size_bytes: number;
  modified_at_fs: string | null;
  snippet: string;
  rank: number;
}

export interface SearchResults {
  results: SearchResult[];
  total_count: number;
}

export interface IndexStatus {
  is_running: boolean;
  total_files: number;
  processed_files: number;
  current_file: string | null;
  failed_files: number;
  errors: string[];
}

// --- Folder Commands ---

export async function addIndexedFolder(path: string): Promise<IndexedRoot> {
  return invoke("add_indexed_folder", { path });
}

export async function removeIndexedFolder(id: number): Promise<void> {
  return invoke("remove_indexed_folder", { id });
}

export async function listIndexedFolders(): Promise<IndexedRoot[]> {
  return invoke("list_indexed_folders");
}

// --- Exclusion Commands ---

export async function addExclusion(pattern: string): Promise<ExcludedPath> {
  return invoke("add_exclusion", { pattern });
}

export async function removeExclusion(id: number): Promise<void> {
  return invoke("remove_exclusion", { id });
}

export async function listExclusions(): Promise<ExcludedPath[]> {
  return invoke("list_exclusions");
}

// --- Indexing Commands ---

export async function startScan(rootId: number): Promise<void> {
  return invoke("start_scan", { rootId });
}

export async function triggerRescan(): Promise<void> {
  return invoke("trigger_rescan");
}

export async function getIndexStatus(): Promise<IndexStatus> {
  return invoke("get_index_status");
}

// --- Search Commands ---

export async function searchFiles(
  query: string,
  extensions?: string[],
  rootId?: number,
  limit?: number,
  offset?: number
): Promise<SearchResults> {
  return invoke("search_files", {
    query,
    extensions: extensions ?? null,
    rootId: rootId ?? null,
    limit: limit ?? 50,
    offset: offset ?? 0,
  });
}

// --- File Commands ---

export async function openFile(path: string): Promise<void> {
  return invoke("open_file", { path });
}

export async function openContainingFolder(path: string): Promise<void> {
  return invoke("open_containing_folder", { path });
}
