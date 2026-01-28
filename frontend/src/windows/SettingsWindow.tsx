import { useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { emit } from '@tauri-apps/api/event';
import { Settings } from '../types';
import { SettingsPanel } from '../components/SettingsPanel';
import { useTheme } from '../hooks/useTheme';

export function SettingsWindow() {
  const [settings, setSettings] = useState<Settings | null>(null);

  useTheme(settings?.theme || 'dark');

  useEffect(() => {
    invoke<Settings>('get_settings').then(setSettings).catch(console.error);
  }, []);

  const handleClose = async () => {
    const win = getCurrentWindow();
    try {
      await win.close();
    } catch (e) {
      console.error('Failed to close settings window:', e);
    }
  };

  const handleSave = async (newSettings: Settings) => {
    try {
      await invoke('save_settings', { settings: newSettings });
      await emit('settings-changed', newSettings);
      setSettings(newSettings);
      handleClose();
    } catch (error) {
      console.error('Failed to save settings:', error);
      alert(`Failed to save settings: ${error}`);
    }
  };

  if (!settings) {
    return <div className="flex h-screen items-center justify-center text-white">Loading...</div>;
  }

  return (
    <div className="h-screen bg-background text-foreground">
      <SettingsPanel settings={settings} onClose={handleClose} onSave={handleSave} />
    </div>
  );
}
