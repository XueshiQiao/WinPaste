import { useState } from 'react';
import { FolderItem } from '../types';
import { Folder, Settings, Plus } from 'lucide-react';
import { clsx } from 'clsx';

interface SidebarProps {
  folders: FolderItem[];
  selectedFolder: string | null;
  onSelectFolder: (folderId: string | null) => void;
  onCreateFolder: (name: string) => void;
  onOpenSettings: () => void;
}

export function Sidebar({
  folders,
  selectedFolder,
  onSelectFolder,
  onCreateFolder,
  onOpenSettings,
}: SidebarProps) {
  const [showNewFolderInput, setShowNewFolderInput] = useState(false);
  const [newFolderName, setNewFolderName] = useState('');

  const handleCreateFolder = () => {
    if (newFolderName.trim()) {
      onCreateFolder(newFolderName.trim());
      setNewFolderName('');
      setShowNewFolderInput(false);
    }
  };

  const customFolders = folders.filter((f) => !f.is_system);

  return (
    <aside className="flex w-64 flex-shrink-0 flex-col border-r border-sidebar-border bg-sidebar">
      <div className="drag-area cursor-move select-none p-4">
        <h1 className="select-none text-xl font-bold text-white">WinPaste</h1>
      </div>

      <nav className="flex-1 space-y-1 overflow-y-auto px-3">
        <div className="pb-2 pt-4">
          <div className="mb-2 flex items-center justify-between px-3">
            <span className="text-xs font-semibold uppercase tracking-wider text-muted-foreground">
              Folders
            </span>
            <button onClick={() => setShowNewFolderInput(true)} className="icon-button p-1">
              <Plus size={14} />
            </button>
          </div>

          {showNewFolderInput && (
            <div className="animate-slide-up mb-2 px-3">
              <input
                type="text"
                value={newFolderName}
                onChange={(e) => setNewFolderName(e.target.value)}
                onKeyDown={(e) => {
                  if (e.key === 'Enter') handleCreateFolder();
                  if (e.key === 'Escape') {
                    setShowNewFolderInput(false);
                    setNewFolderName('');
                  }
                }}
                onBlur={() => {
                  if (!newFolderName.trim()) {
                    setShowNewFolderInput(false);
                  }
                }}
                placeholder="Folder name"
                className="search-input py-1.5 text-sm"
                autoFocus
              />
            </div>
          )}

          {customFolders.map((folder) => (
            <button
              key={folder.id}
              onClick={() => onSelectFolder(folder.id)}
              className={clsx('sidebar-item w-full', selectedFolder === folder.id && 'active')}
            >
              <Folder size={18} />
              <span className="truncate font-medium">{folder.name}</span>
              <span className="ml-auto text-xs text-muted-foreground">{folder.item_count}</span>
            </button>
          ))}
        </div>
      </nav>

      <div className="space-y-1 border-t border-sidebar-border p-3">
        <button onClick={onOpenSettings} className="sidebar-item w-full">
          <Settings size={18} />
          <span className="font-medium">Settings</span>
        </button>
      </div>
    </aside>
  );
}
