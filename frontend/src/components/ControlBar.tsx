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
  onFolderContextMenu?: (e: React.MouseEvent, folderId: string) => void;
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
  onFolderContextMenu,
}: ControlBarProps) {
  const allCategories = [
    { id: null, name: 'All', count: totalClipCount },
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

  const getFolderColor = (name: string) => {
    let hash = 0;
    for (let i = 0; i < name.length; i++) {
      hash = name.charCodeAt(i) + ((hash << 5) - hash);
    }
    const colors = [
      {
        active: 'bg-red-400/30 text-red-400 ring-2 ring-red-500/50 font-bold',
        inactive: 'bg-red-400/10 text-red-400/80 hover:bg-red-400/20 hover:text-red-400',
      },
      {
        active: 'bg-orange-400/30 text-orange-400 ring-2 ring-orange-500/50 font-bold',
        inactive:
          'bg-orange-400/10 text-orange-400/80 hover:bg-orange-400/20 hover:text-orange-400',
      },
      {
        active: 'bg-amber-400/30 text-amber-400 ring-2 ring-amber-500/50 font-bold',
        inactive: 'bg-amber-400/10 text-amber-400/80 hover:bg-amber-400/20 hover:text-amber-400',
      },
      {
        active: 'bg-green-400/30 text-green-400 ring-2 ring-green-500/50 font-bold',
        inactive: 'bg-green-400/10 text-green-400/80 hover:bg-green-400/20 hover:text-green-400',
      },
      {
        active: 'bg-emerald-400/30 text-emerald-400 ring-2 ring-emerald-500/50 font-bold',
        inactive:
          'bg-emerald-400/10 text-emerald-400/80 hover:bg-emerald-400/20 hover:text-emerald-400',
      },
      {
        active: 'bg-teal-400/30 text-teal-400 ring-2 ring-teal-500/50 font-bold',
        inactive: 'bg-teal-400/10 text-teal-400/80 hover:bg-teal-400/20 hover:text-teal-400',
      },
      {
        active: 'bg-cyan-400/30 text-cyan-400 ring-2 ring-cyan-500/50 font-bold',
        inactive: 'bg-cyan-400/10 text-cyan-400/80 hover:bg-cyan-400/20 hover:text-cyan-400',
      },
      {
        active: 'bg-sky-400/30 text-sky-400 ring-2 ring-sky-500/50 font-bold',
        inactive: 'bg-sky-400/10 text-sky-400/80 hover:bg-sky-400/20 hover:text-sky-400',
      },
      {
        active: 'bg-blue-400/30 text-blue-400 ring-2 ring-blue-500/50 font-bold',
        inactive: 'bg-blue-400/10 text-blue-400/80 hover:bg-blue-400/20 hover:text-blue-400',
      },
      {
        active: 'bg-indigo-400/30 text-indigo-400 ring-2 ring-indigo-500/50 font-bold',
        inactive:
          'bg-indigo-400/10 text-indigo-400/80 hover:bg-indigo-400/20 hover:text-indigo-400',
      },
      {
        active: 'bg-violet-400/30 text-violet-400 ring-2 ring-violet-500/50 font-bold',
        inactive:
          'bg-violet-400/10 text-violet-400/80 hover:bg-violet-400/20 hover:text-violet-400',
      },
      {
        active: 'bg-purple-400/30 text-purple-400 ring-2 ring-purple-500/50 font-bold',
        inactive:
          'bg-purple-400/10 text-purple-400/80 hover:bg-purple-400/20 hover:text-purple-400',
      },
      {
        active: 'bg-fuchsia-400/30 text-fuchsia-400 ring-2 ring-fuchsia-500/50 font-bold',
        inactive:
          'bg-fuchsia-400/10 text-fuchsia-400/80 hover:bg-fuchsia-400/20 hover:text-fuchsia-400',
      },
      {
        active: 'bg-pink-400/30 text-pink-400 ring-2 ring-pink-500/50 font-bold',
        inactive: 'bg-pink-400/10 text-pink-400/80 hover:bg-pink-400/20 hover:text-pink-400',
      },
      {
        active: 'bg-rose-400/30 text-rose-400 ring-2 ring-rose-500/50 font-bold',
        inactive: 'bg-rose-400/10 text-rose-400/80 hover:bg-rose-400/20 hover:text-rose-400',
      },
    ];
    return colors[Math.abs(hash) % colors.length];
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
        {/** Search Render Code Omitted here for brevity, referencing original structure **/}
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
          let colorClass =
            'bg-secondary text-muted-foreground hover:bg-secondary/80 hover:text-foreground';

          if (cat.id === null) {
            // System "All" Folder
            if (isActive) {
              colorClass = 'bg-indigo-500/20 text-indigo-400 ring-1 ring-indigo-500/50 font-bold';
            } else {
              colorClass =
                'bg-indigo-500/10 text-indigo-400/80 hover:bg-indigo-500/20 hover:text-indigo-400';
            }
          } else {
            // Custom Folder - Use dynamic color
            const style = getFolderColor(cat.name);
            colorClass = isActive ? style.active : style.inactive;
          }

          return (
            <button
              key={cat.id ?? 'all'}
              onClick={() => onSelectFolder(cat.id)}
              onMouseEnter={() => handleMouseEnter(cat.id)}
              onMouseLeave={handleMouseLeave}
              onMouseUp={() => {
                // MouseUp logic is handled globally
              }}
              onContextMenu={(e) => {
                if (onFolderContextMenu && cat.id) {
                  onFolderContextMenu(e, cat.id);
                }
              }}
              style={{ WebkitAppRegion: 'no-drag' } as any}
              className={clsx(
                'whitespace-nowrap rounded-full px-4 py-1.5 text-sm font-medium transition-all',
                colorClass,
                isDragging && cat.id === dragTargetFolderId && 'bg-accent ring-2 ring-primary'
              )}
            >
              {cat.name}
              {/* Show count badge if defined and > 0 */}
              {cat.count !== undefined && cat.count > 0 && (
                <span className="ml-2 text-[10px] opacity-70">{cat.count}</span>
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
