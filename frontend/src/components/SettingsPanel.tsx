import { Settings, FolderItem } from '../types';
import {
  X,
  Trash2,
  Plus,
  FolderOpen,
  Settings as SettingsIcon,
  BrainCircuit,
  Folder as FolderIcon,
  MoreHorizontal,
  Eye,
  EyeOff,
} from 'lucide-react';
import { useState, useEffect } from 'react';
import { isMacOS } from '../utils/platform';
import { useTheme } from '../hooks/useTheme';
import { useTranslation } from 'react-i18next';
import { invoke } from '@tauri-apps/api/core';
import { emit } from '@tauri-apps/api/event';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { getVersion } from '@tauri-apps/api/app';
import { openUrl } from '@tauri-apps/plugin-opener';
import { check } from '@tauri-apps/plugin-updater';
import { relaunch } from '@tauri-apps/plugin-process';
import { toast } from 'sonner';
import { ConfirmDialog } from './ConfirmDialog';
import { Select } from './ui/Select';
import { useShortcutRecorder } from 'use-shortcut-recorder';
import { clsx } from 'clsx';

interface SettingsPanelProps {
  settings: Settings;
  onClose: () => void;
}

type Tab = 'general' | 'ai' | 'folders';

function PromptEditor({
  label,
  value,
  titleValue,
  placeholder,
  onSave,
  onSaveTitle,
}: {
  label: string;
  value: string;
  titleValue?: string;
  placeholder: string;
  onSave: (val: string) => void;
  onSaveTitle?: (val: string) => void;
}) {
  const { t } = useTranslation();
  const [localValue, setLocalValue] = useState(value);
  const [localTitle, setLocalTitle] = useState(titleValue || label);

  // Sync with prop if it changes externally
  useEffect(() => {
    setLocalValue(value);
  }, [value]);

  useEffect(() => {
    setLocalTitle(titleValue || label);
  }, [titleValue, label]);

  return (
    <div className="space-y-2 rounded-lg border border-border/40 bg-accent/5 p-3">
      <div className="flex items-center justify-between gap-4">
        <input
          type="text"
          value={localTitle}
          onChange={(e) => setLocalTitle(e.target.value)}
          onBlur={() => {
            if (onSaveTitle && localTitle !== (titleValue || label)) {
              onSaveTitle(localTitle);
            }
          }}
          className="bg-transparent text-xs font-semibold text-foreground/70 outline-none transition-colors focus:text-primary"
          title="Click to rename action"
        />
        <span className="font-mono text-[10px] uppercase tracking-wider text-muted-foreground">
          {t('settings.actionName')}
        </span>
      </div>
      <textarea
        value={localValue}
        onChange={(e) => setLocalValue(e.target.value)}
        onBlur={() => {
          if (localValue !== value) {
            onSave(localValue);
          }
        }}
        placeholder={placeholder}
        className="min-h-[60px] w-full resize-none rounded-md border border-border bg-input px-3 py-2 text-xs text-foreground transition-all focus:outline-none focus:ring-1 focus:ring-primary/30"
      />
    </div>
  );
}

