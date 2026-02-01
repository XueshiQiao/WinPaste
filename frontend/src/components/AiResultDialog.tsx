import { X, Copy, Check } from 'lucide-react';
import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { toast } from 'sonner';

interface AiResultDialogProps {
  isOpen: boolean;
  title: string;
  content: string;
  onClose: () => void;
}

export function AiResultDialog({ isOpen, title, content, onClose }: AiResultDialogProps) {
  const [copied, setCopied] = useState(false);

  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === 'Escape') onClose();
    };
    if (isOpen) {
      window.addEventListener('keydown', handleKeyDown);
    }
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [isOpen, onClose]);

  const handleCopy = async () => {
    try {
      // Use the clipboard plugin via existing command or direct plugin usage?
      // Since we want to just write text, we can use navigator.clipboard if available,
      // but in Tauri it's better to use the backend command or plugin.
      // We'll use navigator for simplicity here as it works in Tauri webview usually,
      // OR re-use the `paste_clip` logic if we had a clip ID.
      // But this is new content.
      await navigator.clipboard.writeText(content);
      setCopied(true);
      toast.success('Copied to clipboard');
      setTimeout(() => setCopied(false), 2000);
    } catch (err) {
      console.error('Failed to copy', err);
      toast.error('Failed to copy');
    }
  };

  if (!isOpen) return null;

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50 backdrop-blur-sm">
      <div className="flex h-[80vh] w-[90vw] max-w-2xl flex-col rounded-xl border border-border bg-background shadow-2xl">
        <div className="flex items-center justify-between border-b border-border p-4">
          <h2 className="text-lg font-semibold">{title}</h2>
          <div className="flex items-center gap-2">
            <button
              onClick={handleCopy}
              className="rounded-md p-2 hover:bg-accent hover:text-accent-foreground"
              title="Copy content"
            >
              {copied ? <Check size={18} className="text-emerald-500" /> : <Copy size={18} />}
            </button>
            <button
              onClick={onClose}
              className="rounded-md p-2 hover:bg-accent hover:text-accent-foreground"
            >
              <X size={18} />
            </button>
          </div>
        </div>
        <div className="flex-1 overflow-y-auto p-6">
          <div className="prose dark:prose-invert max-w-none whitespace-pre-wrap text-sm leading-relaxed">
            {content}
          </div>
        </div>
      </div>
    </div>
  );
}
