import { ClipboardItem } from '../types';
import { formatDistanceToNow } from 'date-fns';
import {
  Pin,
  Copy,
  Trash2,
  MoreVertical,
  FileText,
  Image,
  Code,
  Link,
  File,
} from 'lucide-react';
import { clsx } from 'clsx';
import { useState, useRef, useEffect } from 'react';

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

const CLIP_TYPE_ICONS_MAP: Record<string, typeof FileText> = {
  text: FileText,
  image: Image,
  html: Code,
  rtf: FileText,
  file: File,
  url: Link,
};

export function ClipList({
  clips,
  isLoading,
  selectedClipId,
  onSelectClip,
  onPaste,
  onCopy,
  onDelete,
  onPin,
}: ClipListProps) {
  const [menuOpenId, setMenuOpenId] = useState<string | null>(null);
  const menuRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const handleClickOutside = (event: MouseEvent) => {
      if (menuRef.current && !menuRef.current.contains(event.target as Node)) {
        setMenuOpenId(null);
      }
    };

    document.addEventListener('mousedown', handleClickOutside);
    return () => document.removeEventListener('mousedown', handleClickOutside);
  }, []);

  if (isLoading) {
    return (
      <div className="flex items-center justify-center h-64">
        <div className="flex flex-col items-center gap-3">
          <div className="w-8 h-8 border-2 border-primary/30 border-t-primary rounded-full animate-spin" />
          <p className="text-sm text-muted-foreground">Loading clips...</p>
        </div>
      </div>
    );
  }

  if (clips.length === 0) {
    return (
      <div className="flex flex-col items-center justify-center h-64 text-center">
        <div className="w-16 h-16 rounded-full bg-accent flex items-center justify-center mb-4">
          <FileText size={32} className="text-muted-foreground" />
        </div>
        <h3 className="text-lg font-semibold mb-2">No clips yet</h3>
        <p className="text-sm text-muted-foreground max-w-xs">
          Copy something to your clipboard and it will appear here.
        </p>
      </div>
    );
  }

  return (
    <div className="space-y-2">
      {clips.map((clip) => {
        const isSelected = selectedClipId === clip.id;
        const Icon = CLIP_TYPE_ICONS_MAP[clip.clip_type] || FileText;

        return (
          <div
            key={clip.id}
            className={clsx(
              'clip-card cursor-pointer group',
              isSelected && 'selected'
            )}
            onClick={() => onSelectClip(clip.id)}
            onDoubleClick={() => onPaste(clip.id)}
          >
            <div className="flex items-start gap-3">
              <div className={clsx(
                'w-10 h-10 rounded-lg flex items-center justify-center flex-shrink-0',
                'bg-accent text-muted-foreground'
              )}>
                <Icon size={20} />
              </div>

              <div className="flex-1 min-w-0">
                <div className="flex items-center gap-2 mb-1">
                  <span className="text-xs text-muted-foreground uppercase tracking-wider">
                    {clip.clip_type}
                  </span>
                  {clip.is_pinned && (
                    <Pin size={12} className="text-primary fill-primary" />
                  )}
                  <span className="text-xs text-muted-foreground ml-auto">
                    {formatDistanceToNow(new Date(clip.created_at), { addSuffix: true })}
                  </span>
                </div>

                <p className="text-sm text-foreground line-clamp-2 break-all">
                  {clip.preview}
                </p>

                {clip.preview.length > 100 && (
                  <p className="text-xs text-muted-foreground mt-1">
                    {clip.content.length} characters
                  </p>
                )}
              </div>

              <div className="flex items-center gap-1 opacity-0 group-hover:opacity-100 transition-opacity">
                <button
                  onClick={(e) => {
                    e.stopPropagation();
                    onCopy(clip.id);
                  }}
                  className="icon-button"
                  title="Copy to Clipboard"
                >
                  <Copy size={16} />
                </button>
                <button
                  onClick={(e) => {
                    e.stopPropagation();
                    onPin(clip.id);
                  }}
                  className={clsx(
                    'icon-button',
                    clip.is_pinned && 'text-primary'
                  )}
                  title={clip.is_pinned ? 'Unpin' : 'Pin'}
                >
                  <Pin size={16} />
                </button>
                <div className="relative">
                  <button
                    onClick={(e) => {
                      e.stopPropagation();
                      setMenuOpenId(menuOpenId === clip.id ? null : clip.id);
                    }}
                    className="icon-button"
                    title="More options"
                  >
                    <MoreVertical size={16} />
                  </button>

                  {menuOpenId === clip.id && (
                    <div
                      ref={menuRef}
                      className="absolute right-0 top-full mt-1 w-40 py-1 bg-popover border border-border rounded-lg shadow-lg animate-scale-in z-10"
                    >
                      <button
                        onClick={(e) => {
                          e.stopPropagation();
                          onPaste(clip.id);
                          setMenuOpenId(null);
                        }}
                        className="w-full px-3 py-2 text-left text-sm hover:bg-accent flex items-center gap-2"
                      >
                        <Copy size={14} />
                        Paste
                      </button>
                      <button
                        onClick={(e) => {
                          e.stopPropagation();
                          onPin(clip.id);
                          setMenuOpenId(null);
                        }}
                        className="w-full px-3 py-2 text-left text-sm hover:bg-accent flex items-center gap-2"
                      >
                        <Pin size={14} />
                        {clip.is_pinned ? 'Unpin' : 'Pin'}
                      </button>
                      <button
                        onClick={(e) => {
                          e.stopPropagation();
                          onDelete(clip.id);
                          setMenuOpenId(null);
                        }}
                        className="w-full px-3 py-2 text-left text-sm hover:bg-accent text-destructive flex items-center gap-2"
                      >
                        <Trash2 size={14} />
                        Delete
                      </button>
                    </div>
                  )}
                </div>
              </div>
            </div>
          </div>
        );
      })}
    </div>
  );
}
