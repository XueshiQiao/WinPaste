import { ClipboardItem } from '../types';
import { clsx } from 'clsx';
import { useState, useRef, useEffect, CSSProperties } from 'react';
import { Grid, type CellComponentProps } from 'react-window';
import { LAYOUT, TOTAL_COLUMN_WIDTH } from '../constants';

interface ClipListProps {
  clips: ClipboardItem[];
  isLoading: boolean;
  hasMore: boolean;
  selectedClipId: string | null;
  onSelectClip: (clipId: string) => void;
  onPaste: (clipId: string) => void;
  onCopy: (clipId: string) => void;
  onDelete: (clipId: string) => void;
  onPin: (clipId: string) => void;
  onLoadMore: () => void;
}

export function ClipList({
  clips,
  isLoading,
  hasMore,
  selectedClipId,
  onSelectClip,
  onPaste,
  onLoadMore,
}: ClipListProps) {
  const containerRef = useRef<HTMLDivElement>(null);
  const gridRef = useRef<any>(null);
  const loadMoreTriggerRef = useRef<HTMLDivElement>(null);
  const [width, setWidth] = useState(window.innerWidth);
  const [height, setHeight] = useState(LAYOUT.WINDOW_HEIGHT - LAYOUT.CONTROL_BAR_HEIGHT);

  useEffect(() => {
    if (!containerRef.current) return;

    const observer = new ResizeObserver((entries) => {
      for (const entry of entries) {
        if (entry.contentRect.width > 0) setWidth(entry.contentRect.width);
        if (entry.contentRect.height > 0) setHeight(entry.contentRect.height);
      }
    });

    observer.observe(containerRef.current);
    return () => observer.disconnect();
  }, []);

  // Infinite scroll: Load more when scrolling near the end
  useEffect(() => {
    if (!loadMoreTriggerRef.current || !hasMore) return;

    const observer = new IntersectionObserver(
      (entries) => {
        if (entries[0].isIntersecting && hasMore && !isLoading) {
          console.log('Loading more clips (infinite scroll)...');
          onLoadMore();
        }
      },
      {
        root: null,
        rootMargin: '200px', // Trigger 200px before reaching the end
        threshold: 0.1,
      }
    );

    observer.observe(loadMoreTriggerRef.current);
    return () => observer.disconnect();
  }, [hasMore, isLoading, onLoadMore]);

  if (isLoading && clips.length === 0) {
    return (
      <div className="flex items-center justify-center h-full w-full">
        <div className="flex flex-col items-center gap-3">
          <div className="w-8 h-8 border-2 border-primary/30 border-t-primary rounded-full animate-spin" />
          <p className="text-sm text-muted-foreground">Loading clips...</p>
        </div>
      </div>
    );
  }

  if (clips.length === 0) {
    return (
      <div className="flex flex-col items-center justify-center h-full w-full text-center p-8">
        <h3 className="text-lg font-semibold mb-2 text-gray-400">No clips yet</h3>
        <p className="text-sm text-gray-500 max-w-xs">
          Copy something to your clipboard and it will appear here.
        </p>
      </div>
    );
  }

  return (
    <div
      ref={containerRef}
      className="flex-1 h-full w-full no-scrollbar overflow-hidden flex flex-col"
      onWheel={(e) => {
        if (gridRef.current?.element && e.deltaY !== 0) {
          gridRef.current.element.scrollLeft += e.deltaY;
        }
      }}
    >
      <div className="flex-1">
        <Grid
          gridRef={gridRef}
          columnCount={clips.length + (hasMore ? 2 : 1)} // +1 for padding, +1 for trigger if hasMore
          columnWidth={(index) => {
            if (index === clips.length + 1) return 1; // Invisible trigger column
            if (index === clips.length) return LAYOUT.SIDE_PADDING;
            return TOTAL_COLUMN_WIDTH;
          }}
          rowCount={1}
          rowHeight={height}
          // @ts-ignore
          width={width}
          // @ts-ignore
          height={height}
          cellComponent={(props: any) => {
            const { columnIndex } = props;
            // Render the invisible trigger at the second-to-last position
            if (columnIndex === clips.length && hasMore) {
              return (
                <div
                  ref={loadMoreTriggerRef}
                  style={props.style}
                  className="w-1 h-full pointer-events-none"
                />
              );
            }
            return <ClipCell {...props} />;
          }}
          cellProps={{
            clips,
            selectedClipId,
            onSelectClip,
            onPaste,
          }}
          style={{
            width: '100%',
            height: '100%',
            overflow: 'hidden'
          }}
          className="no-scrollbar"
        />
      </div>
    </div>
  );
}

