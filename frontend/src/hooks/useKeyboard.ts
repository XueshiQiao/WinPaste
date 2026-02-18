import { useEffect } from 'react';

interface KeyboardOptions {
  onClose?: () => void;
  onSearch?: () => void;
  onDelete?: () => void;
  onPin?: () => void;
  onNavigateLeft?: () => void;
  onNavigateRight?: () => void;
  onPaste?: () => void;
}

export function useKeyboard(options: KeyboardOptions) {
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === 'Escape' && options.onClose) {
        e.preventDefault();
        options.onClose();
      }

      if ((e.metaKey || e.ctrlKey) && e.key === 'f' && options.onSearch) {
        e.preventDefault();
        options.onSearch();
      }

      if (e.key === 'Delete' && options.onDelete) {
        e.preventDefault();
        options.onDelete();
      }

      if (e.key === 'p' && !e.metaKey && !e.ctrlKey && options.onPin) {
        e.preventDefault();
        options.onPin();
      }

      if (e.key === 'ArrowLeft' && options.onNavigateLeft) {
        e.preventDefault();
        options.onNavigateLeft();
      }

      if (e.key === 'ArrowRight' && options.onNavigateRight) {
        e.preventDefault();
        options.onNavigateRight();
      }

      if (e.key === 'Enter' && options.onPaste) {
        e.preventDefault();
        options.onPaste();
      }
    };

    document.addEventListener('keydown', handleKeyDown);
    return () => document.removeEventListener('keydown', handleKeyDown);
  }, [options]);
}
