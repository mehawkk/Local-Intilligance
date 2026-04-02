import { useState, useCallback, useEffect, useRef } from "react";
import { SearchIcon } from "lucide-react";
import { SearchBar } from "../components/SearchBar";
import { SearchResultItem } from "../components/SearchResultItem";
import { SearchFilters } from "../components/SearchFilters";
import { searchFiles, type SearchResults } from "../lib/commands";
import { onFileChanged } from "../lib/events";

export function SearchPage() {
  const [query, setQuery] = useState("");
  const [results, setResults] = useState<SearchResults | null>(null);
  const [loading, setLoading] = useState(false);
  const [extensions, setExtensions] = useState<string[]>([]);
  const [rootId, setRootId] = useState<number | undefined>();
  const [page, setPage] = useState(0);
  const pageSize = 50;

  // Track the latest search request to cancel stale ones
  const searchIdRef = useRef(0);

  const doSearch = useCallback(
    async (q: string, exts: string[], rid: number | undefined, pg: number) => {
      if (!q.trim()) {
        setResults(null);
        return;
      }

      const thisSearchId = ++searchIdRef.current;
      setLoading(true);

      try {
        const res = await searchFiles(
          q,
          exts.length > 0 ? exts : undefined,
          rid,
          pageSize,
          pg * pageSize
        );
        // Only apply results if this is still the latest search
        if (thisSearchId === searchIdRef.current) {
          setResults(res);
        }
      } catch (e) {
        console.error("Search error:", e);
        if (thisSearchId === searchIdRef.current) {
          setResults({ results: [], total_count: 0 });
        }
      } finally {
        if (thisSearchId === searchIdRef.current) {
          setLoading(false);
        }
      }
    },
    []
  );

  const handleSearch = useCallback(
    (q: string) => {
      setQuery(q);
      setPage(0);
      doSearch(q, extensions, rootId, 0);
    },
    [doSearch, extensions, rootId]
  );

  // Re-search when filters change (not on query change — that's handled by handleSearch)
  const prevFiltersRef = useRef({ extensions, rootId });
  useEffect(() => {
    const prev = prevFiltersRef.current;
    if (prev.extensions !== extensions || prev.rootId !== rootId) {
      prevFiltersRef.current = { extensions, rootId };
      if (query.trim()) {
        setPage(0);
        doSearch(query, extensions, rootId, 0);
      }
    }
  }, [extensions, rootId, query, doSearch]);

  // Debounced re-search on file changes (2-second window to avoid spam during indexing)
  useEffect(() => {
    let debounceTimer: ReturnType<typeof setTimeout> | null = null;

    const unlisten = onFileChanged(() => {
      if (query.trim()) {
        if (debounceTimer) clearTimeout(debounceTimer);
        debounceTimer = setTimeout(() => {
          doSearch(query, extensions, rootId, page);
        }, 2000);
      }
    });

    return () => {
      if (debounceTimer) clearTimeout(debounceTimer);
      unlisten.then((fn) => fn());
    };
  }, [query, extensions, rootId, page, doSearch]);

  const totalPages = results
    ? Math.ceil(results.total_count / pageSize)
    : 0;

  return (
    <div className="flex flex-col h-full p-4 gap-3">
      <SearchBar onSearch={handleSearch} />
      <SearchFilters
        extensions={extensions}
        rootId={rootId}
        onExtensionsChange={setExtensions}
        onRootIdChange={setRootId}
      />

      {/* Results area */}
      <div className="flex-1 overflow-y-auto">
        {loading && (
          <div className="flex items-center justify-center py-12 text-zinc-500">
            <div className="animate-pulse text-sm">Searching...</div>
          </div>
        )}

        {!loading && !results && !query && (
          <div className="flex flex-col items-center justify-center py-20 text-zinc-600 gap-3">
            <SearchIcon className="w-16 h-16 text-zinc-700" />
            <p className="text-lg font-medium text-zinc-500">
              Search your indexed files
            </p>
            <p className="text-sm text-zinc-600">
              Type a query above to search filenames, paths, and file contents
            </p>
            <p className="text-xs text-zinc-700">
              Ctrl+K to focus search | "quotes" for exact phrase
            </p>
          </div>
        )}

        {!loading && results && results.results.length === 0 && (
          <div className="flex flex-col items-center justify-center py-12 text-zinc-500 gap-2">
            <p className="text-sm">No results found for "{query}"</p>
            <p className="text-xs text-zinc-600">
              Try different keywords or check your indexed folders in Settings
            </p>
          </div>
        )}

        {!loading && results && results.results.length > 0 && (
          <div className="flex flex-col gap-2">
            <div className="text-xs text-zinc-500 mb-1">
              {results.total_count} result{results.total_count !== 1 ? "s" : ""}
              {totalPages > 1 && ` — page ${page + 1} of ${totalPages}`}
            </div>
            {results.results.map((r) => (
              <SearchResultItem key={r.file_id} result={r} />
            ))}

            {/* Pagination */}
            {totalPages > 1 && (
              <div className="flex items-center justify-center gap-2 py-3">
                <button
                  onClick={() => {
                    const newPage = page - 1;
                    setPage(newPage);
                    doSearch(query, extensions, rootId, newPage);
                  }}
                  disabled={page === 0}
                  className="px-3 py-1 text-xs rounded bg-zinc-800 text-zinc-300 hover:bg-zinc-700 disabled:opacity-40 disabled:cursor-not-allowed"
                >
                  Previous
                </button>
                <span className="text-xs text-zinc-500">
                  {page + 1} / {totalPages}
                </span>
                <button
                  onClick={() => {
                    const newPage = page + 1;
                    setPage(newPage);
                    doSearch(query, extensions, rootId, newPage);
                  }}
                  disabled={page >= totalPages - 1}
                  className="px-3 py-1 text-xs rounded bg-zinc-800 text-zinc-300 hover:bg-zinc-700 disabled:opacity-40 disabled:cursor-not-allowed"
                >
                  Next
                </button>
              </div>
            )}
          </div>
        )}
      </div>
    </div>
  );
}
