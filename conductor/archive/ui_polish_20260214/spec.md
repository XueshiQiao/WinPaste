# Specification: UI Polish & Fixes

## 1. Menu Bar Icon Update
**Problem:** The current menu bar (tray) icon appears as a "pure white" square (or generic icon) instead of the application's branding.
**Requirement:**
-   Replace the tray icon with a "simple cat paw" design.
-   Ensure it supports macOS menu bar templating (automatically adjusting to light/dark mode if strictly monochrome alpha) or provide proper colored variants if preferred.
-   **Target File:** `src-tauri/icons/tray.png` (and potentially code config if template mode is needed).

## 2. Custom Selector Component
**Problem:** The native `<select>` element renders inconsistently across different macOS versions (especially older ones), breaking the unified UI aesthetic of the Settings page.
**Requirement:**
-   Create a custom `Select` component (React).
-   **Style:** consistent with the existing Tailwind CSS design system (Dark/Light mode support).
-   **Features:**
    -   Custom trigger button.
    -   Custom dropdown menu (absolute positioned).
    -   Support for value/label pairs.
    -   Keyboard navigation (optional but recommended for accessibility).
-   **Implementation:** Replace all instances of HTML `<select>` in `SettingsPanel.tsx` with this new component.
