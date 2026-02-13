import { ClipboardItem } from '../types';
import { clsx } from 'clsx';
import { CLIP_TYPE_ICONS, ClipType } from '../types';
import { FileText, Image, Code, Type, File, Link } from 'lucide-react';

// Map icon string names to Lucide components
const IconMap: Record<string, any> = {
  FileText,
  Image,
  Code,
  Type,
  File,
  Link,
};

interface DragPreviewProps {
  clip: ClipboardItem;
  position: { x: number; y: number };
}

export function DragPreview({ clip, position }: DragPreviewProps) {
  // eslint-disable-next-line @typescript-eslint/no-unused-vars
  const Icon = IconMap[CLIP_TYPE_ICONS[clip.clip_type as ClipType]] || FileText;

  // Generate distinct color based on source app name (reused logic)
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

  const title = clip.source_app || clip.clip_type.toUpperCase();
  const headerColor = getAppColor(title);

  return (
    <div
      className="pointer-events-none fixed z-50 w-64 overflow-hidden rounded-2xl border border-border bg-card opacity-90 shadow-xl ring-2 ring-primary"
      style={{
        left: position.x,
        top: position.y,
        transform: 'translate(10px, 10px)', // Offset from cursor
      }}
    >
      <div className={clsx(headerColor, 'flex items-center gap-2 px-3 py-1.5')}>
        <Icon size={12} className="text-white/80" />
        {clip.source_icon && (
          <img
            src={`data:image/png;base64,${clip.source_icon}`}
            alt=""
            className="h-3 w-3 object-contain"
          />
        )}
        <span className="flex-1 truncate text-[10px] font-bold uppercase tracking-wider text-white">
          {title}
        </span>
      </div>
      <div className="bg-card p-2">
        {clip.clip_type === 'image' ? (
          <div className="flex h-16 items-center justify-center rounded bg-secondary/30">
            <span className="text-xs text-muted-foreground">Image Preview</span>
          </div>
        ) : (
          <pre className="line-clamp-3 whitespace-pre-wrap break-all font-mono text-[10px] leading-tight text-foreground">
            {clip.content.substring(0, 100)}
          </pre>
        )}
      </div>
    </div>
  );
}
