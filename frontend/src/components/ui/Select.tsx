import { useState, useRef, useEffect } from 'react';
import { ChevronDown, Check } from 'lucide-react';
import { clsx } from 'clsx';
import { twMerge } from 'tailwind-merge';

export interface SelectOption {
  label: string;
  value: string;
}

interface SelectProps {
  value: string;
  onChange: (value: string) => void;
  options: SelectOption[];
  placeholder?: string;
  className?: string;
  disabled?: boolean;
}

export function Select({ value, onChange, options, placeholder, className, disabled }: SelectProps) {
  const [isOpen, setIsOpen] = useState(false);
  const containerRef = useRef<HTMLDivElement>(null);

  const selectedOption = options.find((opt) => opt.value === value);

  useEffect(() => {
    const handleClickOutside = (event: MouseEvent) => {
      if (containerRef.current && !containerRef.current.contains(event.target as Node)) {
        setIsOpen(false);
      }
    };

    if (isOpen) {
      document.addEventListener('mousedown', handleClickOutside);
    }

    return () => {
      document.removeEventListener('mousedown', handleClickOutside);
    };
  }, [isOpen]);

  const handleSelect = (newValue: string) => {
    onChange(newValue);
    setIsOpen(false);
  };

  return (
    <div className={twMerge("relative w-full", className)} ref={containerRef}>
      <button
        type="button"
        onClick={() => !disabled && setIsOpen(!isOpen)}
        disabled={disabled}
        className={clsx(
          "flex w-full items-center justify-between rounded-lg border border-border bg-input px-3 py-2 text-sm text-foreground transition-all duration-200 focus:outline-none focus:ring-2 focus:ring-ring",
          disabled && "opacity-50 cursor-not-allowed",
          isOpen && "ring-2 ring-ring border-ring"
        )}
      >
        <span className={clsx(!selectedOption && "text-muted-foreground")}>
          {selectedOption ? selectedOption.label : placeholder || "Select..."}
        </span>
        <ChevronDown 
          size={16} 
          className={clsx("text-muted-foreground transition-transform duration-200", isOpen && "rotate-180")} 
        />
      </button>

      {isOpen && (
        <div className="absolute left-0 top-full z-50 mt-1 w-full overflow-hidden rounded-lg border border-border bg-popover text-popover-foreground shadow-md animate-in fade-in-0 zoom-in-95 duration-100">
          <div className="max-h-60 overflow-y-auto py-1">
            {options.map((option) => (
              <button
                key={option.value}
                type="button"
                onClick={() => handleSelect(option.value)}
                className={clsx(
                  "relative flex w-full cursor-default select-none items-center py-1.5 pl-3 pr-8 text-sm outline-none transition-colors hover:bg-accent hover:text-accent-foreground",
                  option.value === value && "bg-accent/50 text-accent-foreground font-medium"
                )}
              >
                <span className="truncate">{option.label}</span>
                {option.value === value && (
                  <span className="absolute right-2 flex h-3.5 w-3.5 items-center justify-center">
                    <Check size={14} className="text-primary" />
                  </span>
                )}
              </button>
            ))}
          </div>
        </div>
      )}
    </div>
  );
}
