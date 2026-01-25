import { FolderItem } from "../types";
import { Search, Plus, MoreHorizontal } from "lucide-react";
import { clsx } from "clsx";

interface ControlBarProps {
  folders: FolderItem[];
  selectedFolder: string | null;
  onSelectFolder: (folderId: string | null) => void;
  onSearchClick: () => void;
  onAddClick: () => void;
  onMoreClick: () => void;
}

export function ControlBar({
  folders,
  selectedFolder,
  onSelectFolder,
  onSearchClick,
  onAddClick,
  onMoreClick,
}: ControlBarProps) {
  // Merge "All" (null), "Pinned" (special), and user folders
  const allCategories = [
    { id: null, name: "Clipboard History" },
    { id: "pinned", name: "Pinned" },
    ...folders.filter((f) => !f.is_system),
  ];

  return (
    <div className="flex items-center gap-4 p-4 border-b border-gray-800 bg-gray-950/90 drag-area">
      {/* Search Icon */}
      <button
        onClick={onSearchClick}
        className="text-gray-400 hover:text-white transition-colors"
      >
        <Search size={20} />
      </button>

      {/* Category Pills */}
      <div className="flex-1 flex items-center gap-2 overflow-x-auto no-scrollbar mask-gradient-right">
        {allCategories.map((cat) => {
          const isActive = selectedFolder === cat.id;
          return (
            <button
              key={cat.id ?? "all"}
              onClick={() => onSelectFolder(cat.id)}
              className={clsx(
                "px-4 py-1.5 rounded-full text-sm font-medium whitespace-nowrap transition-all",
                isActive
                  ? "bg-purple-600/20 text-purple-400 ring-1 ring-purple-500/50"
                  : "bg-gray-800/50 text-gray-400 hover:bg-gray-800 hover:text-gray-300"
              )}
            >
              {cat.name}
              {/* Blue dot indicator for specific categories if needed (example logic) */}
              {cat.name === "JSON" && (
                <span className="ml-2 inline-block w-1.5 h-1.5 bg-blue-500 rounded-full" />
              )}
            </button>
          );
        })}
      </div>

      {/* Actions */}
      <div className="flex items-center gap-2">
        <button
          onClick={onAddClick}
          className="p-2 text-gray-400 hover:text-white hover:bg-gray-800 rounded-lg transition-colors"
        >
          <Plus size={20} />
        </button>
        <button
          onClick={onMoreClick}
          className="p-2 text-gray-400 hover:text-white hover:bg-gray-800 rounded-lg transition-colors"
        >
          <MoreHorizontal size={20} />
        </button>
      </div>
    </div>
  );
}
