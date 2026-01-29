import { useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { emit } from '@tauri-apps/api/event';
import { Settings } from '../types';
import { SettingsPanel } from '../components/SettingsPanel';

import { Toaster } from 'sonner';

export function SettingsWindow() {
  const [settings, setSettings] = useState<Settings | null>(null);

  // useTheme(settings?.theme || 'dark'); // Moved to SettingsPanel for immediate feedback

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
      // Alert will be handled inside SettingsPanel or we can toast here if we pass toast down?
      // Actually SettingsPanel calls onSave.
      // If we want toast here, we need separate toast call.
      // But SettingsPanel handles most logic.
      // Let's modify SettingsPanel to handle the "Close" triggers?
      // Or just keep this as is, but add Toaster.
      // Wait, SettingsPanel calls onSave, and handleSave calls handleClose.
      // So Success toast might not be seen if window closes immediately.
      // Unless we wait?
      // "Saved" toast usually implies staying open.
      // But "Save" button usually closes settings in this app?
    }
  };

  if (!settings) {
    return <div className="flex h-screen items-center justify-center text-white">Loading...</div>;
  }

  return (
    <div className="h-screen bg-background text-foreground">
      <SettingsPanel settings={settings} onClose={handleClose} onSave={handleSave} />
      <Toaster
        richColors
        position="bottom-center"
        theme={settings.theme === 'light' ? 'light' : 'dark'}
      />
    </div>
  );
}
