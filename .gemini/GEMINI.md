# Project Introduction

PastePaw is a beautiful, high-performance clipboard history manager for Windows. It is built using a modern tech stack: **Rust (Tauri 2.x)** for the backend and **React 18 + TypeScript** for the frontend.

### Key Features:
- **Local & Private:** All data is stored locally in a SQLite database.
- **Multi-Monitor Support:** Automatically appears on the active display where your cursor is.
- **Smart Filtering:** Debounce logic to ignore "Ghost Copies" and support for excluding sensitive applications (e.g., password managers).
- **Customizable:** Hotkeys, themes, and folder organization.
- **Performance:** Lightweight and fast, leveraging Rust's efficiency.

# General Instructions

- **Dependencies:** Always ask before introducing new dependencies. Provide multiple-choice options with pros and cons for selection.
- **Commits:** Do not commit automatically. Let the user review the code first. Commit only after explicit permission.
- **Commit Style:** Follow best practices for commit messages.
- **Windows Compatibility:** Avoid using `&&` in shell commands; use separate commands or PowerShell constructs as the environment is Windows (PowerShell).
- **Indentation:** Use two spaces for indentation.
- **Encapsulation:** For Rust/TypeScript, encapsulate logic into classes or structured modules where possible.
- **Tauri Mapping:** Remember that Tauri v2 maps `camelCase` (JS) to `snake_case` (Rust) for command arguments.
