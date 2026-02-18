# Implementation Plan: UI Polish & Fixes

## Phase 1: Menu Bar Icon
- [x] Task: Create/Source "simple cat paw" icon asset for tray (png/ico).
- [x] Task: Update `src-tauri/icons/tray.png` (and `tray.ico` for Windows if needed).
- [x] Task: Configure Tauri to use the new icon as the system tray icon.
  *Note: Tauri was already configured to use `tray.png`, just needed the file updated.*
- [x] Task: Verify appearance in Light and Dark menu bars.
  *Note: Generated icon is solid black on transparent background, suitable for template mode.*

## Phase 2: Custom Selector Component
- [x] Task: Create `frontend/src/components/ui/Select.tsx` (Reusable component).
  -   *Design:* Trigger button with chevron, dropdown list with hover states.
- [x] Task: Refactor `SettingsPanel.tsx` to import and use the new `Select` component.
  -   *Target:* Language selector.
  -   *Target:* Theme selector.
  -   *Target:* AI Model selector.
  -   *Target:* Window Effect selector.
- [x] Task: Verify functionality (selection updates state correctly).
- [x] Task: Verify styling on older macOS versions (simulated or via rigorous CSS checks).

## Phase 3: Verification
- [x] Task: Conductor - User Manual Verification (Visual check of Tray and Settings).
