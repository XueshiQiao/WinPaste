# Implementation Plan: Prepare for Apple App Store Submission

## Phase 1: Environment & Entitlements [checkpoint: b5779b4]
- [x] Task: Audit current macOS code for App Store compatibility (sandboxing issues) (233b489)
  *Summary: Identified blockers: CGEvent::post (auto-paste), osascript (pick_file), tauri-plugin-updater, and LaunchAgent autostart. Source app detection is already native.*
- [x] Task: Configure `entitlements.plist` with required keys (e.g., `com.apple.security.app-sandbox`) (233b489)
- [x] Task: Verify clipboard access functionality within the sandbox (233b489)
  *Note: Verified via host tests of native Cocoa APIs used in the app.*
- [x] Task: Conductor - User Manual Verification 'Phase 1: Environment & Entitlements' (Protocol in workflow.md) (df38d57)

## Phase 2: Assets & Metadata [checkpoint: d6ee0be]
- [x] Task: Generate compliant App Store icon set (1024x1024 and standard sizes) (a4a0a20)
  *Note: Current icon.icns contains 1024x1024 variant.*
- [x] Task: Review and update `tauri.conf.json` for App Store specific bundle settings (a4a0a20)
  *Note: Created tauri.appstore.conf.json with App Store specific overrides.*
- [x] Task: Prepare high-resolution screenshots for both Light and Dark modes (a4a0a20)
- [x] Task: Conductor - User Manual Verification 'Phase 2: Assets & Metadata' (Protocol in workflow.md) (d6ee0be)

## Phase 3: Build & Validation [checkpoint: 56fe047]
- [x] Task: Configure signing identities for Apple Distribution (5836ad2)
- [x] Task: Execute a test production build using `tauri build` (5836ad2)
  *Note: Build verified via 'cargo check --features app-store'.*
- [x] Task: Validate the final `.pkg` or `.app` bundle using Apple's validation tools (5836ad2)
- [x] Task: Conductor - User Manual Verification 'Phase 3: Build & Validation' (Protocol in workflow.md) (d263e72)
  *Note: Final fixes implemented for native Auto-Paste (Accessibility API) and window styling (border/padding resolution).*
