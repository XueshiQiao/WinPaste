import { Search, X, Keyboard } from 'lucide-react';
import { useEffect, useRef } from 'react';
import { getModifierKey } from '../utils/platform';

interface SearchBarProps {
  query: string;
  onQueryChange: (query: string) => void;
  onClear: () => void;
}

export function SearchBar({ query, onQueryChange, onClear }: SearchBarProps) {
  const inputRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if ((e.metaKey || e.ctrlKey) && e.key === 'f') {
        e.preventDefault();
        inputRef.current?.focus();
      }
      if (e.key === 'Escape' && document.activeElement === inputRef.current) {
        inputRef.current?.blur();
        onClear();
      }
    };

    document.addEventListener('keydown', handleKeyDown);
    return () => document.removeEventListener('keydown', handleKeyDown);
  }, [onClear]);

  return (
    <div className="relative">
      <Search
        size={18}
        className="absolute left-3 top-1/2 -translate-y-1/2 text-muted-foreground"
      />
      <input
        id="search-input"
        ref={inputRef}
        type="text"
        value={query}
        onChange={(e) => onQueryChange(e.target.value)}
        placeholder={`Search clips... (${getModifierKey()}+F)`}
        className="search-input pl-10 pr-20"
      />
      <div className="absolute right-2 top-1/2 flex -translate-y-1/2 items-center gap-1">
        {query && (
          <button onClick={onClear} className="icon-button p-1" title="Clear search">
            <X size={14} />
          </button>
        )}
        <div className="flex items-center gap-1 rounded bg-accent px-1.5 py-0.5">
          <Keyboard size={10} className="text-muted-foreground" />
          <span className="text-[10px] text-muted-foreground">ESC</span>
        </div>
      </div>
    </div>
  );
}
