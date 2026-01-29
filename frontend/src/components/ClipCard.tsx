import { ClipboardItem } from '../types';
import { clsx } from 'clsx';
import { useMemo, memo } from 'react';
import { LAYOUT, TOTAL_COLUMN_WIDTH, PREVIEW_CHAR_LIMIT } from '../constants';

interface ClipCardProps {
  clip: ClipboardItem;
  isSelected: boolean;
  onSelect: () => void;
  onPaste: () => void;
  onDragStart: (clipId: string, startX: number, startY: number) => void;
  onContextMenu?: (e: React.MouseEvent) => void;
}

export const ClipCard = memo(function ClipCard({
  clip,
  isSelected,
  onSelect,
  onPaste,
  onDragStart,
  onContextMenu,
}: ClipCardProps) {
  const title = clip.source_app || clip.clip_type.toUpperCase();

  // Memoize the content rendering
  const renderedContent = useMemo(() => {
    if (clip.clip_type === 'image') {
      return (
        <div className="flex h-full w-full select-none items-center justify-center">
          <img
            src={`data:image/png;base64,${clip.content}`}
            alt="Clipboard Image"
            className="max-h-full max-w-full object-contain"
          />
        </div>
      );
    } else {
      return (
        <pre className="whitespace-pre-wrap break-all font-mono text-[11px] leading-tight text-foreground">
          <span>{clip.content.substring(0, PREVIEW_CHAR_LIMIT)}</span>
        </pre>
      );
    }
  }, [clip.clip_type, clip.content]);

  // Generate distinct color based on source app name
  const getAppColor = (name: string) => {
    let hash = 0;
    for (let i = 0; i < name.length; i++) {
      hash = name.charCodeAt(i) + ((hash << 5) - hash);
    }
    const colors = [
      'bg-red-400',
      'bg-orange-400',
      'bg-amber-400',
      'bg-green-400',
      'bg-emerald-400',
      'bg-teal-400',
      'bg-cyan-400',
      'bg-sky-400',
      'bg-blue-400',
      'bg-indigo-400',
      'bg-violet-400',
      'bg-purple-400',
      'bg-fuchsia-400',
      'bg-pink-400',
      'bg-rose-400',
    ];
    return colors[Math.abs(hash) % colors.length];
  };

  const headerColor = getAppColor(title);

  const handleMouseDown = (e: React.MouseEvent) => {
    // Only left click
    if (e.button !== 0) return;
    onDragStart(clip.id, e.clientX, e.clientY);
  };

  const handleContextMenu = (e: React.MouseEvent) => {
    e.preventDefault();
    onContextMenu?.(e);
  };

  return (
    <div
      style={{
        width: TOTAL_COLUMN_WIDTH - LAYOUT.CARD_GAP,
        height: LAYOUT.WINDOW_HEIGHT - LAYOUT.CONTROL_BAR_HEIGHT - LAYOUT.CARD_VERTICAL_PADDING * 2,
      }}
      className="flex-shrink-0"
    >
      <div
        onMouseDown={handleMouseDown}
        onClick={onSelect}
        onDoubleClick={onPaste}
        onContextMenu={handleContextMenu}
        className={clsx(
          'relative flex h-full w-full cursor-pointer select-none flex-col overflow-hidden rounded-xl border border-border bg-card shadow-lg transition-all',
          isSelected
            ? 'z-10 scale-[1.02] transform ring-4 ring-blue-500'
            : 'hover:-translate-y-1 hover:ring-2 hover:ring-primary/30'
        )}
      >
        <div className={clsx(headerColor, 'flex flex-shrink-0 items-center gap-2 px-4 py-2')}>
          {clip.source_icon && (
            <img
              src={`data:image/png;base64,${clip.source_icon}`}
              alt=""
              className="h-4 w-4 object-contain"
            />
          )}
          <span className="flex-1 truncate text-[10px] font-bold uppercase tracking-wider text-white shadow-sm">
            {title}
          </span>
        </div>

        <div className="relative flex-1 overflow-hidden bg-card p-3">
          {renderedContent}
          <div className="pointer-events-none absolute bottom-0 left-0 right-0 h-12 bg-gradient-to-t from-card to-transparent" />
        </div>

        <div className="flex-shrink-0 border-t border-border bg-secondary px-3 py-1.5">
          <span className="text-[10px] font-medium text-muted-foreground">
            {clip.clip_type === 'image'
              ? `Image (${Math.round((clip.content.length * 0.75) / 1024)}KB)`
              : `${clip.content.length} characters`}
          </span>
        </div>
      </div>
    </div>
  );
});
