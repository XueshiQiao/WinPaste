import { FolderItem } from '../types';
import { Search, Plus, MoreHorizontal, X } from 'lucide-react';
import { clsx } from 'clsx';

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
  onMoveClip: (clipId: string, folderId: string | null) => void;
  isDragging: boolean;
  dragTargetFolderId: string | null;
  onDragHover: (folderId: string | null) => void;
  onDragLeave: () => void;
  totalClipCount: number;
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
  isDragging,
  dragTargetFolderId,
  onDragHover,
  onDragLeave,
  totalClipCount,
}: ControlBarProps) {
  // Merge "All" (null), "Pinned" (special), and user folders
  const allCategories = [
    { id: null, name: 'Clipboard History', count: totalClipCount },
    { id: 'pinned', name: 'Pinned' },
    ...folders.map((f) => ({ ...f, count: f.item_count })),
  ];

  const handleMouseEnter = (folderId: string | null) => {
    if (isDragging) {
      onDragHover(folderId);
    }
  };

  const handleMouseLeave = () => {
    onDragLeave();
  };


  return (
    <div className="drag-area flex min-h-[52px] items-center gap-4 border-b border-border bg-background/90 px-6 py-2">
      {/* Search Toggle / Input */}
      <div
        className={clsx(
          'no-drag flex items-center transition-all duration-300',
          showSearch ? 'w-[300px]' : 'w-10'
        )}
      >
        {showSearch ? (
          <div className="animate-in fade-in slide-in-from-left-2 flex w-full items-center gap-2 rounded-full border border-border bg-input px-3 py-1.5 duration-300">
            <Search size={18} className="text-blue-400" />
            <input
              autoFocus
              type="text"
              value={searchQuery}
              onChange={(e) => onSearchChange(e.target.value)}
              placeholder="Search clips..."
              className="flex-1 border-none bg-transparent text-sm text-foreground outline-none placeholder:text-muted-foreground"
              onKeyDown={(e) => {
                if (e.key === 'Escape') {
                  e.preventDefault();
                  onSearchClick();
                }
              }}
            />
            <button
              onClick={onSearchClick}
              className="rounded-full p-1 text-muted-foreground transition-colors hover:bg-accent hover:text-foreground"
            >
              <X size={16} />
            </button>
          </div>
        ) : (
          <button
            onClick={onSearchClick}
            className="rounded-lg p-2 text-blue-400 transition-colors hover:bg-blue-500/10"
          >
            <Search size={20} />
          </button>
        )}
      </div>

      {/* Category Pills (Always visible) */}
      <div
        className="no-scrollbar mask-gradient-right flex flex-1 items-center gap-2 overflow-x-auto p-1"
        style={{ WebkitAppRegion: 'no-drag' } as any}
      >
        {allCategories.map((cat) => {
          const isActive = selectedFolder === cat.id;

          // Define colors based on category
          let activeClass = 'bg-primary/20 text-primary ring-1 ring-primary/50';
          if (cat.id === null)
            activeClass = 'bg-indigo-500/20 text-indigo-400 ring-1 ring-indigo-500/50';
          else if (cat.id === 'pinned')
            activeClass = 'bg-pink-500/20 text-pink-400 ring-1 ring-pink-500/50';
          else if (isActive) activeClass = 'bg-cyan-500/20 text-cyan-400 ring-1 ring-cyan-500/50';

          return (
            <button
              key={cat.id ?? 'all'}
              onClick={() => onSelectFolder(cat.id)}
              onMouseEnter={() => handleMouseEnter(cat.id)}
              onMouseLeave={handleMouseLeave}
              onMouseUp={() => {
                 // MouseUp logic is handled globally in App.tsx, checking valid hover target
              }}
              style={{ WebkitAppRegion: 'no-drag' } as any}
              className={clsx(
                'whitespace-nowrap rounded-full px-4 py-1.5 text-sm font-medium transition-all',
                isActive
                  ? activeClass
                  : isDragging && cat.id === dragTargetFolderId // Only highlight if it matches the current drag target
                    ? 'ring-2 ring-primary bg-accent' // Show highlight
                    : 'bg-secondary text-muted-foreground hover:bg-secondary/80 hover:text-foreground'
              )}
            >
              {cat.name}
              {/* Show count badge if defined and > 0 */}
              {(cat.count !== undefined && cat.count > 0) && (
                <span className="ml-2 text-[10px] opacity-70">
                    {cat.count}
                </span>
              )}
            </button>
          );
        })}
      </div>

      {/* Actions */}
      <div className="flex flex-shrink-0 items-center gap-2">
        <button
          onClick={onAddClick}
          className="rounded-lg p-2 text-emerald-400 transition-colors hover:bg-emerald-500/10"
        >
          <Plus size={20} />
        </button>
        <button
          onClick={onMoreClick}
          className="rounded-lg p-2 text-amber-400 transition-colors hover:bg-amber-500/10"
        >
          <MoreHorizontal size={20} />
        </button>
      </div>
    </div>
  );
}
