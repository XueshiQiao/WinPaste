import { FolderItem } from "../types";
import { Search, Plus, MoreHorizontal, X } from "lucide-react";
import { clsx } from "clsx";

interface ControlBarProps {
  folders: FolderItem[];
  selectedFolder: string | null;
  onSelectFolder: (folderId: string | null) => void;
  onSearchClick: () => void;
  onAddClick: () => void;
  onMoreClick: () => void;
  showSearch: boolean;
  searchQuery: string;
  onSearchChange: (query: string) => void;
}

export function ControlBar({
  folders,
  selectedFolder,
  onSelectFolder,
  onSearchClick,
  onAddClick,
  onMoreClick,
  showSearch,
  searchQuery,
  onSearchChange,
}: ControlBarProps) {
  // Merge "All" (null), "Pinned" (special), and user folders
  const allCategories = [
    { id: null, name: "Clipboard History" },
    { id: "pinned", name: "Pinned" },
    ...folders.filter((f) => !f.is_system),
  ];

  return (
    <div className="flex items-center gap-4 px-6 py-4 border-b border-border bg-background/90 drag-area min-h-[73px]">
      {/* Search Toggle / Input */}
      <div className={clsx("flex items-center transition-all duration-300", showSearch ? "w-[300px]" : "w-10")}>
        {showSearch ? (
          <div className="w-full flex items-center gap-2 bg-input border border-border rounded-full px-3 py-1.5 animate-in fade-in slide-in-from-left-2 duration-300">
            <Search size={18} className="text-muted-foreground" />
            <input
              autoFocus
              type="text"
              value={searchQuery}
              onChange={(e) => onSearchChange(e.target.value)}
              placeholder="Search clips..."
              className="flex-1 bg-transparent border-none outline-none text-sm text-foreground placeholder:text-muted-foreground"
              onKeyDown={(e) => {
                if (e.key === "Escape") {
                  e.preventDefault();
                  onSearchClick();
                }
              }}
            />
            <button
              onClick={onSearchClick}
              className="p-1 text-muted-foreground hover:text-foreground rounded-full hover:bg-accent transition-colors"
            >
              <X size={16} />
            </button>
          </div>
        ) : (
          <button
            onClick={onSearchClick}
            className="p-2 text-muted-foreground hover:text-foreground hover:bg-accent rounded-lg transition-colors"
          >
            <Search size={20} />
          </button>
        )}
      </div>

      {/* Category Pills (Always visible) */}
      <div className="flex-1 flex items-center gap-2 overflow-x-auto no-scrollbar mask-gradient-right p-1">
        {allCategories.map((cat) => {
            const isActive = selectedFolder === cat.id;
            return (
              <button
                key={cat.id ?? "all"}
                onClick={() => onSelectFolder(cat.id)}
                className={clsx(
                  "px-4 py-1.5 rounded-full text-sm font-medium whitespace-nowrap transition-all",
                  isActive
                    ? "bg-primary/20 text-primary ring-1 ring-primary/50 relative z-10"
                    : "bg-secondary text-muted-foreground hover:bg-secondary/80 hover:text-foreground"
                )}
              >
                {cat.name}
                {cat.name === "JSON" && (
                  <span className="ml-2 inline-block w-1.5 h-1.5 bg-blue-500 rounded-full" />
                )}
              </button>
            );
          })}
      </div>

      {/* Actions */}
      <div className="flex items-center gap-2 flex-shrink-0">
        <button
          onClick={onAddClick}
          className="p-2 text-muted-foreground hover:text-foreground hover:bg-accent rounded-lg transition-colors"
        >
          <Plus size={20} />
        </button>
        <button
          onClick={onMoreClick}
          className="p-2 text-muted-foreground hover:text-foreground hover:bg-accent rounded-lg transition-colors"
        >
          <MoreHorizontal size={20} />
        </button>
      </div>
    </div>
  );
}
