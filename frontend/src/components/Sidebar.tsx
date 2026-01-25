import { useState } from "react";
import { FolderItem } from "../types";
import { Folder, Pin, Settings, Plus } from "lucide-react";
import { clsx } from "clsx";

interface SidebarProps {
  folders: FolderItem[];
  selectedFolder: string | null;
  onSelectFolder: (folderId: string | null) => void;
  onCreateFolder: (name: string) => void;
  onOpenSettings: () => void;
}

const SYSTEM_FOLDERS = [
  { id: null, name: "All", icon: Folder, is_system: true },
  { id: "pinned", name: "Pinned", icon: Pin, is_system: true },
];

export function Sidebar({
  folders,
  selectedFolder,
  onSelectFolder,
  onCreateFolder,
  onOpenSettings,
}: SidebarProps) {
  const [showNewFolderInput, setShowNewFolderInput] = useState(false);
  const [newFolderName, setNewFolderName] = useState("");

  const handleCreateFolder = () => {
    if (newFolderName.trim()) {
      onCreateFolder(newFolderName.trim());
      setNewFolderName("");
      setShowNewFolderInput(false);
    }
  };

  const customFolders = folders.filter((f) => !f.is_system);

  return (
    <aside className="w-64 flex-shrink-0 bg-sidebar border-r border-sidebar-border flex flex-col">
      <div className="p-4 drag-area cursor-move select-none">
        <h1 className="text-xl font-bold text-white select-none">
          WinPaste
        </h1>
      </div>

      <nav className="flex-1 px-3 space-y-1 overflow-y-auto">
        {SYSTEM_FOLDERS.map((folder) => (
          <button
            key={folder.id ?? "all"}
            onClick={() => onSelectFolder(folder.id)}
            className={clsx(
              "sidebar-item w-full",
              selectedFolder === folder.id && "active",
            )}
          >
            <folder.icon size={18} />
            <span className="font-medium">{folder.name}</span>
          </button>
        ))}

        <div className="pt-4 pb-2">
          <div className="flex items-center justify-between px-3 mb-2">
            <span className="text-xs font-semibold text-muted-foreground uppercase tracking-wider">
              Folders
            </span>
            <button
              onClick={() => setShowNewFolderInput(true)}
              className="icon-button p-1"
            >
              <Plus size={14} />
            </button>
          </div>

          {showNewFolderInput && (
            <div className="px-3 mb-2 animate-slide-up">
              <input
                type="text"
                value={newFolderName}
                onChange={(e) => setNewFolderName(e.target.value)}
                onKeyDown={(e) => {
                  if (e.key === "Enter") handleCreateFolder();
                  if (e.key === "Escape") {
                    setShowNewFolderInput(false);
                    setNewFolderName("");
                  }
                }}
                onBlur={() => {
                  if (!newFolderName.trim()) {
                    setShowNewFolderInput(false);
                  }
                }}
                placeholder="Folder name"
                className="search-input text-sm py-1.5"
                autoFocus
              />
            </div>
          )}

          {customFolders.map((folder) => (
            <button
              key={folder.id}
              onClick={() => onSelectFolder(folder.id)}
              className={clsx(
                "sidebar-item w-full",
                selectedFolder === folder.id && "active",
              )}
            >
              <Folder size={18} />
              <span className="font-medium truncate">{folder.name}</span>
              <span className="ml-auto text-xs text-muted-foreground">
                {folder.item_count}
              </span>
            </button>
          ))}
        </div>
      </nav>

      <div className="p-3 border-t border-sidebar-border space-y-1">
        <button onClick={onOpenSettings} className="sidebar-item w-full">
          <Settings size={18} />
          <span className="font-medium">Settings</span>
        </button>
      </div>
    </aside>
  );
}
