import { useEffect, useState } from "react";
import { Filter, X } from "lucide-react";
import { listIndexedFolders, type IndexedRoot } from "../lib/commands";

interface SearchFiltersProps {
  extensions: string[];
  rootId: number | undefined;
  onExtensionsChange: (exts: string[]) => void;
  onRootIdChange: (rootId: number | undefined) => void;
}

const COMMON_EXTENSIONS = [
  "txt",
  "md",
  "pdf",
  "docx",
  "json",
  "csv",
  "py",
  "js",
  "ts",
  "rs",
  "java",
  "html",
  "css",
];

export function SearchFilters({
  extensions,
  rootId,
  onExtensionsChange,
  onRootIdChange,
}: SearchFiltersProps) {
  const [roots, setRoots] = useState<IndexedRoot[]>([]);
  const [showFilters, setShowFilters] = useState(false);

  useEffect(() => {
    listIndexedFolders().then(setRoots).catch(console.error);
  }, []);

  const toggleExtension = (ext: string) => {
    if (extensions.includes(ext)) {
      onExtensionsChange(extensions.filter((e) => e !== ext));
    } else {
      onExtensionsChange([...extensions, ext]);
    }
  };

  const clearAll = () => {
    onExtensionsChange([]);
    onRootIdChange(undefined);
  };

  const hasFilters = extensions.length > 0 || rootId !== undefined;

  return (
    <div className="flex flex-col gap-2">
      <div className="flex items-center gap-2">
        <button
          onClick={() => setShowFilters(!showFilters)}
          className={`flex items-center gap-1.5 px-2.5 py-1 rounded text-xs transition-colors ${
            showFilters || hasFilters
              ? "bg-blue-500/20 text-blue-400 border border-blue-500/30"
              : "text-zinc-400 hover:text-zinc-200 border border-zinc-700 hover:border-zinc-600"
          }`}
        >
          <Filter className="w-3.5 h-3.5" />
          Filters
          {hasFilters && (
            <span className="bg-blue-500 text-white rounded-full w-4 h-4 flex items-center justify-center text-[10px]">
              {extensions.length + (rootId !== undefined ? 1 : 0)}
            </span>
          )}
        </button>
        {hasFilters && (
          <button
            onClick={clearAll}
            className="flex items-center gap-1 text-xs text-zinc-500 hover:text-zinc-300"
          >
            <X className="w-3 h-3" />
            Clear
          </button>
        )}
      </div>

      {showFilters && (
        <div className="flex flex-col gap-3 p-3 bg-zinc-900 border border-zinc-800 rounded-lg">
          {/* Extension filter */}
          <div>
            <label className="text-xs text-zinc-400 mb-1.5 block">
              File Types
            </label>
            <div className="flex flex-wrap gap-1.5">
              {COMMON_EXTENSIONS.map((ext) => (
                <button
                  key={ext}
                  onClick={() => toggleExtension(ext)}
                  className={`px-2 py-0.5 rounded text-xs font-mono transition-colors ${
                    extensions.includes(ext)
                      ? "bg-blue-500/30 text-blue-300 border border-blue-500/40"
                      : "bg-zinc-800 text-zinc-400 border border-zinc-700 hover:border-zinc-600"
                  }`}
                >
                  .{ext}
                </button>
              ))}
            </div>
          </div>

          {/* Root folder filter */}
          {roots.length > 0 && (
            <div>
              <label className="text-xs text-zinc-400 mb-1.5 block">
                Folder
              </label>
              <select
                value={rootId ?? ""}
                onChange={(e) =>
                  onRootIdChange(
                    e.target.value ? Number(e.target.value) : undefined
                  )
                }
                className="w-full px-2 py-1.5 bg-zinc-800 border border-zinc-700 rounded text-xs text-zinc-200 focus:outline-none focus:border-blue-500"
              >
                <option value="">All folders</option>
                {roots.map((root) => (
                  <option key={root.id} value={root.id}>
                    {root.path}
                  </option>
                ))}
              </select>
            </div>
          )}
        </div>
      )}
    </div>
  );
}
