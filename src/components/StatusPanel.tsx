import { useEffect, useState } from "react";
import { Loader2, AlertCircle, CheckCircle2 } from "lucide-react";
import { getIndexStatus, type IndexStatus } from "../lib/commands";
import { onIndexProgress, onIndexComplete } from "../lib/events";

export function StatusPanel() {
  const [status, setStatus] = useState<IndexStatus | null>(null);

  useEffect(() => {
    // Poll initial status
    getIndexStatus().then(setStatus).catch(console.error);

    // Listen for progress updates
    const unlistenProgress = onIndexProgress(setStatus);
    const unlistenComplete = onIndexComplete(setStatus);

    return () => {
      unlistenProgress.then((fn) => fn());
      unlistenComplete.then((fn) => fn());
    };
  }, []);

  if (!status) return null;

  if (status.is_running) {
    const pct =
      status.total_files > 0
        ? Math.round((status.processed_files / status.total_files) * 100)
        : 0;

    return (
      <div className="flex items-center gap-2 text-xs text-zinc-400">
        <Loader2 className="w-3.5 h-3.5 animate-spin text-blue-400" />
        <span>
          Indexing: {status.processed_files}/{status.total_files} ({pct}%)
        </span>
        {status.failed_files > 0 && (
          <span className="flex items-center gap-1 text-amber-400">
            <AlertCircle className="w-3 h-3" />
            {status.failed_files} errors
          </span>
        )}
        {/* Progress bar */}
        <div className="w-24 h-1.5 bg-zinc-700 rounded-full overflow-hidden">
          <div
            className="h-full bg-blue-500 rounded-full transition-all duration-300"
            style={{ width: `${pct}%` }}
          />
        </div>
      </div>
    );
  }

  if (status.processed_files > 0) {
    return (
      <div className="flex items-center gap-1.5 text-xs text-zinc-500">
        <CheckCircle2 className="w-3.5 h-3.5 text-green-500" />
        <span>{status.processed_files} files indexed</span>
      </div>
    );
  }

  return null;
}
