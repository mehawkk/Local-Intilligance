import {
  FileText,
  FolderOpen,
  ExternalLink,
} from "lucide-react";
import type { SearchResult } from "../lib/commands";
import { openFile, openContainingFolder } from "../lib/commands";
import { formatFileSize, formatDate, getExtensionColor } from "../lib/utils";

interface SearchResultItemProps {
  result: SearchResult;
}

export function SearchResultItem({ result }: SearchResultItemProps) {
  const handleOpenFile = async () => {
    try {
      await openFile(result.path);
    } catch (e) {
      console.error("Failed to open file:", e);
    }
  };

  const handleOpenFolder = async (e: React.MouseEvent) => {
    e.stopPropagation();
    try {
      await openContainingFolder(result.path);
    } catch (err) {
      console.error("Failed to open folder:", err);
    }
  };

  const extColor = getExtensionColor(result.extension);

  return (
    <div
      onClick={handleOpenFile}
      className="group flex flex-col gap-1.5 p-3 rounded-lg border border-zinc-800 hover:border-zinc-600 hover:bg-zinc-900/50 cursor-pointer transition-colors"
    >
      {/* Header row: icon, filename, extension badge, actions */}
      <div className="flex items-center gap-2">
        <FileText className="w-4 h-4 shrink-0 text-zinc-400" />
        <span className="font-medium text-sm text-zinc-100 truncate">
          {result.filename}
        </span>
        {result.extension && (
          <span
            className="px-1.5 py-0.5 rounded text-[10px] font-mono uppercase shrink-0"
            style={{
              color: extColor,
              backgroundColor: extColor + "18",
            }}
          >
            {result.extension}
          </span>
        )}
        <div className="flex-1" />
        {/* Actions (visible on hover) */}
        <div className="flex items-center gap-1 opacity-0 group-hover:opacity-100 transition-opacity">
          <button
            onClick={handleOpenFolder}
            className="p-1 rounded hover:bg-zinc-700 text-zinc-400 hover:text-zinc-200"
            title="Open containing folder"
          >
            <FolderOpen className="w-3.5 h-3.5" />
          </button>
          <button
            onClick={handleOpenFile}
            className="p-1 rounded hover:bg-zinc-700 text-zinc-400 hover:text-zinc-200"
            title="Open file"
          >
            <ExternalLink className="w-3.5 h-3.5" />
          </button>
        </div>
      </div>

      {/* Path */}
      <p className="text-xs text-zinc-500 truncate pl-6">{result.path}</p>

      {/* Snippet (if available) */}
      {result.snippet && result.snippet.trim() !== "" && (
        <div
          className="text-xs text-zinc-400 pl-6 line-clamp-2 [&_mark]:bg-yellow-500/30 [&_mark]:text-yellow-200 [&_mark]:rounded [&_mark]:px-0.5"
          dangerouslySetInnerHTML={{ __html: result.snippet }}
        />
      )}

      {/* Metadata row */}
      <div className="flex items-center gap-3 text-[11px] text-zinc-600 pl-6">
        <span>{formatFileSize(result.size_bytes)}</span>
        {result.modified_at_fs && (
          <span>{formatDate(result.modified_at_fs)}</span>
        )}
      </div>
    </div>
  );
}
