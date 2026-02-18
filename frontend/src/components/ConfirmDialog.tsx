import { X, AlertTriangle } from 'lucide-react';
import { useEffect } from 'react';
import { useTranslation } from 'react-i18next';

interface ConfirmDialogProps {
  isOpen: boolean;
  title: string;
  message: string;
  confirmText?: string;
  cancelText?: string;
  onConfirm: () => void;
  onCancel: () => void;
  variant?: 'danger' | 'warning' | 'info';
}

export function ConfirmDialog({
  isOpen,
  title,
  message,
  confirmText,
  cancelText,
  onConfirm,
  onCancel,
  variant = 'danger',
}: ConfirmDialogProps) {
  const { t } = useTranslation();

  useEffect(() => {
    const handleEscape = (e: KeyboardEvent) => {
      if (e.key === 'Escape') onCancel();
    };
    if (isOpen) {
      window.addEventListener('keydown', handleEscape);
    }
    return () => window.removeEventListener('keydown', handleEscape);
  }, [isOpen, onCancel]);

  if (!isOpen) return null;

  return (
    <div className="animate-in fade-in fixed inset-0 z-50 flex items-center justify-center bg-black/50 backdrop-blur-sm duration-200">
      <div className="animate-in zoom-in-95 w-full max-w-md scale-100 rounded-lg border border-border bg-background p-6 shadow-lg duration-200">
        <div className="mb-4 flex items-center justify-between">
          <div className="flex items-center gap-2">
            <div
              className={`flex h-8 w-8 items-center justify-center rounded-full ${
                variant === 'danger'
                  ? 'bg-destructive/10 text-destructive'
                  : 'bg-yellow-500/10 text-yellow-500'
              }`}
            >
              <AlertTriangle size={18} />
            </div>
            <h3 className="text-lg font-semibold">{title}</h3>
          </div>
          <button onClick={onCancel} className="text-muted-foreground hover:text-foreground">
            <X size={18} />
          </button>
        </div>

        <p className="mb-6 text-sm text-muted-foreground">{message}</p>

        <div className="flex justify-end gap-3">
          <button
            onClick={onCancel}
            className="rounded-md border border-input bg-transparent px-4 py-2 text-sm font-medium hover:bg-accent hover:text-accent-foreground"
          >
            {cancelText || t('common.cancel')}
          </button>
          <button
            onClick={onConfirm}
            className={`rounded-md px-4 py-2 text-sm font-medium text-white transition-colors ${
              variant === 'danger'
                ? 'bg-destructive hover:bg-destructive/90'
                : 'bg-primary hover:bg-primary/90'
            }`}
          >
            {confirmText || t('common.confirm')}
          </button>
        </div>
      </div>
    </div>
  );
}