export function SettingsPanel({ settings: initialSettings, onClose }: SettingsPanelProps) {
  const [activeTab, setActiveTab] = useState<Tab>('general');
  const [settings, setSettings] = useState<Settings>(initialSettings);
  const [_historySize, setHistorySize] = useState<number>(0);
  const [isRecordingMode, setIsRecordingMode] = useState(false);
  const [showApiKey, setShowApiKey] = useState(false);
  const [localApiKey, setLocalApiKey] = useState(initialSettings.ai_api_key || '');
  const [localBaseUrl, setLocalBaseUrl] = useState(initialSettings.ai_base_url || '');
  const [localModel, setLocalModel] = useState(initialSettings.ai_model || 'gpt-3.5-turbo');
  const [isAccessibilityEnabled, setIsAccessibilityEnabled] = useState(false);

  // Folder Management State
  const [folders, setFolders] = useState<FolderItem[]>([]);
  const [newFolderName, setNewFolderName] = useState('');
  const [editingFolderId, setEditingFolderId] = useState<string | null>(null);
  const [renameValue, setRenameValue] = useState('');

  // Apply theme immediately when settings.theme changes
  useTheme(settings.theme);

  // i18n hook
  const { i18n, t } = useTranslation();

  // Generic handler for immediate settings updates
  const updateSettings = async (updates: Partial<Settings>) => {
    // Determine the next state before updating React state
    setSettings((prev) => {
      const newSettings = { ...prev, ...updates };

      // Schedule async actions - we use newSettings which is local to this scope
      // This avoids race conditions with 'settings' variable
      (async () => {
        try {
          await invoke('save_settings', { settings: newSettings });
          await emit('settings-changed', newSettings);

          if (updates.hotkey) {
            await invoke('register_global_shortcut', { hotkey: updates.hotkey });
          }
        } catch (error) {
          console.error(`Failed to save settings:`, error);
          toast.error(`Failed to save settings`);
        }
      })();

      // Feedback for changes
      const keys = Object.keys(updates);
      if (keys.length === 1) {
        const key = keys[0] as keyof Settings;
        const value = updates[key];
        if (key !== 'theme') {
          const label = key
            .split('_')
            .map((w) => w.charAt(0).toUpperCase() + w.slice(1))
            .join(' ');
          if (typeof value === 'boolean') {
            toast.success(`${label} was ${value ? 'enabled' : 'disabled'}`);
          } else {
            toast.success(`${label} updated`);
          }
        }
      } else if (keys.length > 1) {
        toast.success('Settings updated');
      }

      return newSettings;
    });
  };

  const updateSetting = (key: keyof Settings, value: any) => {
    updateSettings({ [key]: value });
  };

  const handleThemeChange = (newTheme: string) => {
    updateSetting('theme', newTheme);
  };

  const handleLanguageChange = (newLanguage: string) => {
    updateSetting('language', newLanguage);
    // Change language immediately
    i18n.changeLanguage(newLanguage);
    localStorage.setItem('pastepaw_language', newLanguage);
  };

  // Use use-shortcut-recorder for recording (shows current keys held in real-time)
  const {
    shortcut,
    savedShortcut,
    startRecording: startRecordingLib,
    stopRecording: stopRecordingLib,
    clearLastRecording,
  } = useShortcutRecorder({
    minModKeys: 1, // Require at least one modifier
  });

  // Start recording mode
  const handleStartRecording = () => {
    setIsRecordingMode(true);
    startRecordingLib();
  };

  const [ignoredApps, setIgnoredApps] = useState<string[]>([]);
  const [newIgnoredApp, setNewIgnoredApp] = useState('');
  const [appVersion, setAppVersion] = useState('');

  // Confirmation Dialog State
  const [confirmDialog, setConfirmDialog] = useState({
    isOpen: false,
    title: '',
    message: '',
    action: async () => {},
  });

  const loadFolders = async () => {
    try {
      const data = await invoke<FolderItem[]>('get_folders');
      setFolders(data);
    } catch (error) {
      console.error('Failed to load folders:', error);
    }
  };

  useEffect(() => {
    invoke<number>('get_clipboard_history_size').then(setHistorySize).catch(console.error);
    invoke<string[]>('get_ignored_apps').then(setIgnoredApps).catch(console.error);
    getVersion().then(setAppVersion).catch(console.error);
    loadFolders();

    // Check accessibility permissions on macOS
    if (isMacOS()) {
      const checkPermissions = async () => {
        try {
          const enabled = await invoke<boolean>('check_accessibility_permissions');
          setIsAccessibilityEnabled(enabled);
        } catch (e) {
          console.error('Failed to check accessibility permissions', e);
        }
      };
      
      checkPermissions();
      const interval = setInterval(checkPermissions, 2000);
      return () => clearInterval(interval);
    }
  }, []);

  const handleAddIgnoredApp = async () => {
    if (!newIgnoredApp.trim()) return;
    try {
      await invoke('add_ignored_app', { appName: newIgnoredApp.trim() });
      setIgnoredApps((prev) => [...prev, newIgnoredApp.trim()].sort());
      setNewIgnoredApp('');
      toast.success(`Added ${newIgnoredApp.trim()} to ignored apps`);
    } catch (e) {
      toast.error(`Failed to add ignored app: ${e}`);
      console.error(e);
    }
  };

  const handleBrowseFile = async () => {
    try {
      const path = await invoke<string>('pick_file');
      const filename = path.split(/[\\/]/).pop() || path;
      setNewIgnoredApp(filename);
    } catch (e) {
      console.log('File picker cancelled or failed', e);
    }
  };

  const handleRemoveIgnoredApp = async (app: string) => {
    try {
      await invoke('remove_ignored_app', { appName: app });
      setIgnoredApps((prev) => prev.filter((a) => a !== app));
      toast.success(`Removed ${app} from ignored apps`);
    } catch (e) {
      toast.error(`Failed to remove ignored app: ${e}`);
      console.error(e);
    }
  };

  const confirmClearHistory = () => {
    setConfirmDialog({
      isOpen: true,
      title: t('settings.clearHistoryTitle'),
      message: t('settings.clearHistoryMessage'),
      action: async () => {
        try {
          await invoke('clear_all_clips');
          setHistorySize(0);
          toast.success(t('settings.clearHistorySuccess'));
        } catch (error) {
          console.error('Failed to clear history:', error);
          toast.error(`Failed to clear history: ${error}`);
        }
      },
    });
  };

  // Folder Management Functions
  const handleCreateFolder = async () => {
    if (!newFolderName.trim()) return;
    try {
      await invoke('create_folder', { name: newFolderName.trim(), icon: null, color: null });
      setNewFolderName('');
      await loadFolders();
      toast.success('Folder created');
    } catch (e) {
      toast.error(`Failed to create folder: ${e}`);
    }
  };

  const handleDeleteFolder = async (id: string) => {
    try {
      await invoke('delete_folder', { id });
      await loadFolders();
      toast.success('Folder deleted');
    } catch (e) {
      toast.error(`Failed to delete folder: ${e}`);
    }
  };

  const startRenameFolder = (folder: FolderItem) => {
    setEditingFolderId(folder.id);
    setRenameValue(folder.name);
  };

  const saveRenameFolder = async () => {
    if (!editingFolderId || !renameValue.trim()) return;
    try {
      await invoke('rename_folder', { id: editingFolderId, name: renameValue.trim() });
      setEditingFolderId(null);
      setRenameValue('');
      await loadFolders();
      toast.success('Folder renamed');
    } catch (e) {
      toast.error(`Failed to rename folder: ${e}`);
    }
  };

  // Format shortcut array into Tauri-compatible string
  const formatHotkey = (keys: string[]): string => {
    return keys
      .map((k) => {
        if (k === 'Control') return 'Ctrl';
        if (k === 'Alt') return 'Alt';
        if (k === 'Shift') return 'Shift';
        if (k === 'Meta') return 'Cmd';
        if (k.startsWith('Key')) return k.slice(3);
        if (k.startsWith('Digit')) return k.slice(5);
        return k;
      })
      .join('+');
  };

  const handleSaveHotkey = async () => {
    if (savedShortcut.length > 0) {
      const newHotkey = formatHotkey(savedShortcut);
      await updateSetting('hotkey', newHotkey);
    }
    stopRecordingLib();
    setIsRecordingMode(false);
  };

  const handleCancelRecording = () => {
    stopRecordingLib();
    clearLastRecording();
    setIsRecordingMode(false);
  };

  const handleCheckUpdate = async () => {
    try {
      const loadingToast = toast.loading('Checking for updates...');
      const update = await check();
      toast.dismiss(loadingToast);

      if (update && update.available) {
        toast.info(`Update v${update.version} available!`, {
          duration: 10000,
          action: {
            label: 'Download & Restart',
            onClick: async () => {
              try {
                const dlToast = toast.loading(`Downloading v${update.version}...`);
                await update.downloadAndInstall();
                toast.dismiss(dlToast);
                toast.success('Update installed. Restarting...');
                await relaunch();
              } catch (e) {
                toast.error(`Update failed: ${e}`);
              }
            },
          },
        });
      } else {
        toast.success('You are on the latest version.');
      }
    } catch (e) {
      toast.error(`Check failed: ${e}`);
    }
  };

  return (
    <>
      <ConfirmDialog
        isOpen={confirmDialog.isOpen}
        title={confirmDialog.title}
        message={confirmDialog.message}
        onConfirm={async () => {
          await confirmDialog.action();
          setConfirmDialog((prev) => ({ ...prev, isOpen: false }));
        }}
        onCancel={() => setConfirmDialog((prev) => ({ ...prev, isOpen: false }))}
      />
      <div className="flex h-full flex-col select-none bg-background text-foreground">
        {/* Header */}
        <div
          className="flex items-center justify-between border-b border-border p-4"
          onMouseDown={(e) => {
            if (e.button === 0) {
              getCurrentWindow().startDragging();
            }
          }}
        >
          <h2 className="text-lg font-semibold">{t('settings.title')}</h2>
          <button
            onClick={onClose}
            className="icon-button"
            onMouseDown={(e) => e.stopPropagation()}
          >
            <X size={18} />
          </button>
        </div>

        <div className="flex flex-1 overflow-hidden">
          {/* Sidebar */}
          <div className="w-48 flex-shrink-0 border-r border-border bg-card/50 p-2">
            <div className="flex flex-col gap-1">
              <button
                onClick={() => setActiveTab('general')}
                className={clsx(
                  'flex items-center gap-2 rounded-md px-3 py-2 text-sm font-medium transition-colors',
                  activeTab === 'general'
                    ? 'bg-accent text-accent-foreground'
                    : 'text-muted-foreground hover:bg-accent/50 hover:text-foreground'
                )}
              >
                <SettingsIcon size={16} />
                {t('settings.general')}
              </button>
              <button
                onClick={() => setActiveTab('ai')}
                className={clsx(
                  'flex items-center gap-2 rounded-md px-3 py-2 text-sm font-medium transition-colors',
                  activeTab === 'ai'
                    ? 'bg-accent text-accent-foreground'
                    : 'text-muted-foreground hover:bg-accent/50 hover:text-foreground'
                )}
              >
                <BrainCircuit size={16} />
                {t('settings.ai')}
              </button>
              <button
                onClick={() => setActiveTab('folders')}
                className={clsx(
                  'flex items-center gap-2 rounded-md px-3 py-2 text-sm font-medium transition-colors',
                  activeTab === 'folders'
                    ? 'bg-accent text-accent-foreground'
                    : 'text-muted-foreground hover:bg-accent/50 hover:text-foreground'
                )}
              >
                <FolderIcon size={16} />
                {t('settings.folders')}
              </button>
            </div>
          </div>

          {/* Content Area */}
          <div className="flex-1 overflow-y-auto p-6">
            <div className="mx-auto max-w-2xl space-y-8">
              {/* --- GENERAL TAB --- */}
              {activeTab === 'general' && (
                <>
                  <section className="space-y-4">
                    <h3 className="text-sm font-medium text-muted-foreground">
                      {t('settings.appearanceBehavior')}
                    </h3>

                    <div className="grid grid-cols-2 gap-4">
                      <div className="space-y-3">
                        <label className="block">
                          <span className="text-sm font-medium">{t('settings.theme')}</span>
                        </label>
                        <Select
                          value={settings.theme}
                          onChange={handleThemeChange}
                          options={[
                            { value: 'dark', label: t('settings.themeDark') },
                            { value: 'light', label: t('settings.themeLight') },
                            { value: 'system', label: t('settings.themeSystem') },
                          ]}
                        />
                      </div>

                      <div className="space-y-3">
                        <label className="block">
                          <span className="text-sm font-medium">{t('settings.language')}</span>
                        </label>
                        <Select
                          value={settings.language || 'en'}
                          onChange={handleLanguageChange}
                          options={[
                            { value: 'en', label: 'English' },
                            { value: 'zh', label: '中文' },
                          ]}
                        />
                      </div>
                    </div>

                    {!isMacOS() && (
                      <div className="space-y-3">
                        <label className="block">
                          <span className="text-sm font-medium">{t('settings.windowEffect')}</span>
                        </label>
                        <Select
                          value={settings.mica_effect || 'clear'}
                          onChange={(val) => updateSetting('mica_effect', val)}
                          options={[
                            { value: 'mica_alt', label: 'Mica Alt' },
                            { value: 'mica', label: 'Mica' },
                            { value: 'clear', label: 'Clear' },
                          ]}
                        />
                      </div>
                    )}

                    <div className="flex items-center justify-between rounded-lg border border-border bg-accent/20 p-3">
                      <div>
                        <span className="text-sm font-medium">{isMacOS() ? t('settings.launchAtLogin') : t('settings.startupWithWindows')}</span>
                        <p className="text-xs text-muted-foreground">
                          {isMacOS() ? t('settings.launchAtLoginDesc') : t('settings.startupWithWindowsDesc')}
                        </p>
                      </div>
                      <button
                        onClick={() =>
                          updateSetting('startup_with_windows', !settings.startup_with_windows)
                        }
                        className={`h-6 w-11 rounded-full transition-colors ${settings.startup_with_windows ? 'bg-primary' : 'bg-accent'}`}
                      >
                        <div
                          className={`h-5 w-5 rounded-full bg-white shadow-sm transition-transform ${settings.startup_with_windows ? 'translate-x-5' : 'translate-x-0.5'}`}
                        />
                      </button>
                    </div>

                    {isMacOS() && !isAccessibilityEnabled && (
                      <div className="flex items-center justify-between rounded-lg border border-yellow-500/50 bg-yellow-500/10 p-3">
                        <div className="flex-1">
                          <span className="text-sm font-medium text-yellow-600 dark:text-yellow-500">{t('settings.accessibilityPermission')}</span>
                          <p className="text-xs text-muted-foreground">
                            {t('settings.accessibilityPermissionDesc')}
                          </p>
                        </div>
                        <button
                          onClick={() => invoke('request_accessibility_permissions')}
                          className="rounded bg-yellow-500 px-3 py-1.5 text-xs font-semibold text-white transition-colors hover:bg-yellow-600"
                        >
                          {t('settings.grantPermission')}
                        </button>
                      </div>
                    )}

                    <div className="flex items-center justify-between rounded-lg border border-border bg-accent/20 p-3">
                      <div>
                        <span className="text-sm font-medium">{t('settings.autoPaste')}</span>
                        <p className="text-xs text-muted-foreground">
                          {t('settings.autoPasteDesc')}
                        </p>
                      </div>
                      <button
                        onClick={() => updateSetting('auto_paste', !settings.auto_paste)}
                        className={`h-6 w-11 rounded-full transition-colors ${settings.auto_paste ? 'bg-primary' : 'bg-accent'}`}
                      >
                        <div
                          className={`h-5 w-5 rounded-full bg-white shadow-sm transition-transform ${settings.auto_paste ? 'translate-x-5' : 'translate-x-0.5'}`}
                        />
                      </button>
                    </div>

                    <div className="flex items-center justify-between rounded-lg border border-border bg-accent/20 p-3">
                      <div>
                        <span className="text-sm font-medium">{t('settings.ignoreGhostClips')}</span>
                        <p className="text-xs text-muted-foreground">
                          {t('settings.ignoreGhostClipsDesc')}
                        </p>
                      </div>
                      <button
                        onClick={() =>
                          updateSetting('ignore_ghost_clips', !settings.ignore_ghost_clips)
                        }
                        className={`h-6 w-11 rounded-full transition-colors ${settings.ignore_ghost_clips ? 'bg-primary' : 'bg-accent'}`}
                      >
                        <div
                          className={`h-5 w-5 rounded-full bg-white shadow-sm transition-transform ${settings.ignore_ghost_clips ? 'translate-x-5' : 'translate-x-0.5'}`}
                        />
                      </button>
                    </div>
                  </section>

                  <section className="space-y-4">
                    <h3 className="text-sm font-medium text-muted-foreground">{t('settings.shortcuts')}</h3>
                    <div className="space-y-3">
                      <label className="block">
                        <span className="text-sm font-medium">{t('settings.hotkey')}</span>
                        <p className="text-xs text-muted-foreground">{t('settings.hotkeyDesc')}</p>
                      </label>
                      {isRecordingMode ? (
                        <div className="space-y-2">
                          <div className="flex w-full items-center gap-2 rounded-lg border border-primary bg-input px-3 py-2 text-sm ring-2 ring-primary">
                            <span className="animate-pulse text-primary">
                              {shortcut.length > 0
                                ? formatHotkey(shortcut)
                                : savedShortcut.length > 0
                                  ? formatHotkey(savedShortcut)
                                  : t('settings.hotkeyRecording')}
                            </span>
                          </div>
                          <div className="flex gap-2">
                            <button
                              onClick={handleSaveHotkey}
                              disabled={savedShortcut.length === 0}
                              className="rounded bg-primary px-3 py-1 text-xs text-primary-foreground disabled:opacity-50"
                            >
                              {t('common.save')}
                            </button>
                            <button
                              onClick={handleCancelRecording}
                              className="rounded bg-muted px-3 py-1 text-xs text-muted-foreground"
                            >
                              {t('common.cancel')}
                            </button>
                          </div>
                        </div>
                      ) : (
                        <button
                          onClick={handleStartRecording}
                          className="flex w-full items-center gap-2 rounded-lg border border-border bg-input px-3 py-2 text-sm transition-colors hover:border-primary"
                        >
                          <span className="rounded bg-accent px-2 py-0.5 font-mono text-xs font-medium">
                            {settings.hotkey}
                          </span>
                        </button>
                      )}
                    </div>
                  </section>

                  <section className="space-y-4">
                    <h3 className="text-sm font-medium text-muted-foreground">
                      {t('settings.privacyExceptions')}
                    </h3>
                    <div className="space-y-3">
                      <label className="block">
                        <span className="text-sm font-medium">{t('settings.ignoredApps')}</span>
                        <p className="text-xs text-muted-foreground">
                          {t('settings.ignoredAppsDesc')}
                        </p>
                      </label>

                      <div className="flex gap-2">
                        <input
                          type="text"
                          value={newIgnoredApp}
                          onChange={(e) => setNewIgnoredApp(e.target.value)}
                          placeholder={isMacOS() ? 'e.g. Safari' : 'e.g. notepad.exe'}
                          className="flex-1 rounded-lg border border-border bg-input px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-ring"
                          onKeyDown={(e) => e.key === 'Enter' && handleAddIgnoredApp()}
                        />
                        <button
                          onClick={handleBrowseFile}
                          className="btn btn-secondary px-3"
                          title="Browse executable"
                        >
                          <FolderOpen size={16} />
                        </button>
                        <button
                          onClick={handleAddIgnoredApp}
                          disabled={!newIgnoredApp.trim()}
                          className="btn btn-secondary px-3"
                          title="Add to list"
                        >
                          <Plus size={16} />
                        </button>
                      </div>

                      <div className="max-h-40 space-y-1 overflow-y-auto pr-1">
                        {ignoredApps.length === 0 ? (
                          <div className="rounded-lg border border-dashed border-border p-4 text-center">
                            <p className="text-xs text-muted-foreground">{t('settings.noIgnoredApps')}</p>
                          </div>
                        ) : (
                          ignoredApps.map((app) => (
                            <div
                              key={app}
                              className="group flex items-center justify-between rounded-md border border-transparent bg-accent/30 px-3 py-2 text-sm hover:border-border hover:bg-accent/50"
                            >
                              <span className="font-mono text-xs">{app}</span>
                              <button
                                onClick={() => handleRemoveIgnoredApp(app)}
                                className="text-muted-foreground opacity-0 transition-opacity hover:text-destructive group-hover:opacity-100"
                              >
                                <X size={14} />
                              </button>
                            </div>
                          ))
                        )}
                      </div>
                    </div>
                  </section>

                  <section className="space-y-4">
                    <h3 className="text-sm font-medium text-red-500/80">{t('settings.dataManagement')}</h3>
                    <div className="grid grid-cols-2 gap-3">
                      <button
                        onClick={confirmClearHistory}
                        className="btn border border-destructive/20 bg-destructive/10 text-destructive hover:bg-destructive/20"
                      >
                        <Trash2 size={16} className="mr-2" />
                        {t('settings.clearHistory')}
                      </button>

                      <button
                        onClick={async () => {
                          try {
                            const count = await invoke<number>('remove_duplicate_clips');
                            toast.success(t('settings.removeDuplicatesSuccess', { count }));
                            const newSize = await invoke<number>('get_clipboard_history_size');
                            setHistorySize(newSize);
                          } catch (error) {
                            console.error(error);
                            toast.error(`Failed to remove duplicates: ${error}`);
                          }
                        }}
                        className="btn btn-secondary text-xs"
                      >
                        {t('settings.removeDuplicates')}
                      </button>
                    </div>
                  </section>
                </>
              )}

              {/* --- AI PROCESSING TAB --- */}
              {activeTab === 'ai' && (
                <>
                  <section className="space-y-4">
                    <h3 className="text-sm font-medium text-muted-foreground">{t('settings.aiConfiguration')}</h3>

                    <div className="space-y-3">
                      <label className="block">
                        <span className="text-sm font-medium">{t('settings.provider')}</span>
                      </label>
                      <Select
                        value={settings.ai_provider || 'openai'}
                        onChange={(newProvider) => {
                          const updates: Partial<Settings> = { ai_provider: newProvider };

                          // Auto-fill Base URL and Model based on provider
                          if (newProvider === 'openai') {
                            updates.ai_base_url = 'https://api.openai.com/v1';
                            setLocalBaseUrl('https://api.openai.com/v1');
                          } else if (newProvider === 'deepseek') {
                            updates.ai_base_url = 'https://api.deepseek.com';
                            updates.ai_model = 'deepseek-chat';
                            setLocalBaseUrl('https://api.deepseek.com');
                            setLocalModel('deepseek-chat');
                          }

                          updateSettings(updates);
                        }}
                        options={[
                          { value: 'openai', label: t('settings.providerOpenAI') },
                          { value: 'deepseek', label: t('settings.providerDeepSeek') },
                          { value: 'custom', label: t('settings.providerCustom') },
                        ]}
                      />
                    </div>

                    <div className="space-y-3">
                      <label className="block">
                        <span className="text-sm font-medium">{t('settings.apiKey')}</span>
                      </label>
                      <div className="relative">
                        <input
                          type={showApiKey ? 'text' : 'password'}
                          value={localApiKey}
                          onChange={(e) => setLocalApiKey(e.target.value)}
                          onBlur={() => updateSetting('ai_api_key', localApiKey)}
                          placeholder="sk-..."
                          className="w-full rounded-lg border border-border bg-input py-2 pl-3 pr-10 text-sm focus:outline-none focus:ring-2 focus:ring-ring"
                        />
                        <button
                          type="button"
                          onClick={() => setShowApiKey(!showApiKey)}
                          className="absolute right-3 top-1/2 -translate-y-1/2 text-muted-foreground transition-colors hover:text-foreground"
                        >
                          {showApiKey ? <EyeOff size={16} /> : <Eye size={16} />}
                        </button>
                      </div>
                    </div>

                    <div className="space-y-3">
                      <label className="block">
                        <span className="text-sm font-medium">{t('settings.model')}</span>
                      </label>
                      <input
                        type="text"
                        value={localModel}
                        onChange={(e) => setLocalModel(e.target.value)}
                        onBlur={() => updateSetting('ai_model', localModel)}
                        placeholder="gpt-4o, deepseek-chat, etc."
                        className="w-full rounded-lg border border-border bg-input px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-ring"
                      />
                    </div>

                    <div className="space-y-3">
                      <label className="block">
                        <span className="text-sm font-medium">{t('settings.baseUrl')}</span>
                      </label>
                      <input
                        type="text"
                        value={localBaseUrl}
                        onChange={(e) => setLocalBaseUrl(e.target.value)}
                        onBlur={() => updateSetting('ai_base_url', localBaseUrl)}
                        placeholder="https://api.openai.com/v1"
                        className="w-full rounded-lg border border-border bg-input px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-ring"
                      />
                    </div>
                  </section>

                  <section className="space-y-4 border-t border-border/50 pt-4">
                    <h3 className="text-sm font-medium text-muted-foreground">{t('settings.customPrompts')}</h3>
                    <p className="text-xs italic text-muted-foreground">
                      {t('settings.customPromptsDesc')}
                    </p>

                    <div className="space-y-4">
                      <PromptEditor
                        label={t('settings.aiSummarize')}
                        value={settings.ai_prompt_summarize || ''}
                        titleValue={settings.ai_title_summarize}
                        onSave={(val) => updateSetting('ai_prompt_summarize', val)}
                        onSaveTitle={(val) => updateSetting('ai_title_summarize', val)}
                        placeholder={t('settings.aiSummarizePlaceholder')}
                      />

                      <PromptEditor
                        label={t('settings.aiTranslate')}
                        value={settings.ai_prompt_translate || ''}
                        titleValue={settings.ai_title_translate}
                        onSave={(val) => updateSetting('ai_prompt_translate', val)}
                        onSaveTitle={(val) => updateSetting('ai_title_translate', val)}
                        placeholder={t('settings.aiTranslatePlaceholder')}
                      />

                      <PromptEditor
                        label={t('settings.aiExplainCode')}
                        value={settings.ai_prompt_explain_code || ''}
                        titleValue={settings.ai_title_explain_code}
                        onSave={(val) => updateSetting('ai_prompt_explain_code', val)}
                        onSaveTitle={(val) => updateSetting('ai_title_explain_code', val)}
                        placeholder={t('settings.aiExplainCodePlaceholder')}
                      />

                      <PromptEditor
                        label={t('settings.aiFixGrammar')}
                        value={settings.ai_prompt_fix_grammar || ''}
                        titleValue={settings.ai_title_fix_grammar}
                        onSave={(val) => updateSetting('ai_prompt_fix_grammar', val)}
                        onSaveTitle={(val) => updateSetting('ai_title_fix_grammar', val)}
                        placeholder={t('settings.aiFixGrammarPlaceholder')}
                      />
                    </div>
                  </section>
                </>
              )}

              {/* --- FOLDERS TAB --- */}
              {activeTab === 'folders' && (
                <section className="space-y-4">
                  <h3 className="text-sm font-medium text-muted-foreground">{t('settings.manageFolders')}</h3>

                  <div className="flex gap-2">
                    <input
                      type="text"
                      value={newFolderName}
                      onChange={(e) => setNewFolderName(e.target.value)}
                      placeholder={t('settings.newFolderPlaceholder')}
                      className="flex-1 rounded-lg border border-border bg-input px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-ring"
                      onKeyDown={(e) => e.key === 'Enter' && handleCreateFolder()}
                    />
                    <button
                      onClick={handleCreateFolder}
                      disabled={!newFolderName.trim()}
                      className="btn btn-secondary px-3"
                    >
                      <Plus size={16} className="mr-1" />
                      {t('settings.add')}
                    </button>
                  </div>

                  <div className="mt-4 space-y-2">
                    {folders.filter((f) => !f.is_system).length === 0 ? (
                      <p className="rounded-lg border border-dashed border-border py-4 text-center text-xs text-muted-foreground">
                        {t('settings.noFolders')}
                      </p>
                    ) : (
                      folders
                        .filter((f) => !f.is_system)
                        .map((folder) => (
                          <div
                            key={folder.id}
                            className="flex items-center justify-between rounded-lg border border-border bg-card p-3"
                          >
                            {editingFolderId === folder.id ? (
                              <div className="flex flex-1 items-center gap-2">
                                <input
                                  type="text"
                                  value={renameValue}
                                  onChange={(e) => setRenameValue(e.target.value)}
                                  className="flex-1 rounded-md border border-input bg-background px-2 py-1 text-sm"
                                  autoFocus
                                  onKeyDown={(e) => {
                                    if (e.key === 'Enter') saveRenameFolder();
                                    if (e.key === 'Escape') setEditingFolderId(null);
                                  }}
                                />
                                <button
                                  onClick={saveRenameFolder}
                                  className="text-xs text-primary hover:underline"
                                >
                                  {t('common.save')}
                                </button>
                                <button
                                  onClick={() => setEditingFolderId(null)}
                                  className="text-xs text-muted-foreground hover:underline"
                                >
                                  {t('common.cancel')}
                                </button>
                              </div>
                            ) : (
                              <>
                                <div className="flex items-center gap-3">
                                  <FolderIcon size={16} className="text-blue-400" />
                                  <span className="text-sm font-medium">{folder.name}</span>
                                  <span className="text-xs text-muted-foreground">
                                    ({folder.item_count} items)
                                  </span>
                                </div>
                                <div className="flex items-center gap-2">
                                  <button
                                    onClick={() => startRenameFolder(folder)}
                                    className="rounded p-1 text-muted-foreground hover:bg-accent hover:text-foreground"
                                    title="Rename"
                                  >
                                    <MoreHorizontal size={14} />
                                  </button>
                                  <button
                                    onClick={() => handleDeleteFolder(folder.id)}
                                    className="rounded p-1 text-muted-foreground hover:bg-destructive/10 hover:text-destructive"
                                    title="Delete"
                                  >
                                    <Trash2 size={14} />
                                  </button>
                                </div>
                              </>
                            )}
                          </div>
                        ))
                    )}
                  </div>
                </section>
              )}
            </div>
          </div>
        </div>

        {/* Footer */}
        <div className="flex flex-col items-center gap-1 border-t border-border bg-background px-4 py-3 text-center">
          <button
            onClick={() => openUrl('https://github.com/XueshiQiao/PastePaw').catch(console.error)}
            className="text-xs text-muted-foreground transition-colors hover:text-foreground"
          >
            PastePaw {appVersion || '...'}
          </button>
          <div className="flex gap-2 text-xs text-muted-foreground">
            <span>© 2025</span>
            <span>•</span>
            <button onClick={handleCheckUpdate} className="underline hover:text-foreground">
              {t('settings.checkForUpdates')}
            </button>
          </div>
        </div>
      </div>
    </>
  );
}
