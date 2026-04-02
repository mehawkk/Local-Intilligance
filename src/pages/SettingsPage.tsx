import { RefreshCw } from "lucide-react";
import { FolderList } from "../components/FolderList";
import { ExclusionList } from "../components/ExclusionList";
import { triggerRescan } from "../lib/commands";
import { useState } from "react";

export function SettingsPage() {
  const [rescanning, setRescanning] = useState(false);

  const handleRescan = async () => {
    try {
      setRescanning(true);
      await triggerRescan();
    } catch (e) {
      console.error("Rescan failed:", e);
    } finally {
      setRescanning(false);
    }
  };

  return (
    <div className="flex flex-col h-full p-6 gap-8 overflow-y-auto max-w-3xl">
      <div>
        <h2 className="text-lg font-semibold text-zinc-100 mb-1">Settings</h2>
        <p className="text-sm text-zinc-500">
          Manage your indexed folders and exclusion patterns.
        </p>
      </div>

      <FolderList />

      <hr className="border-zinc-800" />

      <ExclusionList />

      <hr className="border-zinc-800" />

      {/* Rescan All */}
      <div className="flex flex-col gap-2">
        <h3 className="text-sm font-medium text-zinc-200">Full Rescan</h3>
        <p className="text-xs text-zinc-500">
          Re-scan all indexed folders. This will re-check every file and update
          the index for any changes. Useful after changing exclusion patterns.
        </p>
        <button
          onClick={handleRescan}
          disabled={rescanning}
          className="flex items-center gap-1.5 px-4 py-2 bg-zinc-700 hover:bg-zinc-600 disabled:opacity-50 text-zinc-200 rounded text-sm font-medium transition-colors w-fit"
        >
          <RefreshCw
            className={`w-4 h-4 ${rescanning ? "animate-spin" : ""}`}
          />
          {rescanning ? "Starting rescan..." : "Rescan All Folders"}
        </button>
      </div>
    </div>
  );
}
