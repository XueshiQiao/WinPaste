# WinPaste Project Analysis

## 1. Project Overview
**WinPaste** is a native Windows clipboard history manager application built with a modern stack. It runs as a desktop application with a background service to monitor clipboard changes.

- **Type:** Desktop Application (Tauri)
- **Primary Goal:** Store and manage clipboard history (Text, with scaffolding for other types).
- **Key Features:**
  - Clipboard monitoring and history storage.
  - Search functionality.
  - Organization (Folders: All, Pinned, Recent).
  - Keyboard shortcuts (`Ctrl+Shift+V` default).
  - Modern, dark-themed UI with virtualized list for performance.

## 2. Technical Architecture
The project uses a hybrid architecture typical of Tauri apps:

### **Backend (Rust)**
- **Framework:** Tauri v2.0
- **Database:** SQLite (via `sqlx`).
- **Clipboard Monitoring:** Custom thread using `arboard` crate (Polling mechanism).
- **Concurrency:** Uses `tokio` for async database operations and `std::thread` for blocking clipboard polling.
- **IPC:** Exposes commands (e.g., `get_clips`, `paste_clip`) to frontend.

### **Frontend (React)**
- **Framework:** React 18 + TypeScript + Vite.
- **Styling:** Tailwind CSS.
- **Performance:** `react-window` for virtualized rendering of large lists.
- **Window Management:** Custom window animations for slide-up/slide-down effects.

## 3. Code Structure Analysis
- **`src-tauri/src/main.rs` & `lib.rs`**: Entry point. Sets up the system tray, global shortcuts, and starts the clipboard monitor thread.
- **`src-tauri/src/clipboard.rs`**:
  - Spawns a dedicated thread that wakes up every **1 second**.
  - Checks for clipboard changes by comparing hashes.
  - Inserts new clips into SQLite.
- **`src-tauri/src/database.rs`**: Handles all SQLite interactions. Good schema design with separate tables for `clips`, `folders`, and `settings`.
- **`frontend/src/App.tsx`**: Main UI controller. Fetches clips and handles global state.
- **`frontend/src/components/ClipList.tsx`**: Efficiently renders the list of clips using virtualization.

## 4. Identified Issues & Bugs

### **Critical Limitations**
1.  **Limited Clipboard Support (Text Only)**:
    - Although the database schema supports different `clip_type`s, the monitoring loop in `clipboard.rs` **hardcodes** handling only text.
    - **Issue**: Images, files, and HTML content are ignored or not captured.

2.  **Slow Polling Interval**:
    - **File**: `src-tauri/src/clipboard.rs`
    - **Issue**: The thread sleeps for `1000ms` (1 second).
    - **Impact**: Rapid copy operations (e.g., copying two things quickly) will likely result in the second one overwriting the first or being missed entirely before the poller wakes up. 1 second is quite noticeable lag for a clipboard tool.

### **Performance Concerns**
3.  **Frontend Data Loading**:
    - **File**: `frontend/src/App.tsx`
    - **Issue**: `loadClips` calls `get_clips` with `limit: 10000`.
    - **Impact**: While `react-window` handles *rendering* efficiently, fetching 10,000 items into JS memory on every folder switch or significant update is not scalable and will eventually cause lag as the database grows. Pagination (infinite scroll) should be implemented at the data fetching level.

4.  **Database Storage**:
    - **Issue**: `content` is stored as a `BLOB`. For text, this is fine, but for potentially efficient searching, ensuring it's indexed correctly is key. Currently, `content_hash` is indexed, which is good for duplicate detection.

## 5. Suggested Improvements

### **High Priority**
1.  **Implement Image Support**:
    - Update `clipboard.rs` to check for images using `arboard` (which supports it).
    - Store image data (efficiently, perhaps resizing for preview) in the database.
    - Update Frontend to render image previews.

2.  **Optimize Polling**:
    - Reduce sleep time to `200ms` or implement an OS-level event listener (though `arboard` is polling-only, other crates like `clipboard-master` might offer event-driven updates).

3.  **True Infinite Scroll**:
    - Modify Frontend `ClipList` to request more data as the user scrolls down, rather than loading 10k items at start.

### **Code Quality**
4.  **Sync Constants**:
    - `WINDOW_HEIGHT` is defined in both Rust and TypeScript manually. This should be passed from Backend to Frontend via the `get_layout_config` command (which seems to exist) to ensure they never drift.

5.  **Refactor Clipboard Monitor**:
    - The `clipboard.rs` creates a new `tokio::runtime` inside the thread. It would be cleaner to reuse a handle to the global runtime if possible, or keep the runtime alive rather than recreating it (though it seems it creates it once per thread start, which is fine).
