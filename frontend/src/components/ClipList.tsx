import { ClipboardItem } from '../types';
import { clsx } from 'clsx';
import { useState, useRef, useEffect, CSSProperties } from 'react';
import { Grid, type CellComponentProps } from 'react-window';

interface ClipListProps {
  clips: ClipboardItem[];
  isLoading: boolean;
  selectedClipId: string | null;
  onSelectClip: (clipId: string) => void;
  onPaste: (clipId: string) => void;
  onCopy: (clipId: string) => void;
  onDelete: (clipId: string) => void;
  onPin: (clipId: string) => void;
}

export function ClipList({
  clips,
  isLoading,
  selectedClipId,
  onSelectClip,
  onPaste,
}: ClipListProps) {
  const containerRef = useRef<HTMLDivElement>(null);
  const gridRef = useRef<any>(null);
  const [width, setWidth] = useState(0);
  const [height, setHeight] = useState(0);

  useEffect(() => {
    if (!containerRef.current) return;
    
    const observer = new ResizeObserver((entries) => {
      for (const entry of entries) {
        setWidth(entry.contentRect.width);
        setHeight(entry.contentRect.height);
      }
    });
    
    observer.observe(containerRef.current);
    return () => observer.disconnect();
  }, []);

  if (isLoading) {
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
        <h3 className="text-lg font-semibold mb-2 text-gray-400">No clips found</h3>
        <p className="text-sm text-gray-500 max-w-xs">
          Your clipboard history is empty for this category.
        </p>
      </div>
    );
  }

  const ITEM_WIDTH = 210;
  const GAP = 24;
  const COLUMN_WIDTH = ITEM_WIDTH + GAP;

  const listWidth = width > 0 ? width : window.innerWidth;
  const listHeight = height > 0 ? height : 300;

  return (
    <div 
      ref={containerRef} 
      className="h-full w-full no-scrollbar overflow-hidden"
      onWheel={(e) => {
        if (gridRef.current?.element && e.deltaY !== 0) {
          gridRef.current.element.scrollLeft += e.deltaY;
        }
      }}
    >
      <Grid
        gridRef={gridRef}
        columnCount={clips.length}
        columnWidth={COLUMN_WIDTH}
        rowCount={1}
        rowHeight={listHeight}
        cellComponent={ClipCell}
        cellProps={{
          clips,
          selectedClipId,
          onSelectClip,
          onPaste,
        }}
        style={{ 
          width: listWidth,
          height: listHeight,
          overflow: 'hidden' 
        }}
        className="no-scrollbar"
      />
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
  const clip = clips[columnIndex];
  if (!clip) return null;

  const isSelected = selectedClipId === clip.id;
  const title = clip.source_app || clip.clip_type.toUpperCase();

  // Adjust style to account for gap and left padding
  const cardStyle: CSSProperties = {
    ...style,
    left: Number(style.left) + 24, // 24px starting padding
    width: Number(style.width) - 24, // Leave 24px gap between items
    height: '100%',
  };

  return (
    <div style={cardStyle}>
      <div
        onClick={() => onSelectClip(clip.id)}
        onDoubleClick={() => onPaste(clip.id)}
        style={{
          marginTop: 12,
          height: 'calc(100% - 24px)'
        }}
        className={clsx(
          'w-full h-full flex flex-col rounded-xl overflow-hidden cursor-pointer transition-all shadow-lg',
          isSelected 
            ? 'ring-4 ring-blue-500 transform scale-[1.02] z-10' 
            : 'hover:ring-2 hover:ring-purple-500/30 hover:-translate-y-1'
        )}
      >
        <div className="bg-primary px-4 py-3 flex items-center justify-between flex-shrink-0">
          <span className="font-bold text-primary-foreground text-sm truncate w-full">
            {title}
          </span>
        </div>

        <div className="flex-1 bg-card p-4 overflow-hidden relative">
          <pre className="font-mono text-xs leading-tight whitespace-pre-wrap break-all text-syntax-default">
            {clip.content.split(/(\s+)/).map((word, i) => {
              let colorClass = "text-syntax-default";
              if (/^(const|let|var|function|return|import|from|class|if|else|export|default|async|await)$/.test(word)) colorClass = "text-syntax-keyword";
              else if (/^('.*'|".*"|`.*`)$/.test(word)) colorClass = "text-syntax-string";
              else if (/^\d+$/.test(word)) colorClass = "text-syntax-number";
              else if (/[{}()[\]]/.test(word)) colorClass = "text-syntax-bracket";
              return <span key={i} className={colorClass}>{word}</span>
            })}
          </pre>
          <div className="absolute bottom-0 left-0 right-0 h-16 bg-gradient-to-t from-card to-transparent pointer-events-none" />
        </div>

        <div className="bg-secondary px-4 py-2 border-t border-border flex-shrink-0">
          <span className="text-xs text-muted-foreground font-medium">
            {clip.content.length} characters
          </span>
        </div>
      </div>
    </div>
  );
}
