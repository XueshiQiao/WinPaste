import { CSSProperties, useEffect, useMemo, useRef } from 'react';
import { useTranslation } from 'react-i18next';
import { Grid, GridProps, useGridCallbackRef } from 'react-window';
import { ClipboardItem } from '../types';
import { ClipCard } from './ClipCard';
import { TOTAL_COLUMN_WIDTH } from '../constants';

interface ClipListProps {
  clips: ClipboardItem[];
  isLoading: boolean;
  hasMore: boolean;
  resetToken: number;
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
  resetToken,
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
  const wheelTargetRef = useRef(0);
  const wheelRafRef = useRef<number | null>(null);
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

  useEffect(() => {
    if (wheelRafRef.current !== null) {
      cancelAnimationFrame(wheelRafRef.current);
      wheelRafRef.current = null;
    }
    wheelTargetRef.current = 0;

    const element = gridApi?.element;
    if (element) {
      element.scrollLeft = 0;
    }

    gridApi?.scrollToColumn({
      index: 0,
      align: 'start',
      behavior: 'auto',
    });
  }, [gridApi, resetToken]);

  const handleWheel = (e: React.WheelEvent) => {
    const element = gridApi?.element;
    if (!element) return;

    const rawDelta = Math.abs(e.deltaX) > Math.abs(e.deltaY) ? e.deltaX : e.deltaY;
    if (rawDelta === 0) return;

    e.preventDefault();

    let deltaPx = rawDelta;
    if (e.deltaMode === 1) {
      deltaPx *= 16;
    } else if (e.deltaMode === 2) {
      deltaPx *= element.clientWidth;
    }

    // Smaller per-notch travel with a single RAF-driven animation target.
    const scrollStep = deltaPx * 0.52;
    const estimatedMax = Math.max(0, clips.length * TOTAL_COLUMN_WIDTH - element.clientWidth);
    const measuredMax = Math.max(0, element.scrollWidth - element.clientWidth);
    const maxScrollLeft = Math.max(estimatedMax, measuredMax);

    const baseTarget = wheelRafRef.current === null ? element.scrollLeft : wheelTargetRef.current;
    wheelTargetRef.current = Math.min(maxScrollLeft, Math.max(0, baseTarget + scrollStep));

    if (wheelRafRef.current !== null) return;

    const tick = () => {
      const el = gridApi?.element;
      if (!el) {
        wheelRafRef.current = null;
        return;
      }

      const diff = wheelTargetRef.current - el.scrollLeft;
      if (Math.abs(diff) < 0.5) {
        el.scrollLeft = wheelTargetRef.current;
        wheelRafRef.current = null;
        return;
      }

      el.scrollLeft += diff * 0.24;
      wheelRafRef.current = requestAnimationFrame(tick);
    };

    wheelRafRef.current = requestAnimationFrame(tick);
  };

  useEffect(() => {
    return () => {
      if (wheelRafRef.current !== null) {
        cancelAnimationFrame(wheelRafRef.current);
      }
    };
  }, []);

  const handleCellsRendered: GridProps<{}>['onCellsRendered'] = (_visibleCells, allCells) => {
    if (!hasMore || isLoading) return;
    if (allCells.columnStopIndex >= clips.length - 2) {
      onLoadMore();
    }
  };

  const Cell = ({ columnIndex, style }: { columnIndex: number; style: CSSProperties }) => {
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
        <p className="max-w-xs text-sm text-gray-500">{t('clipList.emptyDesc')}</p>
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
      cellComponent={({ columnIndex, style }) => <Cell columnIndex={columnIndex} style={style} />}
      cellProps={{}}
      onCellsRendered={handleCellsRendered}
      onWheel={handleWheel}
      style={{
        scrollBehavior: 'auto',
      }}
    />
  );
}
