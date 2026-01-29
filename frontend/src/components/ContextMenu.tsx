import { useEffect, useRef } from 'react';

interface ContextMenuProps {
  x: number;
  y: number;
  options: {
    label: string;
    onClick: () => void;
    danger?: boolean;
    disabled?: boolean;
  }[];
  onClose: () => void;
}

export function ContextMenu({ x, y, options, onClose }: ContextMenuProps) {
  const menuRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    function handleClickOutside(event: MouseEvent) {
      if (menuRef.current && !menuRef.current.contains(event.target as Node)) {
        onClose();
      }
    }
    // Handle Escape key
    function handleKeyDown(event: KeyboardEvent) {
      if (event.key === 'Escape') {
        onClose();
      }
    }

    document.addEventListener('mousedown', handleClickOutside);
    document.addEventListener('keydown', handleKeyDown);
    return () => {
      document.removeEventListener('mousedown', handleClickOutside);
      document.removeEventListener('keydown', handleKeyDown);
    };
  }, [onClose]);

  // Adjust position if it flows off screen (basic)
  const style = {
    top: y,
    left: x,
  };

  return (
    <div
      ref={menuRef}
      className="animate-in fade-in-0 zoom-in-95 data-[state=open]:animate-in data-[state=closed]:animate-out data-[state=closed]:fade-out-0 data-[state=closed]:zoom-out-95 data-[side=bottom]:slide-in-from-top-2 data-[side=left]:slide-in-from-right-2 data-[side=right]:slide-in-from-left-2 data-[side=top]:slide-in-from-bottom-2 fixed z-50 min-w-[12rem] overflow-hidden rounded-md border border-border bg-popover p-1 shadow-md"
      style={style}
    >
      <div className="flex flex-col">
        {options.map((option, index) => (
          <button
            key={index}
            disabled={option.disabled}
            onClick={() => {
              option.onClick();
              onClose();
            }}
            className={`relative flex cursor-default select-none items-center rounded-sm px-2 py-1.5 text-sm outline-none transition-colors hover:bg-accent hover:text-accent-foreground focus:bg-accent focus:text-accent-foreground disabled:pointer-events-none disabled:opacity-50 ${option.danger ? 'text-red-500 focus:text-red-500' : 'text-popover-foreground'} `}
          >
            {option.label}
          </button>
        ))}
      </div>
    </div>
  );
}
