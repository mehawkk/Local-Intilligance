import { Search, Settings, HardDrive } from "lucide-react";
import { NavLink, Outlet } from "react-router-dom";
import { StatusPanel } from "./StatusPanel";

export function Layout() {
  return (
    <div className="flex flex-col h-screen bg-zinc-950 text-zinc-100">
      {/* Top Navigation Bar */}
      <header className="flex items-center gap-1 px-4 py-2 bg-zinc-900 border-b border-zinc-800 shrink-0">
        <HardDrive className="w-5 h-5 text-blue-400 mr-2" />
        <span className="font-semibold text-sm mr-6">Local File Intelligence</span>

        <NavLink
          to="/"
          className={({ isActive }) =>
            `flex items-center gap-1.5 px-3 py-1.5 rounded text-sm transition-colors ${
              isActive
                ? "bg-zinc-700 text-white"
                : "text-zinc-400 hover:text-zinc-200 hover:bg-zinc-800"
            }`
          }
        >
          <Search className="w-4 h-4" />
          Search
        </NavLink>

        <NavLink
          to="/settings"
          className={({ isActive }) =>
            `flex items-center gap-1.5 px-3 py-1.5 rounded text-sm transition-colors ${
              isActive
                ? "bg-zinc-700 text-white"
                : "text-zinc-400 hover:text-zinc-200 hover:bg-zinc-800"
            }`
          }
        >
          <Settings className="w-4 h-4" />
          Settings
        </NavLink>

        <div className="flex-1" />
        <StatusPanel />
      </header>

      {/* Main Content */}
      <main className="flex-1 overflow-hidden">
        <Outlet />
      </main>
    </div>
  );
}
