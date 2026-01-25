# WinPaste Development Guide

A beautiful clipboard history manager for Windows, built with Rust + Tauri + React + TypeScript.

## Project Structure

```
WinPaste/
├── src-tauri/           # Rust backend
│   ├── src/
│   │   ├── main.rs      # App entry point
│   │   ├── lib.rs       # Core logic and Tauri setup
│   │   ├── clipboard.rs # Clipboard monitoring
│   │   ├── database.rs  # SQLite operations (sqlx)
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
└── package.json
```

## Build Commands

### Frontend (React + TypeScript)

```bash
# Install dependencies
pnpm install

# Development server
pnpm dev

# Build for production
pnpm build

# Preview production build
pnpm preview
```

### Tauri Application

```bash
# Install Tauri CLI (one-time)
cargo install tauri-cli

# Development build with hot reload
pnpm tauri dev

# Production build (creates NSIS installer)
pnpm tauri build
```

### Rust Backend

```bash
# Build debug
cargo build

# Build release
cargo build --release

# Run tests
cargo test

# Run a single test
cargo test <test_name>

# Check for compilation errors
cargo check

# Format code
cargo fmt

# Lint
cargo clippy
```

## Code Style Guidelines

### General Principles

- Write clear, self-documenting code
- Keep functions small and focused on single responsibilities
- Use meaningful names for all identifiers
- Handle errors explicitly rather than silently

### Rust Guidelines

**Naming Conventions:**
- Functions: `snake_case`
- Variables: `snake_case`
- Constants: `UPPER_SNAKE_CASE`
- Types/Enums: `PascalCase`
- Module names: `snake_case`

**Error Handling:**
- Use `thiserror` for defining error types
- Return `Result<T, String>` for Tauri commands
- Use `?` operator for error propagation
- Propagate errors with `.map_err(|e| e.to_string())` for IPC

**Async Operations:**
- Use `tokio` runtime (create with `tokio::runtime::Runtime::new()`)
- Block on async operations in sync contexts with `rt.block_on(async { ... })`
- Handle runtime creation errors explicitly

**Imports:**
- Group imports by crate: std → external → local
- Use absolute paths for intra-crate imports (`crate::`, `super::`)

**Code Patterns:**
- Use `Arc` for shared ownership, `OnceLock` for lazy global initialization
- Use `String::from_utf8_lossy` for clipboard content conversion
- Prefer pattern matching over if-let chains where appropriate

### TypeScript/React Guidelines

**TypeScript Configuration:**
- Strict mode enabled (`strict: true`)
- `noUnusedLocals: true`, `noUnusedParameters: true`
- `noFallthroughCasesInSwitch: true`

**Naming Conventions:**
- Variables/functions: `camelCase`
- Interfaces/Types: `PascalCase`
- Constants: `UPPER_SNAKE_CASE`
- Components: `PascalCase` (named exports preferred)

**React Patterns:**
- Use `useCallback` for callback functions passed as props
- Use `useMemo` for expensive computations
- Define prop interfaces for all components
- Prefer function components with TypeScript interfaces

**Imports:**
- React imports first, then external libraries, then local imports
- Use `@/*` alias for frontend src imports (configured in vite.config.ts)
- Use named imports for libraries: `import { useState } from 'react'`

**Error Handling:**
- Wrap async operations in try/catch blocks
- Log errors with `console.error`
- Always handle loading states

**Styling:**
- Use Tailwind CSS utility classes
- Use `clsx` and `tailwind-merge` for conditional classes
- Follow existing color scheme (dark theme by default)

**File Organization:**
- Components: `frontend/src/components/`
- Hooks: `frontend/src/hooks/`
- Types: `frontend/src/types/index.ts`
- Utilities: Group with related components or in dedicated utils file

## Keyboard Shortcuts

- `Ctrl + F` - Focus search
- `Escape` - Close window / Clear search
- `Enter` - Paste selected item
- `Delete` - Delete selected item
- `P` - Pin/Unpin selected item
- `Arrow Up/Down` - Navigate items

## Tech Stack

- **Backend**: Rust + Tauri 2.x + SQLite (sqlx)
- **Frontend**: React 18 + TypeScript + Vite
- **Database**: SQLite with sqlx ORM
- **Styling**: Tailwind CSS
- **Icons**: Lucide React
- **Package Manager**: pnpm

## Notes

- All clipboard content is stored locally; no data is sent to external servers
- The app uses a custom protocol for development (tauri://localhost:1420)
- Images are stored as binary data in SQLite
- Content hashing (SHA-256) prevents duplicate entries
