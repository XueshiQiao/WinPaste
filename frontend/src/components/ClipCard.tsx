import { ClipboardItem } from '../types';
import { clsx } from 'clsx';
import { useMemo, memo, useState, forwardRef } from 'react';
import { useTranslation } from 'react-i18next';
import { LAYOUT, TOTAL_COLUMN_WIDTH, PREVIEW_CHAR_LIMIT } from '../constants';
import { Copy, Check } from 'lucide-react';

interface ClipCardProps {
  clip: ClipboardItem;
  isSelected: boolean;
  onSelect: () => void;
  onPaste: () => void;
  onCopy: () => void;
  onDragStart: (clipId: string, startX: number, startY: number) => void;
  onContextMenu?: (e: React.MouseEvent) => void;
}

export const ClipCard = memo(forwardRef<HTMLDivElement, ClipCardProps>(function ClipCard({
  clip,
  isSelected,
  onSelect,
  onPaste,
  onCopy,
  onDragStart,
  onContextMenu,
}: ClipCardProps, ref) {
  const { t } = useTranslation();
  const [copied, setCopied] = useState(false);
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
        <pre className="whitespace-pre-wrap break-all font-mono text-[13px] leading-tight text-foreground">
          <span>{clip.content.substring(0, PREVIEW_CHAR_LIMIT)}</span>
        </pre>
      );
    }
  }, [clip.clip_type, clip.content]);

  // Generate stable color index based on source app name
  const getAppColorIndex = (name: string) => {
    let hash = 0;
    for (let i = 0; i < name.length; i++) {
      hash = name.charCodeAt(i) + ((hash << 5) - hash);
    }
    return Math.abs(hash) % 15;
  };

  const appHue = useMemo(() => {
    const index = getAppColorIndex(title);
    const hueStep = 360 / 15;
    return Math.round(index * hueStep);
  }, [title]);

  const handleMouseDown = (e: React.MouseEvent) => {
    // Only left click
    if (e.button !== 0) return;
    onDragStart(clip.id, e.clientX, e.clientY);
  };

  const handleContextMenu = (e: React.MouseEvent) => {
    e.preventDefault();
    onContextMenu?.(e);
  };

  const handleAmbientMove = (e: React.MouseEvent<HTMLDivElement>) => {
    const rect = e.currentTarget.getBoundingClientRect();
    const x = e.clientX - rect.left;
    const y = e.clientY - rect.top;

    const leftDistance = x;
    const rightDistance = rect.width - x;
    const topDistance = y;
    const bottomDistance = rect.height - y;
    const minDistance = Math.min(leftDistance, rightDistance, topDistance, bottomDistance);

    let edgeX = x;
    let edgeY = y;

    if (minDistance === leftDistance) {
      edgeX = 0;
    } else if (minDistance === rightDistance) {
      edgeX = rect.width;
    } else if (minDistance === topDistance) {
      edgeY = 0;
    } else {
      edgeY = rect.height;
    }

    e.currentTarget.style.setProperty('--ambient-x', `${x}px`);
    e.currentTarget.style.setProperty('--ambient-y', `${y}px`);
    e.currentTarget.style.setProperty('--edge-x', `${edgeX}px`);
    e.currentTarget.style.setProperty('--edge-y', `${edgeY}px`);
    e.currentTarget.style.setProperty('--ambient-opacity', '1');
  };

  const handleAmbientLeave = (e: React.MouseEvent<HTMLDivElement>) => {
    e.currentTarget.style.setProperty('--ambient-opacity', '0');
  };

  return (
    <div
      ref={ref}
      style={{
        width: TOTAL_COLUMN_WIDTH - LAYOUT.CARD_GAP,
        height: 'calc(100% - 32px)',
      }}
      className="flex-shrink-0"
    >
      <div
        onMouseDown={handleMouseDown}
        onMouseMove={handleAmbientMove}
        onMouseLeave={handleAmbientLeave}
        onClick={onSelect}
        onDoubleClick={onPaste}
        onContextMenu={handleContextMenu}
        style={{
          '--ambient-x': '50%',
          '--ambient-y': '50%',
          '--edge-x': '50%',
          '--edge-y': '0%',
          '--ambient-opacity': 0,
          '--app-hue': `${appHue}`,
        } as React.CSSProperties}
        className={clsx(
          'relative flex h-full w-full cursor-pointer select-none flex-col overflow-hidden rounded-2xl border border-border bg-card shadow-lg transition-all',
          isSelected
            ? 'z-10 scale-[1.02] transform ring-4 ring-blue-500'
            : 'hover:-translate-y-1',
          'group'
        )}
      >
        <div
          className="pointer-events-none absolute inset-0 z-20 transition-opacity duration-200 dark:hidden"
          style={{
            opacity: 'calc(var(--ambient-opacity) * 0.78)',
            background: `
              radial-gradient(300px circle at var(--ambient-x) var(--ambient-y), hsl(var(--foreground) / 0.09), transparent 68%),
              radial-gradient(150px circle at var(--ambient-x) var(--ambient-y), hsl(var(--app-hue) 88% 60% / 0.12), transparent 74%)
            `,
            mixBlendMode: 'normal',
          }}
        />

        <div
          className="pointer-events-none absolute inset-0 z-20 hidden transition-opacity duration-200 dark:block"
          style={{
            opacity: 'calc(var(--ambient-opacity) * 0.95)',
            background: `
              radial-gradient(260px circle at var(--ambient-x) var(--ambient-y), hsl(var(--app-hue) 88% 62% / 0.22), transparent 68%),
              radial-gradient(140px circle at var(--ambient-x) var(--ambient-y), hsl(var(--foreground) / 0.12), transparent 72%)
            `,
            mixBlendMode: 'screen',
          }}
        />

        <div
          className={clsx(
            'pointer-events-none absolute inset-0 z-20 rounded-2xl p-[1.25px] transition-opacity duration-200',
            isSelected ? 'opacity-100' : 'opacity-0 group-hover:opacity-100'
          )}
          style={{
            background: `
              radial-gradient(170px circle at var(--edge-x) var(--edge-y), hsl(var(--app-hue) 90% 64% / 0.92), transparent 62%),
              radial-gradient(120px circle at var(--edge-x) var(--edge-y), hsl(var(--app-hue) 86% 58% / 0.52), transparent 70%),
              radial-gradient(95px circle at var(--edge-x) var(--edge-y), hsl(var(--app-hue) 82% 50% / 0.46), transparent 76%)
            `,
            WebkitMask:
              'linear-gradient(#000 0 0) content-box, linear-gradient(#000 0 0)',
            WebkitMaskComposite: 'xor',
            maskComposite: 'exclude',
            filter: 'saturate(1.2) blur(0.2px)',
          } as React.CSSProperties}
        />

        <div
          className="relative z-10 flex flex-shrink-0 items-center gap-2 px-2 py-1.5"
          style={{ backgroundColor: `hsl(${appHue} 82% 60%)` }}
        >
          {clip.source_icon && (
            <img
              src={`data:image/png;base64,${clip.source_icon}`}
              alt=""
              className="h-4 w-4 object-contain"
            />
          )}
          <span className="flex-1 truncate text-[11px] font-bold uppercase tracking-wider text-foreground">
            {title}
          </span>
          <button
            onClick={(e) => {
              e.stopPropagation();
              onCopy();
              setCopied(true);
              setTimeout(() => setCopied(false), 2000);
            }}
            className="rounded-md p-1 opacity-0 transition-all hover:bg-black/10 group-hover:opacity-100"
            title="Copy to clipboard"
          >
            {copied ? (
              <Check size={14} className="text-emerald-500" />
            ) : (
              <Copy size={14} className="text-foreground/70 hover:text-foreground" />
            )}
          </button>
        </div>

        <div className="relative z-10 flex-1 overflow-hidden bg-card/90 p-2">
          {renderedContent}
          <div className="pointer-events-none absolute bottom-0 left-0 right-0 h-12 bg-gradient-to-t from-card/100 to-card/30" />
        </div>

        <div className="absolute bottom-0 left-0 right-0 z-10 bg-gradient-to-t from-card via-card/100 to-transparent/0 px-3 py-1.5">
          <span className="text-[11px] font-medium text-muted-foreground/50">
            {clip.clip_type === 'image'
              ? t('clipList.imageSize', { size: Math.round((clip.content.length * 0.75) / 1024) })
              : t('clipList.textLength', { count: clip.content.length })}
          </span>
        </div>
      </div>
    </div>
  );
}));
