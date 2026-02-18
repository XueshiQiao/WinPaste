# Technology Stack - PastePaw

## Core Frameworks
- **Backend:** Rust with [Tauri 2.x](https://tauri.app/) for a lightweight, secure, and high-performance system integration.
- **Frontend:** [React 18](https://react.dev/) with [TypeScript](https://www.typescriptlang.org/) for a robust and maintainable user interface.

## Data & State
- **Database:** [SQLite](https://www.sqlite.org/) managed via [sqlx](https://github.com/launchbadge/sqlx) for local-first, reliable data persistence.
- **Package Manager:** [pnpm](https://pnpm.io/) for efficient dependency management.

## UI & Styling
- **Styling:** [Tailwind CSS](https://tailwindcss.com/) for rapid, utility-first UI development.
- **Icons:** [Lucide React](https://lucide.dev/) for a consistent and modern iconography set.
- **Localization:** [i18next](https://www.i18next.com/) and `react-i18next` for comprehensive multi-language support (English/Chinese).

## Infrastructure & Runtime
- **Runtime:** [Tokio](https://tokio.rs/) as the asynchronous runtime for high-concurrency clipboard monitoring and database operations.
- **Build Tools:** [Vite](https://vitejs.dev/) for fast frontend development and bundling.
