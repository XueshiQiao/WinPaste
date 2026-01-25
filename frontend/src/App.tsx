import { useEffect, useState, useCallback, useRef } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { ClipboardItem, FolderItem, Settings } from './types';
import { ClipList } from './components/ClipList';
import { SearchBar } from './components/SearchBar';
import { SettingsPanel } from './components/SettingsPanel';
import { ControlBar } from './components/ControlBar';
import { useKeyboard } from './hooks/useKeyboard';

function App() {
  const [clips, setClips] = useState<ClipboardItem[]>([]);
  const [folders, setFolders] = useState<FolderItem[]>([]);
  const [selectedFolder, setSelectedFolder] = useState<string | null>(null);
  const [searchQuery, setSearchQuery] = useState('');
  const [showSearch, setShowSearch] = useState(false);
  const [selectedClipId, setSelectedClipId] = useState<string | null>(null);
  const [showSettings, setShowSettings] = useState(false);
  const [settings, setSettings] = useState<Settings>({
    max_items: 1000,
    auto_delete_days: 30,
    startup_with_windows: true,
    show_in_taskbar: false,
    hotkey: 'Ctrl+Alt+V',
    theme: 'dark',
  });
  const [isLoading, setIsLoading] = useState(true);

  const window = getCurrentWindow();
  const selectedFolderRef = useRef(selectedFolder);
  selectedFolderRef.current = selectedFolder;

  const loadClips = useCallback(async (folderId: string | null) => {
    try {
      console.log('loadClips called with folderId:', folderId);
      setIsLoading(true);
      const data = await invoke<ClipboardItem[]>('get_clips', {
        filterId: folderId,
        limit: 100,
        offset: 0,
      });
      console.log('Clips loaded:', data.length);
      setClips(data);
    } catch (error) {
      console.error('Failed to load clips:', error);
    } finally {
      setTimeout(() => setIsLoading(false), 100);
    }
  }, []);

  const loadFolders = useCallback(async () => {
    try {
      console.log('Loading folders...');
      const data = await invoke<FolderItem[]>('get_folders');
      console.log('Folders loaded:', data);
      setFolders(data);
    } catch (error) {
      console.error('Failed to load folders:', error);
    }
  }, []);

  const refreshCurrentFolder = useCallback(() => {
    console.log('Refreshing folder:', selectedFolderRef.current);
    loadClips(selectedFolderRef.current);
  }, [loadClips]);

  useEffect(() => {
    console.log('Loading settings...');
    invoke<Settings>('get_settings')
      .then(setSettings)
      .catch(console.error);
  }, []);

  useEffect(() => {
    console.log('Folder changed to:', selectedFolder);
    loadFolders();
    loadClips(selectedFolder);
  }, [selectedFolder, loadFolders, loadClips]);

  useEffect(() => {
    console.log('Setting up clipboard listener');
    const unlistenClipboard = listen('clipboard-change', () => {
      console.log('Clipboard changed event received');
      refreshCurrentFolder();
    });

    return () => {
      unlistenClipboard.then((unlisten) => {
        if (typeof unlisten === 'function') unlisten();
      });
    };
  }, [refreshCurrentFolder]);

  useKeyboard({
    onClose: () => window.hide(),
    onSearch: () => setShowSearch(true),
    onDelete: () => handleDelete(selectedClipId),
    onPin: () => handlePin(selectedClipId),
  });

  const handleSearch = async (query: string) => {
    setSearchQuery(query);
    if (query.trim()) {
      try {
        const data = await invoke<ClipboardItem[]>('search_clips', { query, limit: 100 });
        setClips(data);
      } catch (error) {
        console.error('Failed to search clips:', error);
      }
    } else {
      loadClips(selectedFolder);
    }
  };

  const handleDelete = async (clipId: string | null) => {
    if (!clipId) return;
    try {
      await invoke('delete_clip', { id: clipId, hardDelete: false });
      setClips(clips.filter(c => c.id !== clipId));
      setSelectedClipId(null);
    } catch (error) {
      console.error('Failed to delete clip:', error);
    }
  };

  const handlePin = async (clipId: string | null) => {
    if (!clipId) return;
    const clip = clips.find(c => c.id === clipId);
    if (!clip) return;

    try {
      if (clip.is_pinned) {
        await invoke('unpin_clip', { id: clipId });
      } else {
        await invoke('pin_clip', { id: clipId });
      }
      setClips(prev => {
        const updated = prev.map(c =>
          c.id === clipId ? { ...c, is_pinned: !c.is_pinned } : c
        );
        return [...updated.filter(c => c.is_pinned), ...updated.filter(c => !c.is_pinned)];
      });
    } catch (error) {
      console.error('Failed to pin/unpin clip:', error);
    }
  };

  const handlePaste = async (clipId: string) => {
    try {
      await invoke('paste_clip', { id: clipId });
      window.hide();
    } catch (error) {
      console.error('Failed to paste clip:', error);
    }
  };

  const handleCopy = async (clipId: string) => {
    const clip = clips.find(c => c.id === clipId);
    if (clip) {
      try {
        await navigator.clipboard.writeText(clip.content);
      } catch (error) {
        console.error('Failed to copy to clipboard:', error);
      }
    }
  };

  const handleCreateFolder = async (name: string) => {
    try {
      await invoke('create_folder', { name, icon: null, color: null });
      await loadFolders();
    } catch (error) {
      console.error('Failed to create folder:', error);
    }
  };

  return (
    <div className="flex flex-col h-screen bg-[#1E1E1E] text-white overflow-hidden font-sans">
      <ControlBar
        folders={folders}
        selectedFolder={selectedFolder}
        onSelectFolder={setSelectedFolder}
        onSearchClick={() => setShowSearch(!showSearch)}
        onAddClick={() => {
           // For now, prompt for new folder creation as a placeholder for "Add"
           const name = prompt("Enter new folder name:");
           if (name) handleCreateFolder(name);
        }}
        onMoreClick={() => setShowSettings(true)}
      />

      {showSearch && (
        <div className="p-4 border-b border-gray-800 bg-[#252526]">
          <SearchBar
            query={searchQuery}
            onQueryChange={handleSearch}
            onClear={() => {
              setSearchQuery('');
              loadClips(selectedFolder);
              setShowSearch(false);
            }}
          />
        </div>
      )}

      <main className="flex-1 overflow-hidden relative no-scrollbar">
        <ClipList
          clips={clips}
          isLoading={isLoading}
          selectedClipId={selectedClipId}
          onSelectClip={setSelectedClipId}
          onPaste={handlePaste}
          onCopy={handleCopy}
          onDelete={handleDelete}
          onPin={handlePin}
        />
      </main>

      {showSettings && (
        <SettingsPanel
          settings={settings}
          onClose={() => setShowSettings(false)}
          onSave={async (newSettings) => {
            await invoke('save_settings', { settings: newSettings });
            setSettings(newSettings);
            setShowSettings(false);
          }}
        />
      )}
    </div>
  );
}

export default App;
