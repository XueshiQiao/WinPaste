import { Settings } from '../types';
import { X, Save, Trash2, Info } from 'lucide-react';
import { useState, useEffect, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';

interface SettingsPanelProps {
  settings: Settings;
  onClose: () => void;
  onSave: (settings: Settings) => void;
}

export function SettingsPanel({ settings: initialSettings, onClose, onSave }: SettingsPanelProps) {
  const [settings, setSettings] = useState<Settings>(initialSettings);
  const [historySize, setHistorySize] = useState<number>(0);
  const [recordingHotkey, setRecordingHotkey] = useState(false);

  useEffect(() => {
    invoke<number>('get_clipboard_history_size').then(setHistorySize).catch(console.error);
  }, []);

  const handleSave = async () => {
    try {
      await invoke('register_global_shortcut', { hotkey: settings.hotkey });
    } catch (error) {
      console.error('Failed to register hotkey:', error);
    }
    onSave(settings);
  };

  const handleClearHistory = async () => {
    try {
      await invoke('clear_clipboard_history');
      setHistorySize(0);
    } catch (error) {
      console.error('Failed to clear history:', error);
    }
  };

  const handleKeyDown = useCallback(
    (e: KeyboardEvent) => {
      if (!recordingHotkey) return;

      e.preventDefault();
      e.stopPropagation();

      const modifiers: string[] = [];
      if (e.ctrlKey) modifiers.push('Ctrl');
      if (e.altKey) modifiers.push('Alt');
      if (e.shiftKey) modifiers.push('Shift');
      if (e.metaKey) modifiers.push('Cmd');

      const key = e.key.toUpperCase();
      if (key.length === 1 && /[A-Z0-9]/.test(key)) {
        modifiers.push(key);
      } else if (key === ' ') {
        modifiers.push('Space');
      } else if (key === 'ESCAPE') {
        setRecordingHotkey(false);
        return;
      }

      const newHotkey = modifiers.join('+');
      setSettings((prev) => ({ ...prev, hotkey: newHotkey }));
      setRecordingHotkey(false);
    },
    [recordingHotkey]
  );

  useEffect(() => {
    if (recordingHotkey) {
      window.addEventListener('keydown', handleKeyDown);
      return () => window.removeEventListener('keydown', handleKeyDown);
    }
  }, [recordingHotkey, handleKeyDown]);

  return (
    <div className="flex h-full flex-col bg-background text-foreground">
      <div className="drag-area flex items-center justify-between border-b border-border p-4">
        <h2 className="text-lg font-semibold">Settings</h2>
        <button onClick={onClose} className="icon-button">
          <X size={18} />
        </button>
      </div>

      <div className="flex-1 space-y-6 overflow-y-auto p-4">
        <div className="space-y-3">
          <label className="block">
            <span className="text-sm font-medium">Theme</span>
          </label>
          <select
            value={settings.theme}
            onChange={(e) => setSettings({ ...settings, theme: e.target.value })}
            className="w-full rounded-lg border border-border bg-input px-3 py-2 text-sm text-foreground focus:outline-none focus:ring-2 focus:ring-ring"
          >
            <option value="dark">Dark</option>
            <option value="light">Light</option>
            <option value="system">System</option>
          </select>
        </div>

        <div className="space-y-3">
          <label className="block">
            <span className="text-sm font-medium">Storage Limit</span>
            <span className="ml-2 text-xs text-muted-foreground">({historySize} items stored)</span>
          </label>
          <input
            type="range"
            min="100"
            max="5000"
            step="100"
            value={settings.max_items}
            onChange={(e) => setSettings({ ...settings, max_items: parseInt(e.target.value) })}
            className="w-full accent-primary"
          />
          <div className="flex justify-between text-xs text-muted-foreground">
            <span>100 items</span>
            <span className="font-medium text-primary">{settings.max_items} items</span>
            <span>5000 items</span>
          </div>
        </div>

        <div className="space-y-3">
          <label className="block">
            <span className="text-sm font-medium">Auto-delete after</span>
          </label>
          <select
            value={settings.auto_delete_days}
            onChange={(e) =>
              setSettings({ ...settings, auto_delete_days: parseInt(e.target.value) })
            }
            className="w-full rounded-lg border border-border bg-input px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-ring"
          >
            <option value="7">7 days</option>
            <option value="14">14 days</option>
            <option value="30">30 days</option>
            <option value="60">60 days</option>
            <option value="90">90 days</option>
            <option value="365">1 year</option>
            <option value="0">Never</option>
          </select>
        </div>

        <div className="space-y-3">
          <label className="block">
            <span className="text-sm font-medium">Hotkey</span>
          </label>
          <button
            onClick={() => {
              setRecordingHotkey(true);
            }}
            className={`flex w-full items-center gap-2 rounded-lg border border-border bg-input px-3 py-2 text-sm transition-colors ${
              recordingHotkey ? 'border-primary ring-2 ring-primary' : ''
            }`}
          >
            {recordingHotkey ? (
              <span className="animate-pulse text-primary">Press any key...</span>
            ) : (
              <span>{settings.hotkey}</span>
            )}
          </button>
          <p className="text-xs text-muted-foreground">
            {recordingHotkey
              ? 'Press ESC to cancel'
              : 'Click to change, then press your new hotkey'}
          </p>
        </div>

        <div className="flex items-center justify-between">
          <div>
            <span className="text-sm font-medium">Startup with Windows</span>
            <p className="text-xs text-muted-foreground">Automatically start when Windows boots</p>
          </div>
          <button
            onClick={() =>
              setSettings({ ...settings, startup_with_windows: !settings.startup_with_windows })
            }
            className={`h-6 w-11 rounded-full transition-colors ${
              settings.startup_with_windows ? 'bg-primary' : 'bg-accent'
            }`}
          >
            <div
              className={`h-5 w-5 rounded-full bg-white shadow-sm transition-transform ${
                settings.startup_with_windows ? 'translate-x-5' : 'translate-x-0.5'
              }`}
            />
          </button>
        </div>

        <div className="border-t border-border pt-4">
          <button onClick={handleClearHistory} className="btn btn-destructive w-full">
            <Trash2 size={16} className="mr-2" />
            Clear All History
          </button>
        </div>

        <div className="space-y-2 border-t border-border pt-2">
          <p className="text-xs font-medium text-muted-foreground">Debug Tools</p>
          <div className="flex gap-2">
            <button
              onClick={async () => {
                if (confirm('Delete ALL clips? This cannot be undone.')) {
                  try {
                    await invoke('clear_all_clips');
                    setHistorySize(0);
                    alert('All clips deleted');
                  } catch (error) {
                    console.error(error);
                  }
                }
              }}
              className="btn btn-secondary flex-1 text-xs"
            >
              Clear All
            </button>
            <button
              onClick={async () => {
                try {
                  const count = await invoke<number>('remove_duplicate_clips');
                  alert(`Removed ${count} duplicate clips`);
                  const newSize = await invoke<number>('get_clipboard_history_size');
                  setHistorySize(newSize);
                } catch (error) {
                  console.error(error);
                }
              }}
              className="btn btn-secondary flex-1 text-xs"
            >
              Remove Duplicates
            </button>
          </div>
        </div>

        <div className="flex items-start gap-2 rounded-lg bg-accent/50 p-3">
          <Info size={16} className="mt-0.5 flex-shrink-0 text-muted-foreground" />
          <p className="text-xs text-muted-foreground">
            Items that in custom folders will never be auto-deleted. Use the pin feature to keep important
            clips permanently.
          </p>
        </div>
      </div>

      <div className="flex items-center justify-end gap-2 border-t border-border bg-background p-4">
        <button onClick={onClose} className="btn btn-secondary">
          Cancel
        </button>
        <button onClick={handleSave} className="btn btn-primary">
          <Save size={16} className="mr-2" />
          Save
        </button>
      </div>
    </div>
  );
}
