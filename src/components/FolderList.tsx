import { useEffect, useState } from "react";
import { FolderPlus, Trash2, Play, Folder } from "lucide-react";
import { open } from "@tauri-apps/plugin-dialog";
import {
  listIndexedFolders,
  addIndexedFolder,
  removeIndexedFolder,
  startScan,
  type IndexedRoot,
} from "../lib/commands";

export function FolderList() {
  const [roots, setRoots] = useState<IndexedRoot[]>([]);
  const [loading, setLoading] = useState(false);

  const loadRoots = async () => {
    try {
      const folders = await listIndexedFolders();
      setRoots(folders);
    } catch (e) {
      console.error("Failed to load folders:", e);
    }
  };

  useEffect(() => {
    loadRoots();
  }, []);

  const handleAddFolder = async () => {
    try {
      const selected = await open({ directory: true, multiple: false });
      if (selected) {
        setLoading(true);
        const root = await addIndexedFolder(selected);
        setRoots((prev) => [root, ...prev]);
        // Auto-start scan for new folder
        await startScan(root.id);
      }
    } catch (e) {
      console.error("Failed to add folder:", e);
    } finally {
      setLoading(false);
    }
  };

  const handleRemoveFolder = async (id: number) => {
    try {
      await removeIndexedFolder(id);
      setRoots((prev) => prev.filter((r) => r.id !== id));
    } catch (e) {
      console.error("Failed to remove folder:", e);
    }
  };

  const handleScanFolder = async (id: number) => {
    try {
      await startScan(id);
    } catch (e) {
      console.error("Failed to start scan:", e);
    }
  };

  return (
    <div className="flex flex-col gap-3">
      <div className="flex items-center justify-between">
        <h3 className="text-sm font-medium text-zinc-200">Indexed Folders</h3>
        <button
          onClick={handleAddFolder}
          disabled={loading}
          className="flex items-center gap-1.5 px-3 py-1.5 bg-blue-600 hover:bg-blue-500 disabled:opacity-50 text-white rounded text-xs font-medium transition-colors"
        >
          <FolderPlus className="w-3.5 h-3.5" />
          Add Folder
        </button>
      </div>

      {roots.length === 0 ? (
        <div className="flex flex-col items-center gap-2 py-8 text-zinc-500">
          <Folder className="w-10 h-10 text-zinc-600" />
          <p className="text-sm">No folders indexed yet</p>
          <p className="text-xs">Add a folder to start searching your files</p>
        </div>
      ) : (
        <div className="flex flex-col gap-2">
          {roots.map((root) => (
            <div
              key={root.id}
              className="flex items-center gap-2 p-3 bg-zinc-800/50 border border-zinc-800 rounded-lg"
            >
              <Folder className="w-4 h-4 text-blue-400 shrink-0" />
              <span className="text-sm text-zinc-200 truncate flex-1">
                {root.path}
              </span>
              <button
                onClick={() => handleScanFolder(root.id)}
                className="p-1.5 rounded hover:bg-zinc-700 text-zinc-400 hover:text-green-400 transition-colors"
                title="Re-scan this folder"
              >
                <Play className="w-3.5 h-3.5" />
              </button>
              <button
                onClick={() => handleRemoveFolder(root.id)}
                className="p-1.5 rounded hover:bg-zinc-700 text-zinc-400 hover:text-red-400 transition-colors"
                title="Remove folder"
              >
                <Trash2 className="w-3.5 h-3.5" />
              </button>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
