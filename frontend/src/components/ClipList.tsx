import { ClipboardItem } from "../types";
import { clsx } from "clsx";
import { useRef, useMemo } from "react";
import { memo } from "react";
import { LAYOUT, TOTAL_COLUMN_WIDTH } from "../constants";

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

  // Native onScroll handler for infinite scroll
  const handleScroll = () => {
    if (!containerRef.current || !hasMore || isLoading) return;

    // We check native scroll properties
    const { scrollLeft, scrollWidth, clientWidth } = containerRef.current;

    // If scrolled within 300px of the end
    if (scrollLeft + clientWidth >= scrollWidth - 300) {
      console.log("Scroll to end detected (native), loading more...");
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
        <h3 className="text-lg font-semibold mb-2 text-gray-400">
          No clips yet
        </h3>
        <p className="text-sm text-gray-500 max-w-xs">
          Copy something to your clipboard and it will appear here.
        </p>
      </div>
    );
  }

  return (
    <div
      ref={containerRef}
      className="flex-1 h-full w-full overflow-x-auto overflow-y-hidden no-scrollbar flex items-center px-4 gap-4"
      onScroll={handleScroll}
      onWheel={handleWheel}
      style={{
        // Smooth scrolling disabled per user request
        scrollBehavior: "auto",
      }}
    >
      {clips.map((clip) => (
        <ClipCard
          key={clip.id}
          clip={clip}
          isSelected={selectedClipId === clip.id}
          onSelect={() => onSelectClip(clip.id)}
          onPaste={() => onPaste(clip.id)}
        />
      ))}

      {/* Loading indicator at the end */}
      {isLoading && clips.length > 0 && (
        <div className="h-full flex items-center justify-center min-w-[100px]">
          <div className="w-6 h-6 border-2 border-primary/30 border-t-primary rounded-full animate-spin" />
        </div>
      )}

      {/* Spacer end */}
      <div className="min-w-[20px] h-full flex-shrink-0" />
    </div>
  );
}

const ClipCard = memo(function ClipCard({
  clip,
  isSelected,
  onSelect,
  onPaste,
}: {
  clip: ClipboardItem;
  isSelected: boolean;
  onSelect: () => void;
  onPaste: () => void;
}) {
  const title = clip.source_app || clip.clip_type.toUpperCase();

  // Memoize the content rendering
  const renderedContent = useMemo(() => {
    if (clip.clip_type === "image") {
      return (
        <div className="w-full h-full flex items-center justify-center">
          <img
            src={`data:image/png;base64,${clip.content}`}
            alt="Clipboard Image"
            className="max-w-full max-h-full object-contain"
          />
        </div>
      );
    } else {
      return (
        <pre className="font-mono text-[11px] leading-tight whitespace-pre-wrap break-all text-foreground">
          <span>{clip.content}</span>
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
      "bg-red-400",
      "bg-orange-400",
      "bg-amber-400",
      "bg-green-400",
      "bg-emerald-400",
      "bg-teal-400",
      "bg-cyan-400",
      "bg-sky-400",
      "bg-blue-400",
      "bg-indigo-400",
      "bg-violet-400",
      "bg-purple-400",
      "bg-fuchsia-400",
      "bg-pink-400",
      "bg-rose-400",
    ];
    return colors[Math.abs(hash) % colors.length];
  };

  const headerColor = getAppColor(title);

  return (
    <div
      style={{
        width: TOTAL_COLUMN_WIDTH - LAYOUT.CARD_GAP,
        height:
          LAYOUT.WINDOW_HEIGHT -
          LAYOUT.CONTROL_BAR_HEIGHT -
          LAYOUT.CARD_VERTICAL_PADDING * 2,
      }}
      className="flex-shrink-0"
    >
      <div
        onClick={onSelect}
        onDoubleClick={onPaste}
        className={clsx(
          "w-full h-full flex flex-col rounded-xl overflow-hidden cursor-pointer transition-all shadow-lg bg-card border border-border relative",
          isSelected
            ? "ring-4 ring-blue-500 transform scale-[1.02] z-10"
            : "hover:ring-2 hover:ring-primary/30 hover:-translate-y-1",
        )}
      >
        <div
          className={clsx(
            headerColor,
            "px-4 py-2 flex items-center gap-2 flex-shrink-0",
          )}
        >
          {clip.source_icon && (
            <img
              src={`data:image/png;base64,${clip.source_icon}`}
              alt=""
              className="w-4 h-4 object-contain"
            />
          )}
          <span className="font-bold text-white text-[10px] uppercase tracking-wider truncate flex-1 shadow-sm">
            {title}
          </span>
        </div>

        <div className="flex-1 bg-card p-3 overflow-hidden relative">
          {renderedContent}
          <div className="absolute bottom-0 left-0 right-0 h-12 bg-gradient-to-t from-card to-transparent pointer-events-none" />
        </div>

        <div className="bg-secondary px-3 py-1.5 border-t border-border flex-shrink-0">
          <span className="text-[10px] text-muted-foreground font-medium">
            {clip.clip_type === "image"
              ? `Image (${Math.round((clip.content.length * 0.75) / 1024)}KB)`
              : `${clip.content.length} characters`}
          </span>
        </div>
      </div>
    </div>
  );
});
