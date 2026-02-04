<h1 align="center">
    <picture>
      <img src="https://raw.githubusercontent.com/XueshiQiao/PastePaw/refs/heads/main/src-tauri/icons/64x64.png" alt="Claude-Mem" width="32">
    </picture>
    PastePaw - Clipboard History Manager
</h1>


<p align="center">
  <a href="README_CN.md">中文</a> •
  <a href="README.md">English</a>
</p>

<p align="center">
  <a href="https://github.com/XueshiQiao/PastePaw/actions/workflows/release.yml?branch=main"><img src="https://img.shields.io/github/actions/workflow/status/XueshiQiao/PastePaw/release.yml?branch=main&style=flat" alt="CI status"></a>
  <a href="https://github.com/XueshiQiao/PastePaw/releases"><img src="https://img.shields.io/github/v/release/XueshiQiao/PastePaw?include_prereleases&style=flat" alt="GitHub release"></a>
  <a href="LICENSE"><img src="https://img.shields.io/badge/License-GPL3.0-blue.svg?style=flat" alt="MIT License"></a>
</p>

A beautiful clipboard history manager for Windows, built with Rust + Tauri + React + TypeScript.

## Features

- 🔒 **Private** - IMPORTANT! All data stored locally
- 🎨 **Beautiful UI** - Modern dark/light theme with immediate switching
- ⚡ **Fast & Lightweight** - Built with Rust for performance
- 📋 **Clipboard History** - Automatically saves everything you copy
- 🖥️ **Support multiple Displays** - Show on the active display
- 🔍 **Search** - Quickly find previously copied content
- 📁 **Folders** - Organize clips into custom folders
- 🚫 **Application Exceptions** - Ignore content from specific sensitive apps (e.g., Password Managers)
- ⌨️ **Customizable Hotkey** - Set your preferred shortcut to open the history
- 🔄 **Infinite Scroll** - Seamlessly browse through unlimited history
- 🛡️ **Smart Filtering** - Intelligent debounce logic to ignore "Ghost Copies" from other clipboard tools
- 🤖 **AI Powered** - Built-in AI to summarize, translate, explain code, and fix grammar
- ⚙️ **Customizable AI** - Fully customize AI action names and system prompts to suit your workflow

## Keyboard Shortcuts

### Global
- **Toggle Window**: `Ctrl+Shift+V` (Default, Customizable in Settings)

### In-App

- `Ctrl + F` - Focus search
- `Escape` - Close window / Clear search
- `Enter` - Paste selected item
- `Delete` - Delete selected item
- `P` - Pin/Unpin selected item
- `Arrow Up/Down` - Navigate items

## Application Exceptions (Ignored Apps)

PastePaw allows you to exclude specific applications from being recorded in the clipboard history. This is useful for privacy-sensitive applications like password managers or banking apps.

(You need to provide the API Key for the AI provider)

**Logic & Behavior:**
- **How to manage:** Go to Settings -> Ignored Applications. You can browse for an executable (`.exe`) or strictly type its name.
- **Privacy Protection:** When content is copied, PastePaw checks the source application against your ignore list.
- **Robust Matching:** The system checks against **both**:
    1.  **Executable Name** (e.g., `notepad.exe`) - Matches any instance of this app regardless of location.
    2.  **Full File Path** (e.g., `C:\Windows\System32\notepad.exe`) - Matches only the specific installed instance.
- **Case Insensitive:** Matching is case-insensitive to ensure reliable detection on Windows.

## AI Features

PastePaw integrates powerful AI capabilities to help you process your clipboard content more efficiently.

- **Actions:** Right-click any clip to access AI actions:
    - **Summarize:** Get a concise summary of long texts.
    - **Translate:** Translate content to your preferred language.
    - **Explain Code:** Understand complex code snippets instantly.
    - **Fix Grammar:** Polishing your writing with professional grammar checks.
- **Full Customization:**
    - **Custom Names:** Rename AI actions in Settings (e.g., change "Translate" to "To Spanish").
    - **Custom Prompts:** Override default system prompts to tailor the AI's behavior and output style.
    - **Provider Support:** Support for OpenAI, DeepSeek, and other OpenAI-compatible APIs.

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
PastePaw/
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

### Window Behavior & Multi-Monitor Support

The application is designed to appear on the **active monitor** (the one containing the mouse cursor) whenever the global hotkey is pressed.

- **Detection Logic:**
    - Located in `src-tauri/src/lib.rs` (`animate_window_show`).
    - Uses the Windows API `GetCursorPos` (via the `windows` crate) to determine the global mouse coordinates.
    - Iterates through `window.available_monitors()` to find the monitor whose bounds contain the cursor point.
    - Fallback: If the cursor position cannot be determined, it defaults to `window.current_monitor()`.

- **Positioning:**
    - The window is positioned at the bottom of the detected active monitor's work area (excluding taskbar).
    - An animation slides the window up from the bottom edge.

### Adjusting the Layout

The application uses a centralized layout system to ensure the native window and the virtualized list remain synchronized.

-   **Backend Constants:** `src-tauri/src/constants.rs` (Controls the OS window size).
*   **Frontend Constants:** `frontend/src/constants.ts` (Controls UI rendering and math).

#### How to change Card Height
The card height is dynamic and fills the available window space. To change it:
1.  Update `WINDOW_HEIGHT` in **both** `constants.rs` and `constants.ts` to the same value.
2.  Restart the application (required for Rust changes).

#### How to change Vertical Spacing (Safe Zones)
To add more or less space at the top/bottom of the cards (e.g., to prevent clipping during hover):
1.  Modify `CARD_VERTICAL_PADDING` in `frontend/src/constants.ts`.
2.  Increasing this value makes cards **shorter**; decreasing it makes them **taller**.


## License

GPL-3.0
