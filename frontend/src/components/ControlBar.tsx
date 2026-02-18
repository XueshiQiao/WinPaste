import { FolderItem } from '../types';
import { Search, Plus, MoreHorizontal, X } from 'lucide-react';
import { clsx } from 'clsx';
import { useTranslation } from 'react-i18next';

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
  theme?: 'light' | 'dark';
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
  theme = 'dark',
}: ControlBarProps) {
  const { t } = useTranslation();

  const allCategories = [
    { id: null, name: t('folders.all'), count: totalClipCount },
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
    const colors =
      theme === 'light'
        ? [
            {
              active: 'bg-red-600 text-white ring-2 ring-red-500/50 font-bold drop-shadow-sm',
              inactive: 'bg-red-400 text-white hover:bg-red-500 hover:text-white drop-shadow-sm',
            },
            {
              active: 'bg-orange-600 text-white ring-2 ring-orange-500/50 font-bold drop-shadow-sm',
              inactive:
                'bg-orange-400 text-white hover:bg-orange-500 hover:text-white drop-shadow-sm',
            },
            {
              active: 'bg-amber-600 text-white ring-2 ring-amber-500/50 font-bold drop-shadow-sm',
              inactive:
                'bg-amber-400 text-white hover:bg-amber-500 hover:text-white drop-shadow-sm',
            },
            {
              active: 'bg-green-600 text-white ring-2 ring-green-500/50 font-bold drop-shadow-sm',
              inactive:
                'bg-green-400 text-white hover:bg-green-500 hover:text-white drop-shadow-sm',
            },
            {
              active:
                'bg-emerald-600 text-white ring-2 ring-emerald-500/50 font-bold drop-shadow-sm',
              inactive:
                'bg-emerald-400 text-white hover:bg-emerald-500 hover:text-white drop-shadow-sm',
            },
            {
              active: 'bg-teal-600 text-white ring-2 ring-teal-500/50 font-bold drop-shadow-sm',
              inactive: 'bg-teal-400 text-white hover:bg-teal-500 hover:text-white drop-shadow-sm',
            },
            {
              active: 'bg-cyan-600 text-white ring-2 ring-cyan-500/50 font-bold drop-shadow-sm',
              inactive: 'bg-cyan-400 text-white hover:bg-cyan-500 hover:text-white drop-shadow-sm',
            },
            {
              active: 'bg-sky-600 text-white ring-2 ring-sky-500/50 font-bold drop-shadow-sm',
              inactive: 'bg-sky-400 text-white hover:bg-sky-500 hover:text-white drop-shadow-sm',
            },
            {
              active: 'bg-blue-600 text-white ring-2 ring-blue-500/50 font-bold drop-shadow-sm',
              inactive: 'bg-blue-400 text-white hover:bg-blue-500 hover:text-white drop-shadow-sm',
            },
            {
              active: 'bg-indigo-600 text-white ring-2 ring-indigo-500/50 font-bold drop-shadow-sm',
              inactive:
                'bg-indigo-400 text-white hover:bg-indigo-500 hover:text-white drop-shadow-sm',
            },
            {
              active: 'bg-violet-600 text-white ring-2 ring-violet-500/50 font-bold drop-shadow-sm',
              inactive:
                'bg-violet-400 text-white hover:bg-violet-500 hover:text-white drop-shadow-sm',
            },
            {
              active: 'bg-purple-600 text-white ring-2 ring-purple-500/50 font-bold drop-shadow-sm',
              inactive:
                'bg-purple-400 text-white hover:bg-purple-500 hover:text-white drop-shadow-sm',
            },
            {
              active:
                'bg-fuchsia-600 text-white ring-2 ring-fuchsia-500/50 font-bold drop-shadow-sm',
              inactive:
                'bg-fuchsia-400 text-white hover:bg-fuchsia-500 hover:text-white drop-shadow-sm',
            },
            {
              active: 'bg-pink-600 text-white ring-2 ring-pink-500/50 font-bold drop-shadow-sm',
              inactive: 'bg-pink-400 text-white hover:bg-pink-500 hover:text-white drop-shadow-sm',
            },
            {
              active: 'bg-rose-600 text-white ring-2 ring-rose-500/50 font-bold drop-shadow-sm',
              inactive: 'bg-rose-400 text-white hover:bg-rose-500 hover:text-white drop-shadow-sm',
            },
          ]
        : [
            {
              active: 'bg-red-400/30 text-white ring-2 ring-red-500/50 font-bold drop-shadow-sm',
              inactive:
                'bg-red-400/10 text-white/80 hover:bg-red-400/20 hover:text-white drop-shadow-sm',
            },
            {
              active:
                'bg-orange-400/30 text-white ring-2 ring-orange-500/50 font-bold drop-shadow-sm',
              inactive:
                'bg-orange-400/10 text-white/80 hover:bg-orange-400/20 hover:text-white drop-shadow-sm',
            },
            {
              active:
                'bg-amber-400/30 text-white ring-2 ring-amber-500/50 font-bold drop-shadow-sm',
              inactive:
                'bg-amber-400/10 text-white/80 hover:bg-amber-400/20 hover:text-white drop-shadow-sm',
            },
            {
              active:
                'bg-green-400/30 text-white ring-2 ring-green-500/50 font-bold drop-shadow-sm',
              inactive:
                'bg-green-400/10 text-white/80 hover:bg-green-400/20 hover:text-white drop-shadow-sm',
            },
            {
              active:
                'bg-emerald-400/30 text-white ring-2 ring-emerald-500/50 font-bold drop-shadow-sm',
              inactive:
                'bg-emerald-400/10 text-white/80 hover:bg-emerald-400/20 hover:text-white drop-shadow-sm',
            },
            {
              active: 'bg-teal-400/30 text-white ring-2 ring-teal-500/50 font-bold drop-shadow-sm',
              inactive:
                'bg-teal-400/10 text-white/80 hover:bg-teal-400/20 hover:text-white drop-shadow-sm',
            },
            {
              active: 'bg-cyan-400/30 text-white ring-2 ring-cyan-500/50 font-bold drop-shadow-sm',
              inactive:
                'bg-cyan-400/10 text-white/80 hover:bg-cyan-400/20 hover:text-white drop-shadow-sm',
            },
            {
              active: 'bg-sky-400/30 text-white ring-2 ring-sky-500/50 font-bold drop-shadow-sm',
              inactive:
                'bg-sky-400/10 text-white/80 hover:bg-sky-400/20 hover:text-white drop-shadow-sm',
            },
            {
              active: 'bg-blue-400/30 text-white ring-2 ring-blue-500/50 font-bold drop-shadow-sm',
              inactive:
                'bg-blue-400/10 text-white/80 hover:bg-blue-400/20 hover:text-white drop-shadow-sm',
            },
            {
              active:
                'bg-indigo-400/30 text-white ring-2 ring-indigo-500/50 font-bold drop-shadow-sm',
              inactive:
                'bg-indigo-400/10 text-white/80 hover:bg-indigo-400/20 hover:text-white drop-shadow-sm',
            },
            {
              active:
                'bg-violet-400/30 text-white ring-2 ring-violet-500/50 font-bold drop-shadow-sm',
              inactive:
                'bg-violet-400/10 text-white/80 hover:bg-violet-400/20 hover:text-white drop-shadow-sm',
            },
            {
              active:
                'bg-purple-400/30 text-white ring-2 ring-purple-500/50 font-bold drop-shadow-sm',
              inactive:
                'bg-purple-400/10 text-white/80 hover:bg-purple-400/20 hover:text-white drop-shadow-sm',
            },
            {
              active:
                'bg-fuchsia-400/30 text-white ring-2 ring-fuchsia-500/50 font-bold drop-shadow-sm',
              inactive:
                'bg-fuchsia-400/10 text-white/80 hover:bg-fuchsia-400/20 hover:text-white drop-shadow-sm',
            },
            {
              active: 'bg-pink-400/30 text-white ring-2 ring-pink-500/50 font-bold drop-shadow-sm',
              inactive:
                'bg-pink-400/10 text-white/80 hover:bg-pink-400/20 hover:text-white drop-shadow-sm',
            },
            {
              active: 'bg-rose-400/30 text-white ring-2 ring-rose-500/50 font-bold drop-shadow-sm',
              inactive:
                'bg-rose-400/10 text-white/80 hover:bg-rose-400/20 hover:text-white drop-shadow-sm',
            },
          ];
    return colors[Math.abs(hash) % colors.length];
  };

  return (
    <div className="drag-area flex min-h-[44px] items-center gap-4 bg-background/90 px-6 pt-2">
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
              placeholder={t('common.search')}
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
            theme === 'light'
              ? 'bg-slate-400 text-white hover:bg-slate-500 hover:text-white shadow-sm'
              : 'bg-secondary text-white hover:bg-secondary/80 hover:text-white shadow-sm';

          if (cat.id === null) {
            // System "All" Folder
            if (theme === 'light') {
              colorClass = isActive
                ? 'bg-slate-600 text-white ring-1 ring-slate-500/50 font-bold shadow-sm'
                : 'bg-slate-400 text-white hover:bg-slate-500 hover:text-white shadow-sm';
            } else {
              colorClass = isActive
                ? 'bg-indigo-500/20 text-white ring-1 ring-indigo-500/50 font-bold shadow-sm'
                : 'bg-indigo-500/10 text-white/80 hover:bg-indigo-500/20 hover:text-white shadow-sm';
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
              style={
                {
                  WebkitAppRegion: 'no-drag',
                  textShadow:
                    theme === 'light' ? '0 1px 3px rgba(0,0,0,0.8)' : '0 1px 2px rgba(0,0,0,0.7)',
                } as any
              }
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
      <div 
        className="flex flex-shrink-0 items-center gap-2"
        style={{ WebkitAppRegion: 'no-drag' } as any}
        onDoubleClick={(e) => e.stopPropagation()}
      >
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
