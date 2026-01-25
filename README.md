# WinPaste - Clipboard History Manager

A beautiful clipboard history manager for Windows, built with Rust + Tauri + React + TypeScript.

## Features

- 📋 **Clipboard History** - Automatically saves everything you copy
- 🔍 **Search** - Quickly find previously copied content
- 📌 **Pin Items** - Keep important clips permanently
- 📁 **Folders** - Organize clips into custom folders
- 🎨 **Beautiful UI** - Modern dark theme with smooth animations
- ⚡ **Fast & Lightweight** - Built with Rust for performance
- 🔒 **Private** - All data stored locally

## Tech Stack

- **Backend**: Rust + Tauri 2.x
- **Frontend**: React 18 + TypeScript
- **Database**: SQLite
- **Styling**: Tailwind CSS
- **Package Manager**: pnpm

## Getting Started

### Prerequisites

- Node.js 18+
- Rust 1.70+
- pnpm

### Installation

```bash
# Install dependencies
pnpm install

# Install Tauri CLI
cargo install tauri-cli

# Run development build
pnpm tauri dev
```

### Building

```bash
# Build for production
pnpm tauri build
```

## Project Structure

```
WinPaste/
├── src-tauri/           # Rust backend
│   ├── src/
│   │   ├── main.rs      # App entry point
│   │   ├── lib.rs       # Core logic
│   │   ├── clipboard.rs # Clipboard monitoring
│   │   ├── database.rs  # SQLite operations
│   │   ├── commands.rs  # Tauri IPC commands
│   │   └── models.rs    # Data models
│   └── Cargo.toml
├── frontend/            # React frontend
│   ├── src/
│   │   ├── components/  # UI components
│   │   ├── hooks/       # React hooks
│   │   ├── types/       # TypeScript types
│   │   └── App.tsx
│   └── package.json
└── README.md
```

## Development Notes

### Tauri Command Argument Mapping

Tauri v2 enforces a strict case mapping between JavaScript/TypeScript and Rust:

- **JavaScript/Frontend:** Use `camelCase` for argument names in `invoke` calls (e.g., `filterId`).
- **Rust/Backend:** Use `snake_case` for function arguments in `#[tauri::command]` (e.g., `filter_id`).

**Example:**
*   **Frontend:** `invoke('get_clips', { filterId: 'pinned' })`
*   **Backend:** `pub fn get_clips(filter_id: Option<String>)`

Failure to follow this convention (e.g., passing `snake_case` from the frontend) will result in arguments being passed as `null` or `None` to the backend.

## Keyboard Shortcuts

- `Ctrl + F` - Focus search
- `Escape` - Close window / Clear search
- `Enter` - Paste selected item
- `Delete` - Delete selected item
- `P` - Pin/Unpin selected item
- `Arrow Up/Down` - Navigate items

## License

MIT
