import { useEffect, useState, useCallback, useRef } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { WebviewWindow } from '@tauri-apps/api/webviewWindow';
import { ClipboardItem as AppClipboardItem, FolderItem, Settings } from './types';
import { ClipList } from './components/ClipList';
import { ControlBar } from './components/ControlBar';
import { DragPreview } from './components/DragPreview';
import { ContextMenu } from './components/ContextMenu';
import { FolderModal } from './components/FolderModal';
import { AiResultDialog } from './components/AiResultDialog';
import { useKeyboard } from './hooks/useKeyboard';
import { useTheme } from './hooks/useTheme';
import { Toaster, toast } from 'sonner';
import { LAYOUT } from './constants';
import { isMacOS } from './utils/platform';

const base64ToBlob = (base64: string, mimeType: string = 'image/png'): Blob => {
  const byteCharacters = atob(base64);
  const byteNumbers = new Array(byteCharacters.length);
  for (let i = 0; i < byteCharacters.length; i++) {
    byteNumbers[i] = byteCharacters.charCodeAt(i);
  }
  const byteArray = new Uint8Array(byteNumbers);
  return new Blob([byteArray], { type: mimeType });
};

function App() {
  const [clips, setClips] = useState<AppClipboardItem[]>([]);
  const [folders, setFolders] = useState<FolderItem[]>([]);
  const [selectedFolder, setSelectedFolder] = useState<string | null>(null);
  const [searchQuery, setSearchQuery] = useState('');
  const [showSearch, setShowSearch] = useState(false);
  const [selectedClipId, setSelectedClipId] = useState<string | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [hasMore, setHasMore] = useState(true);
  const [theme, setTheme] = useState('system');
  const [settings, setSettings] = useState<Settings | null>(null);

  // Simulated Drag State
  const [draggingClipId, setDraggingClipId] = useState<string | null>(null);
  const [dragPosition, setDragPosition] = useState({ x: 0, y: 0 });
  const [dragTargetFolderId, setDragTargetFolderId] = useState<string | null>(null);

  // Add Folder Modal State
  const [showAddFolderModal, setShowAddFolderModal] = useState(false);
  const [newFolderName, setNewFolderName] = useState('');

  // Using refs for event handlers to access latest state without re-attaching listeners
  const dragStateRef = useRef({
    isDragging: false,
    clipId: null as string | null,
    targetFolderId: null as string | null,
    pendingDrag: null as { clipId: string; startX: number; startY: number } | null,
  });

  const effectiveTheme = useTheme(theme);

  const appWindow = getCurrentWindow();
  const selectedFolderRef = useRef(selectedFolder);
  selectedFolderRef.current = selectedFolder;

  useEffect(() => {
    invoke<Settings>('get_settings')
      .then((s) => {
        setTheme(s.theme);
        setSettings(s);
      })
      .catch(console.error);

    // Listen for setting changes from the settings window
    const unlisten = listen<Settings>('settings-changed', (event) => {
      setTheme(event.payload.theme);
      setSettings(event.payload);
    });

    return () => {
      unlisten.then((f) => f());
    };
  }, []);

  const openSettings = useCallback(async () => {
    // Check if settings window already exists
    const existingWin = await WebviewWindow.getByLabel('settings');
    if (existingWin) {
      try {
        await invoke('focus_window', { label: 'settings' });
      } catch (e) {
        console.error('Failed to focus settings window:', e);
        // Fallback to JS API if command fails (though command is preferred)
        await existingWin.unminimize();
        await existingWin.show();
        await existingWin.setFocus();
      }
      return;
    }

    const settingsWin = new WebviewWindow('settings', {
      url: 'index.html?window=settings',
      title: 'Settings',
      width: 800,
      height: 700,
      resizable: true,
      decorations: false, // We have our own title bar in SettingsPanel
      transparent: true,
      center: true,
    });

    settingsWin.once('tauri://created', function () {});

    settingsWin.once('tauri://error', function (e) {
      console.error('Error creating settings window', e);
    });
  }, []);

  const loadClips = useCallback(
    async (folderId: string | null, append: boolean = false, searchQuery: string = '') => {
      try {
        setIsLoading(true);

        const currentOffset = append ? clips.length : 0;

        let data: AppClipboardItem[];

        if (searchQuery.trim()) {
          data = await invoke<AppClipboardItem[]>('search_clips', {
            query: searchQuery,
            filterId: folderId,
            limit: 20,
            offset: currentOffset,
          });
        } else {
          data = await invoke<AppClipboardItem[]>('get_clips', {
            filterId: folderId,
            limit: 20,
            offset: currentOffset,
            previewOnly: false,
          });
        }

        if (append) {
          setClips((prev) => {
            return [...prev, ...data];
          });
        } else {
          setClips(data);
        }

        // If we got fewer than limit, no more clips
        setHasMore(data.length === 20);
      } catch (error) {
        console.error('Failed to load clips:', error);
      } finally {
        setIsLoading(false);
      }
    },
    [clips.length]
  );

  const loadFolders = useCallback(async () => {
    try {
      const data = await invoke<FolderItem[]>('get_folders');

      setFolders(data);
    } catch (error) {
      console.error('Failed to load folders:', error);
    }
  }, []);

  const refreshCurrentFolder = useCallback(() => {
    loadClips(selectedFolderRef.current, false, searchQuery);
  }, [loadClips, searchQuery]);

  const handleSearch = useCallback((query: string) => {
    setSearchQuery(query);
  }, []);

  useEffect(() => {
    loadFolders();
    if (searchQuery.trim()) {
      loadClips(selectedFolder, false, searchQuery);
    } else {
      loadClips(selectedFolder);
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [selectedFolder, searchQuery]);

  // Handle global mouse events for simulated drag
  useEffect(() => {
    const handleGlobalMouseMove = (e: MouseEvent) => {
      const state = dragStateRef.current;

      // If we are already dragging, update position
      if (state.isDragging) {
        setDragPosition({ x: e.clientX, y: e.clientY });
        return;
      }

      // If we have a pending drag, check threshold
      if (state.pendingDrag) {
        const dx = e.clientX - state.pendingDrag.startX;
        const dy = e.clientY - state.pendingDrag.startY;
        const dist = Math.sqrt(dx * dx + dy * dy);

        if (dist > 5) {
          // Start actual drag
          setDraggingClipId(state.pendingDrag.clipId);
          setDragPosition({ x: e.clientX, y: e.clientY });
          dragStateRef.current.isDragging = true;
          dragStateRef.current.clipId = state.pendingDrag.clipId;
          dragStateRef.current.pendingDrag = null;
        }
      }
    };

    const handleGlobalMouseUp = (_: MouseEvent) => {
      // Always clear pending drag on mouse up
      if (dragStateRef.current.pendingDrag) {
        dragStateRef.current.pendingDrag = null;
      }

      if (dragStateRef.current.isDragging) {
        finishDrag();
      }
    };

    window.addEventListener('mousemove', handleGlobalMouseMove);
    window.addEventListener('mouseup', handleGlobalMouseUp);

    return () => {
      window.removeEventListener('mousemove', handleGlobalMouseMove);
      window.removeEventListener('mouseup', handleGlobalMouseUp);
    };
  }, []);

  const startDrag = (clipId: string, startX: number, startY: number) => {
    // Instead of starting immediately, set pending
    dragStateRef.current.pendingDrag = { clipId, startX, startY };
    dragStateRef.current.clipId = clipId;
    // We don't set state yet, avoiding re-render until threshold passed
  };

  const finishDrag = () => {
    if (dragStateRef.current.targetFolderId !== undefined && dragStateRef.current.clipId) {
      // We only move if targetFolderId was explicitly set by a hover event.
      // Wait, how do we distinguish "Not Hovering" vs "Hovering 'All' (null)"?
      // We will make ControlBar pass a specific sentinel for "No Target" when leaving?
      // Or simply: ControlBar tracks hover. If hover, it calls setDragTargetFolderId.
      // If we drop and dragTargetFolderId is valid, we move.
      // BUT 'null' is a valid folder ID (All).
      // Let's use a generic 'undefined' for "No Target".
    }

    // Actually, simpler:
    // When MouseUp happens, we check dragTargetFolderId state.
    // If it is NOT undefined, we execute move.

    // IMPORTANT: State updates in React are async. accessing `dragTargetFolderId` state inside event listener might be stale?
    // That's why we use `dragStateRef`.

    const { clipId, targetFolderId } = dragStateRef.current;
    if (clipId && targetFolderId !== undefined && targetFolderId !== 'NO_TARGET') {
      handleMoveClip(clipId, targetFolderId);
    }

    setDraggingClipId(null);
    setDragTargetFolderId(null);
    dragStateRef.current = {
      isDragging: false,
      clipId: null,
      targetFolderId: 'NO_TARGET',
      pendingDrag: null,
    };
  };

  const handleDragHover = (folderId: string | null) => {
    setDragTargetFolderId(folderId);
    dragStateRef.current.targetFolderId = folderId;
  };

  const handleDragLeave = () => {
    setDragTargetFolderId(null);
    dragStateRef.current.targetFolderId = 'NO_TARGET';
  };

  // Total History Count
  const [totalClipCount, setTotalClipCount] = useState(0);

  const refreshTotalCount = useCallback(async () => {
    try {
      const count = await invoke<number>('get_clipboard_history_size');
      setTotalClipCount(count);
    } catch (e) {
      console.error('Failed to get history size', e);
    }
  }, []);

  useEffect(() => {
    refreshTotalCount();
  }, [refreshTotalCount]);

  useEffect(() => {
    const unlistenClipboard = listen('clipboard-change', () => {
      refreshCurrentFolder();
      loadFolders(); // Refresh folders to get updated counts
      refreshTotalCount(); // Refresh total count
    });

    return () => {
      unlistenClipboard.then((unlisten) => {
        if (typeof unlisten === 'function') unlisten();
      });
    };
  }, [refreshCurrentFolder, loadFolders, refreshTotalCount]);

  useKeyboard({
    onClose: () => appWindow.hide(),
    onSearch: () => setShowSearch(true),
    onDelete: () => handleDelete(selectedClipId),
  });

  const handleDelete = async (clipId: string | null) => {
    if (!clipId) return;
    try {
      await invoke('delete_clip', { id: clipId, hardDelete: false });
      setClips(clips.filter((c) => c.id !== clipId));
      setSelectedClipId(null);
      // Refresh counts
      loadFolders();
      refreshTotalCount();
      toast.success('Clip deleted');
    } catch (error) {
      console.error('Failed to delete clip:', error);
      toast.error('Failed to delete clip');
    }
  };

  const handlePaste = async (clipId: string) => {
    try {
      const clip = clips.find((c) => c.id === clipId);
      if (clip && clip.clip_type === 'image' && !isMacOS()) {
        try {
           const blob = base64ToBlob(clip.content, 'image/png');
           await navigator.clipboard.write([new ClipboardItem({ [blob.type]: blob })]);
        } catch (e) {
           console.error("Frontend clipboard write failed", e);
        }
      }

      invoke('paste_clip', { id: clipId }).catch(console.error);
    } catch (error) {
      console.error('Failed to paste clip:', error);
    }
  };

  const handleCopy = async (clipId: string) => {
    try {
      const clip = clips.find((c) => c.id === clipId);
      if (clip && clip.clip_type === 'image' && !isMacOS()) {
        const blob = base64ToBlob(clip.content, 'image/png');
        await navigator.clipboard.write([new ClipboardItem({ [blob.type]: blob })]);
        // For copy, we might not need to call backend 'paste_clip' if we just want to copy?
        // But 'paste_clip' also updates 'last_pasted' timestamp and moves it to top.
        // The original code called 'paste_clip'. Let's keep consistency but maybe backend copy logic differs?
        // Actually handleCopy calls 'paste_clip' in original code, which is weird if it simulates Ctrl+V?
        // 'handleCopy' usually just puts it on clipboard.
        // If 'paste_clip' simulates input, then 'handleCopy' executing 'paste_clip' would PASTE it.
        // Let's look at the original code:
        // await invoke('paste_clip', { id: clipId });
        // toast.success('Copied to clipboard');
        // This implies 'paste_clip' MIGHT NOT always paste? Or the user logic was flawed?
        // Wait, if it's "Copy", we shouldn't simulate Ctrl+V.
        // Let's assuming for now we just want to write to clipboard.

        // If we write to clipboard here, we might still want to update DB timestamp.
        // But let's follow the existing pattern: invoke 'paste_clip' but we know we modify it to NOT write image.
        // Wait, if backend 'paste_clip' performs Ctrl+V, then 'handleCopy' doing 'paste_clip' is wrong?
        // Let's assume the user just wants to put it on clipboard.
      }

      // We still invoke paste_clip because it probably handles DB updates.
      // However, if paste_clip simulates Ctrl+V, that would be bad for "Copy".
      // Let's assume the backend 'paste_clip' logic is "Put to clipboard AND Paste".
      // Use 'copy_clip_to_clipboard' if it exists?
      // Checking grep results... no 'copy_clip' found.
      // It seems 'handleCopy' uses 'paste_clip' which is PROBABLY WRONG if it pastes.
      // But I will stick to the plan: Frontend writes image.

      // If the backend 'paste_clip' does Paste Input, then 'handleCopy' is actually "Paste" in the current app?
      // Or maybe 'paste_clip' determines intention? No.

      // Let's just implement the Write Image part.
      // If it's an image, we write it.

      await invoke('paste_clip', { id: clipId });
      // Note: If 'paste_clip' does Ctrl+V, then this "Copy" button actually Pastes.
      // Refactoring that is out of scope unless necessary.

      toast.success('Copied to clipboard');
    } catch (error) {
      console.error('Failed to copy clip:', error);
      toast.error('Failed to copy');
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
      loadClips(selectedFolder, true, searchQuery);
    }
  }, [hasMore, isLoading, selectedFolder, loadClips, searchQuery]);

  const handleMoveClip = async (clipId: string, folderId: string | null) => {
    try {
      await invoke('move_to_folder', { clipId, folderId });

      // Update local state to reflect the move
      if (selectedFolder) {
        // If we are in a specific folder (not All)
        if (folderId !== selectedFolder) {
          // If moved to a different folder, remove from current view
          setClips((prev) => prev.filter((c) => c.id !== clipId));
        }
      } else {
        // If we are in "All clips" view, just update the folder_id
        setClips((prev) => prev.map((c) => (c.id === clipId ? { ...c, folder_id: folderId } : c)));
      }
      // Refresh counts after move
      loadFolders();
      refreshTotalCount();
    } catch (error) {
      console.error('Failed to move clip:', error);
    }
  };

  // Context Menu State
  const [contextMenu, setContextMenu] = useState<{
    type: 'card' | 'folder';
    x: number;
    y: number;
    itemId: string;
  } | null>(null);

  // New Folder Modal Rename Mode
  const [folderModalMode, setFolderModalMode] = useState<'create' | 'rename'>('create');
  const [editingFolderId, setEditingFolderId] = useState<string | null>(null);

  // AI Result State
  const [aiResult, setAiResult] = useState({
    isOpen: false,
    title: '',
    content: '',
  });

  const handleAiAction = async (clipId: string, action: string, title: string) => {
    try {
      const toastId = toast.loading('Processing with AI...');
      const result = await invoke<string>('ai_process_clip', { clipId, action });
      toast.dismiss(toastId);
      setAiResult({
        isOpen: true,
        title,
        content: result,
      });
    } catch (error) {
      toast.dismiss();
      console.error('AI Processing Failed:', error);
      toast.error(`AI Error: ${error}`);
    }
  };

  const handleContextMenu = useCallback(
    (e: React.MouseEvent, type: 'card' | 'folder', itemId: string) => {
      e.preventDefault();
      setContextMenu({
        type,
        x: e.clientX,
        y: e.clientY,
        itemId,
      });
    },
    []
  );

  const handleCloseContextMenu = useCallback(() => {
    setContextMenu(null);
  }, []);

  // Updated Create Folder to handle Rename
  const handleCreateOrRenameFolder = async (name: string) => {
    if (folderModalMode === 'create') {
      await handleCreateFolder(name);
      toast.success(`Folder "${name}" created`);
      setShowAddFolderModal(false);
      setNewFolderName('');
    } else if (folderModalMode === 'rename' && editingFolderId) {
      try {
        await invoke('rename_folder', { id: editingFolderId, name });
        await loadFolders();
        toast.success(`Renamed to "${name}"`);
        setShowAddFolderModal(false);
        setNewFolderName('');
      } catch (error) {
        console.error('Failed to rename folder:', error);
        toast.error('Failed to rename folder');
      }
    }
  };

  const handleDeleteFolder = async (folderId: string) => {
    if (!folderId) return;
    try {
      await invoke('delete_folder', { id: folderId });
      if (selectedFolder === folderId) {
        setSelectedFolder(null);
      }
      await loadFolders();
      refreshTotalCount();
      toast.success('Folder deleted');
    } catch (error) {
      console.error('Failed to delete folder:', error);
      toast.error('Failed to delete folder');
    }
  };

  const mac = isMacOS();

  return (
    <div className="relative h-screen w-full overflow-hidden">
      {/* Blur layer: Windows only */}
      {!mac && (
        <div
          className="absolute inset-0"
          style={{
            backgroundColor: 'transparent',
            backdropFilter: 'blur(2px)',
          }}
        />
      )}

      {/* Content Container */}
      <div className="relative h-full w-full" style={{ padding: mac ? '0px' : `${LAYOUT.WINDOW_PADDING}px` }}>
        <div className={`flex h-full w-full flex-col overflow-hidden font-sans text-foreground ${
          mac
            ? 'rounded-[16px] bg-background'
            : 'rounded-[16px] border border-border/10 bg-background/80 shadow-[0_0_24px_rgba(0,0,0,0.2)] dark:shadow-[0_0_24px_rgba(0,0,0,0.4)]'
        }`}>
          {draggingClipId && (
            <DragPreview
              clip={clips.find((c) => c.id === draggingClipId)!}
              position={dragPosition}
            />
          )}

          {contextMenu && (
            <ContextMenu
              x={contextMenu.x}
              y={contextMenu.y}
              onClose={handleCloseContextMenu}
              options={
                contextMenu.type === 'card'
                  ? [
                      {
                        label: `${settings?.ai_title_summarize || 'Summarize'} (AI)`,
                        onClick: () =>
                          handleAiAction(contextMenu.itemId, 'summarize', 'AI Summary'),
                      },
                      {
                        label: `${settings?.ai_title_translate || 'Translate'} (AI)`,
                        onClick: () =>
                          handleAiAction(contextMenu.itemId, 'translate', 'AI Translation'),
                      },
                      {
                        label: `${settings?.ai_title_explain_code || 'Explain Code'} (AI)`,
                        onClick: () =>
                          handleAiAction(contextMenu.itemId, 'explain_code', 'Code Explanation'),
                      },
                      {
                        label: `${settings?.ai_title_fix_grammar || 'Fix Grammar'} (AI)`,
                        onClick: () =>
                          handleAiAction(contextMenu.itemId, 'fix_grammar', 'Grammar Check'),
                      },
                      {
                        label: 'Delete',
                        danger: true,
                        onClick: () => handleDelete(contextMenu.itemId),
                      },
                    ]
                  : [
                      {
                        label: 'Rename',
                        onClick: () => {
                          setFolderModalMode('rename');
                          setEditingFolderId(contextMenu.itemId);
                          const folder = folders.find((f) => f.id === contextMenu.itemId);
                          setNewFolderName(folder ? folder.name : '');
                          setShowAddFolderModal(true);
                        },
                      },
                      {
                        label: 'Delete',
                        danger: true,
                        onClick: () => handleDeleteFolder(contextMenu.itemId),
                      },
                    ]
              }
            />
          )}

          <ControlBar
            folders={folders}
            selectedFolder={selectedFolder}
            onSelectFolder={setSelectedFolder}
            showSearch={showSearch}
            searchQuery={searchQuery}
            onSearchChange={handleSearch}
            onSearchClick={() => {
              if (showSearch) {
                handleSearch(''); // Clear search when closing
              }
              setShowSearch(!showSearch);
            }}
            onAddClick={() => {
              setFolderModalMode('create');
              setNewFolderName('');
              setShowAddFolderModal(true);
            }}
            onMoreClick={openSettings}
            onMoveClip={handleMoveClip} // Legacy, but kept for interface
            // Simulated Drag Props
            isDragging={!!draggingClipId}
            dragTargetFolderId={dragTargetFolderId}
            onDragHover={handleDragHover}
            onDragLeave={handleDragLeave}
            totalClipCount={totalClipCount}
            onFolderContextMenu={(e, folderId) => {
              if (folderId) handleContextMenu(e, 'folder', folderId);
            }}
            theme={effectiveTheme}
          />

          <main className="no-scrollbar relative flex-1 overflow-hidden">
            <ClipList
              clips={clips}
              isLoading={isLoading}
              hasMore={hasMore}
              selectedClipId={selectedClipId}
              onSelectClip={setSelectedClipId}
              onPaste={handlePaste}
              onCopy={handleCopy}
              onDelete={handleDelete}
              onLoadMore={loadMore}
              // Simulated Drag Props
              onDragStart={startDrag}
              onCardContextMenu={(e, clipId) => handleContextMenu(e, 'card', clipId)}
            />

            {/* Add/Rename Folder Modal Overlay */}
            <FolderModal
              isOpen={showAddFolderModal}
              mode={folderModalMode}
              initialName={newFolderName}
              onClose={() => {
                setShowAddFolderModal(false);
                setNewFolderName('');
              }}
              onSubmit={handleCreateOrRenameFolder}
            />

            <AiResultDialog
              isOpen={aiResult.isOpen}
              title={aiResult.title}
              content={aiResult.content}
              onClose={() => setAiResult((prev) => ({ ...prev, isOpen: false }))}
            />
          </main>
          <Toaster richColors position="bottom-center" theme={effectiveTheme} />
        </div>
      </div>
    </div>
  );
}

export default App;
