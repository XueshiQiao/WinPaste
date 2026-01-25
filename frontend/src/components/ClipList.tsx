import { ClipboardItem } from '../types';
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

export function ClipList({
  clips,
  isLoading,
  selectedClipId,
  onSelectClip,
  onPaste,
}: ClipListProps) {
  // eslint-disable-next-line @typescript-eslint/no-unused-vars
  const [_, setMenuOpenId] = useState<string | null>(null);
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

  return (
    <div className="flex flex-row overflow-x-auto gap-6 p-6 items-start h-full min-h-[300px] w-full snap-x no-scrollbar">
      {clips.map((clip) => {
        const isSelected = selectedClipId === clip.id;
        // Determine title from app or type
        const title = clip.source_app || clip.clip_type.toUpperCase();
        
        return (
          <div
            key={clip.id}
            onClick={() => onSelectClip(clip.id)}
            onDoubleClick={() => onPaste(clip.id)}
            className={clsx(
              'flex-shrink-0 w-[300px] h-[400px] flex flex-col rounded-xl overflow-hidden cursor-pointer transition-all snap-center shadow-lg',
              isSelected 
                ? 'ring-4 ring-blue-500 transform scale-[1.02] z-10' 
                : 'hover:ring-2 hover:ring-purple-500/30 hover:-translate-y-1'
            )}
          >
            {/* Header: Solid Purple Block */}
            <div className="bg-[#6D28D9] px-4 py-3 flex items-center justify-between">
              <span className="font-bold text-white text-sm truncate w-full">
                {title}
              </span>
            </div>

            {/* Body: Code Snippet View */}
            <div className="flex-1 bg-[#1E1E1E] p-4 overflow-hidden relative">
              <pre className="font-mono text-sm leading-relaxed whitespace-pre-wrap break-all text-gray-300">
                {/* Simulated syntax highlighting colors for demo purposes since we don't have a parser yet */}
                {clip.content.split(/(\s+)/).map((word, i) => {
                  // Simple heuristic for coloring to simulate syntax highlighting
                  let colorClass = "text-[#D4D4D4]"; // Default
                  if (/^(const|let|var|function|return|import|from|class|if|else|export|default|async|await)$/.test(word)) colorClass = "text-[#569CD6]"; // Blue keywords
                  else if (/^('.*'|".*"|`.*`)$/.test(word)) colorClass = "text-[#6A9955]"; // Green Strings
                  else if (/^\d+$/.test(word)) colorClass = "text-[#B5CEA8]"; // Light Green Numbers
                  else if (/[{}()[\]]/.test(word)) colorClass = "text-[#FFD700]"; // Yellow Brackets
                  
                  return <span key={i} className={colorClass}>{word}</span>
                })}
              </pre>
              
              {/* Fade out at bottom */}
              <div className="absolute bottom-0 left-0 right-0 h-16 bg-gradient-to-t from-[#1E1E1E] to-transparent pointer-events-none" />
            </div>

            {/* Footer: Size Indicator */}
            <div className="bg-[#252526] px-4 py-2 border-t border-[#333]">
              <span className="text-xs text-gray-500 font-medium">
                {clip.content.length} characters
              </span>
            </div>
          </div>
        );
      })}
    </div>
  );
}