function ClipCell({
  columnIndex,
  style,
  clips,
  selectedClipId,
  onSelectClip,
  onPaste,
}: CellComponentProps<{
  clips: ClipboardItem[];
  selectedClipId: string | null;
  onSelectClip: (id: string) => void;
  onPaste: (id: string) => void;
}>) {
  if (columnIndex === clips.length) {
    return <div style={style} />;
  }

  const clip = clips[columnIndex];
  if (!clip) return null;

  const isSelected = selectedClipId === clip.id;
  const title = clip.source_app || clip.clip_type.toUpperCase();

  const cardStyle: CSSProperties = {
    ...style,
    left: Number(style.left) + LAYOUT.SIDE_PADDING,
    width: Number(style.width) - LAYOUT.CARD_GAP,
    height: '100%',
    padding: `${LAYOUT.CARD_VERTICAL_PADDING}px 0`, // Safe zones
  };

  return (
    <div style={cardStyle}>
      <div
        onClick={() => onSelectClip(clip.id)}
        onDoubleClick={() => onPaste(clip.id)}
        className={clsx(
          'w-full h-full flex flex-col rounded-xl overflow-hidden cursor-pointer transition-all shadow-lg bg-card border border-border',
          isSelected
            ? 'ring-4 ring-blue-500 transform scale-[1.02] z-10'
            : 'hover:ring-2 hover:ring-primary/30 hover:-translate-y-1'
        )}
      >
        <div className="bg-primary px-4 py-2 flex items-center justify-between flex-shrink-0">
          <span className="font-bold text-primary-foreground text-[10px] uppercase tracking-wider truncate w-full">
            {title}
          </span>
        </div>

        <div className="flex-1 bg-card p-3 overflow-hidden relative">
          {clip.clip_type === 'image' ? (
             <div className="w-full h-full flex items-center justify-center">
               <img
                 src={`data:image/png;base64,${clip.content}`}
                 alt="Clipboard Image"
                 className="max-w-full max-h-full object-contain"
               />
             </div>
          ) : (
            <pre className="font-mono text-[11px] leading-tight whitespace-pre-wrap break-all text-syntax-default">
              {clip.content.split(/(\s+)/).map((word, i) => {
                let colorClass = "text-syntax-default";
                if (/^(const|let|var|function|return|import|from|class|if|else|export|default|async|await)$/.test(word)) colorClass = "text-syntax-keyword";
                else if (/^('.*'|".*"|`.*`)$/.test(word)) colorClass = "text-syntax-string";
                else if (/^\d+$/.test(word)) colorClass = "text-syntax-number";
                else if (/[{}()[\]]/.test(word)) colorClass = "text-syntax-bracket";
                return <span key={i} className={colorClass}>{word}</span>
              })}
            </pre>
          )}
          <div className="absolute bottom-0 left-0 right-0 h-12 bg-gradient-to-t from-card to-transparent pointer-events-none" />
        </div>

        <div className="bg-secondary px-3 py-1.5 border-t border-border flex-shrink-0">
          <span className="text-[10px] text-muted-foreground font-medium">
            {clip.clip_type === 'image' ? `Image (${Math.round(clip.content.length * 0.75 / 1024)}KB)` : `${clip.content.length} characters`}
          </span>
        </div>
      </div>
    </div>
  );
}
