import { CSSProperties, useEffect, useMemo } from 'react';
import { useTranslation } from 'react-i18next';
import { Grid, GridProps, useGridCallbackRef } from 'react-window';
import { ClipboardItem } from '../types';
import { ClipCard } from './ClipCard';
import { TOTAL_COLUMN_WIDTH } from '../constants';

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
  const { t } = useTranslation();
  const [gridApi, setGridApi] = useGridCallbackRef();
  const selectedClipIndex = useMemo(
    () => (selectedClipId ? clips.findIndex((clip) => clip.id === selectedClipId) : -1),
    [clips, selectedClipId]
  );

  useEffect(() => {
    if (selectedClipIndex >= 0) {
      gridApi?.scrollToColumn({
        index: selectedClipIndex,
        align: 'smart',
        behavior: 'smooth',
      });
    }
  }, [gridApi, selectedClipIndex]);

  const handleWheel = (e: React.WheelEvent) => {
    if (e.deltaY !== 0) {
      const element = gridApi?.element;
      if (!element) return;
      e.preventDefault();
      element.scrollLeft += e.deltaY;
    }
  };

  const handleCellsRendered: GridProps<{}>['onCellsRendered'] = (_visibleCells, allCells) => {
    if (!hasMore || isLoading) return;
    if (allCells.columnStopIndex >= clips.length - 2) {
      onLoadMore();
    }
  };

  const Cell = ({
    columnIndex,
    style,
  }: {
    columnIndex: number;
    style: CSSProperties;
  }) => {
    const clip = clips[columnIndex];
    if (!clip) return null;

    return (
      <div style={style} className="flex h-full items-center justify-center">
        <ClipCard
          clip={clip}
          isSelected={selectedClipId === clip.id}
          onSelect={() => onSelectClip(clip.id)}
          onPaste={() => onPaste(clip.id)}
          onCopy={() => onCopy(clip.id)}
          onDragStart={onDragStart}
          onContextMenu={(e: React.MouseEvent) => onCardContextMenu?.(e, clip.id)}
        />
      </div>
    );
  };

  if (isLoading && clips.length === 0) {
    return (
      <div className="flex h-full w-full items-center justify-center">
        <div className="flex flex-col items-center gap-3">
          <div className="h-8 w-8 animate-spin rounded-full border-2 border-primary/30 border-t-primary" />
          <p className="text-sm text-muted-foreground">{t('clipList.loadingClips')}</p>
        </div>
      </div>
    );
  }

  if (clips.length === 0) {
    return (
      <div className="flex h-full w-full flex-col items-center justify-center p-8 text-center">
        <h3 className="mb-2 text-lg font-semibold text-gray-400">{t('clipList.empty')}</h3>
        <p className="max-w-xs text-sm text-gray-500">
          {t('clipList.emptyDesc')}
        </p>
      </div>
    );
  }

  return (
    <Grid
      className="no-scrollbar h-full w-full flex-1 overflow-x-auto overflow-y-hidden px-4"
      defaultHeight={240}
      defaultWidth={1000}
      gridRef={setGridApi}
      rowCount={1}
      rowHeight="100%"
      columnCount={clips.length}
      columnWidth={TOTAL_COLUMN_WIDTH}
      overscanCount={4}
      cellComponent={({ columnIndex, style }) => (
        <Cell
          columnIndex={columnIndex}
          style={style}
        />
      )}
      cellProps={{}}
      onCellsRendered={handleCellsRendered}
      onWheel={handleWheel}
      style={{
        scrollBehavior: 'auto',
      }}
    />
  );
}
