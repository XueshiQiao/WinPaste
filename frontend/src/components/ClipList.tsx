import { useRef } from 'react';
import { ClipboardItem } from '../types';
import { ClipCard } from './ClipCard';

interface ClipListProps {
  clips: ClipboardItem[];
  isLoading: boolean;
  hasMore: boolean;
  selectedClipId: string | null;
  onSelectClip: (clipId: string) => void;
  onPaste: (clipId: string) => void;
  onCopy: (clipId: string) => void;
  onDelete: (clipId: string) => void;
  onLoadMore: () => void;
  onDragStart: (clipId: string, startX: number, startY: number) => void;
  onCardContextMenu?: (e: React.MouseEvent, clipId: string) => void;
}

export function ClipList({
  clips,
  isLoading,
  hasMore,
  selectedClipId,
  onSelectClip,
  onPaste,
  onCopy,
  onLoadMore,
  onDragStart,
  onCardContextMenu,
}: ClipListProps) {
  const containerRef = useRef<HTMLDivElement>(null);

  // Native onScroll handler for infinite scroll
  const handleScroll = () => {
    if (!containerRef.current || !hasMore || isLoading) return;

    // We check native scroll properties
    const { scrollLeft, scrollWidth, clientWidth } = containerRef.current;

    // If scrolled within 300px of the end
    if (scrollLeft + clientWidth >= scrollWidth - 300) {
      onLoadMore();
    }
  };

  // Map vertical mouse wheel to horizontal scroll for better UX
  const handleWheel = (e: React.WheelEvent) => {
    if (containerRef.current && e.deltaY !== 0) {
      // Multiply by 2 for faster scrolling as requested
      containerRef.current.scrollLeft += e.deltaY * 1;
    }
  };

  if (isLoading && clips.length === 0) {
    return (
      <div className="flex h-full w-full items-center justify-center">
        <div className="flex flex-col items-center gap-3">
          <div className="h-8 w-8 animate-spin rounded-full border-2 border-primary/30 border-t-primary" />
          <p className="text-sm text-muted-foreground">Loading clips...</p>
        </div>
      </div>
    );
  }

  if (clips.length === 0) {
    return (
      <div className="flex h-full w-full flex-col items-center justify-center p-8 text-center">
        <h3 className="mb-2 text-lg font-semibold text-gray-400">No clips yet</h3>
        <p className="max-w-xs text-sm text-gray-500">
          Copy something to your clipboard and it will appear here.
        </p>
      </div>
    );
  }

  return (
    <div
      ref={containerRef}
      className="no-scrollbar flex h-full w-full flex-1 items-center gap-4 overflow-x-auto overflow-y-hidden px-4"
      onScroll={handleScroll}
      onWheel={handleWheel}
      style={{
        // Smooth scrolling disabled per user request
        scrollBehavior: 'auto',
      }}
    >
      {clips.map((clip) => (
        <ClipCard
          key={clip.id}
          clip={clip}
          isSelected={selectedClipId === clip.id}
          onSelect={() => onSelectClip(clip.id)}
          onPaste={() => onPaste(clip.id)}
          onCopy={() => onCopy(clip.id)}
          onDragStart={onDragStart}
          onContextMenu={(e: React.MouseEvent) => onCardContextMenu?.(e, clip.id)}
        />
      ))}

      {/* Loading indicator at the end */}
      {isLoading && clips.length > 0 && (
        <div className="flex h-full min-w-[100px] items-center justify-center">
          <div className="h-6 w-6 animate-spin rounded-full border-2 border-primary/30 border-t-primary" />
        </div>
      )}

      {/* Spacer end */}
      <div className="h-full min-w-[20px] flex-shrink-0" />
    </div>
  );
}
