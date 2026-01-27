import { useEffect, useState, useCallback, useRef } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { WebviewWindow } from '@tauri-apps/api/webviewWindow';
import { ClipboardItem, FolderItem, Settings } from './types';
import { ClipList } from './components/ClipList';
import { ControlBar } from './components/ControlBar';
import { useKeyboard } from './hooks/useKeyboard';
import { useTheme } from './hooks/useTheme';

function App() {
  const [clips, setClips] = useState<ClipboardItem[]>([]);
  const [folders, setFolders] = useState<FolderItem[]>([]);
  const [selectedFolder, setSelectedFolder] = useState<string | null>(null);
  const [searchQuery, setSearchQuery] = useState('');
  const [showSearch, setShowSearch] = useState(false);
  const [selectedClipId, setSelectedClipId] = useState<string | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [hasMore, setHasMore] = useState(true);
  const [theme, setTheme] = useState('dark');

  useTheme(theme);

  const window = getCurrentWindow();
  const selectedFolderRef = useRef(selectedFolder);
  selectedFolderRef.current = selectedFolder;

  useEffect(() => {
    invoke<Settings>('get_settings').then(s => setTheme(s.theme)).catch(console.error);

    // Listen for setting changes from the settings window
    const unlisten = listen<Settings>('settings-changed', (event) => {
        setTheme(event.payload.theme);
    });
    return () => {
        unlisten.then(f => f());
    };
  }, []);

  const openSettings = useCallback(async () => {
    // Check if settings window already exists
    const existingWin = await WebviewWindow.getByLabel('settings');
    if (existingWin) {
      await existingWin.setFocus();
      return;
    }

    const settingsWin = new WebviewWindow('settings', {
      url: 'index.html?window=settings',
      title: 'Settings',
      width: 500,
      height: 700,
      resizable: false,
      decorations: false, // We have our own title bar in SettingsPanel
      center: true,
    });

    settingsWin.once('tauri://created', function () {
      console.log('Settings window created');
    });

    settingsWin.once('tauri://error', function (e) {
      console.error('Error creating settings window', e);
    });
  }, []);

  const loadClips = useCallback(async (folderId: string | null, append: boolean = false) => {
    try {
      console.log('loadClips START | folderId:', folderId, 'append:', append, 'currentClips:', clips.length);
      setIsLoading(true);

      const currentOffset = append ? clips.length : 0;
      console.log('Fetching with offset:', currentOffset);

      const data = await invoke<ClipboardItem[]>('get_clips', {
        filterId: folderId,
        limit: 20, // Reverted to 20 per user request
        offset: currentOffset,
        previewOnly: false, // Load full image data directly
      });

      console.log('Clips loaded:', data.length);

      if (append) {
        setClips(prev => {
           console.log('Appending clips. Prev:', prev.length, 'New:', data.length);
           return [...prev, ...data];
        });
      } else {
        console.log('Setting new clips');
        setClips(data);
      }

      // If we got fewer than limit, no more clips
      setHasMore(data.length === 20);
    } catch (error) {
      console.error('Failed to load clips:', error);
    } finally {
      setIsLoading(false);
    }
  }, [clips.length]);

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
    console.log('Folder changed to:', selectedFolder);
    loadFolders();
    if (searchQuery.trim()) {
        handleSearch(searchQuery);
    } else {
        loadClips(selectedFolder);
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [selectedFolder]);

  useEffect(() => {
    console.log('Setting up clipboard listener');
    const unlistenClipboard = listen('clipboard-change', (event) => {
      console.log('Clipboard changed event received:', event);
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
        const data = await invoke<ClipboardItem[]>('search_clips', {
            query,
            filterId: selectedFolder,
            limit: 100
        });
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

  const loadMore = useCallback(() => {
    if (hasMore && !isLoading) {
      loadClips(selectedFolder, true);
    }
  }, [hasMore, isLoading, selectedFolder, loadClips]);

  return (
    <div className="flex flex-col h-screen bg-background text-foreground overflow-hidden font-sans">
      <ControlBar
        folders={folders}
        selectedFolder={selectedFolder}
        onSelectFolder={setSelectedFolder}
        showSearch={showSearch}
        searchQuery={searchQuery}
        onSearchChange={handleSearch}
        onSearchClick={() => {
          if (showSearch) {
            handleSearch(""); // Clear search when closing
          }
          setShowSearch(!showSearch);
        }}
        onAddClick={() => {
           // For now, prompt for new folder creation as a placeholder for "Add"
           const name = prompt("Enter new folder name:");
           if (name) handleCreateFolder(name);
        }}
        onMoreClick={openSettings}
      />

      <main className="flex-1 relative no-scrollbar">
        <ClipList
          clips={clips}
          isLoading={isLoading}
          hasMore={hasMore}
          selectedClipId={selectedClipId}
          onSelectClip={setSelectedClipId}
          onPaste={handlePaste}
          onCopy={handleCopy}
          onDelete={handleDelete}
          onPin={handlePin}
          onLoadMore={loadMore}
        />
      </main>
    </div>
  );
}

export default App;
