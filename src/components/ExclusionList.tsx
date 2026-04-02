import { useEffect, useState } from "react";
import { Plus, Trash2, ShieldOff } from "lucide-react";
import {
  listExclusions,
  addExclusion,
  removeExclusion,
  type ExcludedPath,
} from "../lib/commands";

export function ExclusionList() {
  const [exclusions, setExclusions] = useState<ExcludedPath[]>([]);
  const [newPattern, setNewPattern] = useState("");

  const loadExclusions = async () => {
    try {
      const list = await listExclusions();
      setExclusions(list);
    } catch (e) {
      console.error("Failed to load exclusions:", e);
    }
  };

  useEffect(() => {
    loadExclusions();
  }, []);

  const handleAdd = async () => {
    if (!newPattern.trim()) return;
    try {
      const ex = await addExclusion(newPattern.trim());
      setExclusions((prev) => [...prev, ex]);
      setNewPattern("");
    } catch (e) {
      console.error("Failed to add exclusion:", e);
    }
  };

  const handleRemove = async (id: number) => {
    try {
      await removeExclusion(id);
      setExclusions((prev) => prev.filter((e) => e.id !== id));
    } catch (e) {
      console.error("Failed to remove exclusion:", e);
    }
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === "Enter") handleAdd();
  };

  return (
    <div className="flex flex-col gap-3">
      <h3 className="text-sm font-medium text-zinc-200">Exclusion Patterns</h3>
      <p className="text-xs text-zinc-500">
        Folders matching these patterns will be skipped during indexing.
      </p>

      {/* Add new pattern */}
      <div className="flex items-center gap-2">
        <input
          type="text"
          value={newPattern}
          onChange={(e) => setNewPattern(e.target.value)}
          onKeyDown={handleKeyDown}
          placeholder="e.g., node_modules, .git, build"
          className="flex-1 px-3 py-1.5 bg-zinc-800 border border-zinc-700 rounded text-sm text-zinc-200 placeholder-zinc-500 focus:outline-none focus:border-blue-500"
        />
        <button
          onClick={handleAdd}
          disabled={!newPattern.trim()}
          className="flex items-center gap-1.5 px-3 py-1.5 bg-zinc-700 hover:bg-zinc-600 disabled:opacity-40 text-zinc-200 rounded text-xs font-medium transition-colors"
        >
          <Plus className="w-3.5 h-3.5" />
          Add
        </button>
      </div>

      {/* Exclusion list */}
      {exclusions.length === 0 ? (
        <div className="flex items-center gap-2 py-4 text-zinc-500 justify-center">
          <ShieldOff className="w-5 h-5" />
          <span className="text-sm">No exclusions configured</span>
        </div>
      ) : (
        <div className="flex flex-wrap gap-2">
          {exclusions.map((ex) => (
            <div
              key={ex.id}
              className="flex items-center gap-1.5 px-2.5 py-1 bg-zinc-800 border border-zinc-700 rounded-full text-xs text-zinc-300"
            >
              <span className="font-mono">{ex.pattern}</span>
              <button
                onClick={() => handleRemove(ex.id)}
                className="text-zinc-500 hover:text-red-400 transition-colors"
              >
                <Trash2 className="w-3 h-3" />
              </button>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
