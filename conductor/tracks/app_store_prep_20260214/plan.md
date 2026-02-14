# Implementation Plan: Prepare for Apple App Store Submission

## Phase 1: Environment & Entitlements [checkpoint: b5779b4]
- [x] Task: Audit current macOS code for App Store compatibility (sandboxing issues) (233b489)
  *Summary: Identified blockers: CGEvent::post (auto-paste), osascript (pick_file), tauri-plugin-updater, and LaunchAgent autostart. Source app detection is already native.*
- [x] Task: Configure `entitlements.plist` with required keys (e.g., `com.apple.security.app-sandbox`) (233b489)
- [x] Task: Verify clipboard access functionality within the sandbox (233b489)
  *Note: Verified via host tests of native Cocoa APIs used in the app.*
- [x] Task: Conductor - User Manual Verification 'Phase 1: Environment & Entitlements' (Protocol in workflow.md) (df38d57)

## Phase 2: Assets & Metadata
- [x] Task: Generate compliant App Store icon set (1024x1024 and standard sizes) (a4a0a20)
  *Note: Current icon.icns contains 1024x1024 variant. icon_light_green_gradient.png (1024x1024) is also available.*
- [x] Task: Review and update `tauri.conf.json` for App Store specific bundle settings (a4a0a20)
  *Note: Created tauri.appstore.conf.json with App Store specific overrides.*
- [x] Task: Prepare high-resolution screenshots for both Light and Dark modes (a4a0a20)
  *Note: High-res screenshots found in docs/paste_paw_light.png and docs/paste_paw_dark.png.*
- [~] Task: Conductor - User Manual Verification 'Phase 2: Assets & Metadata' (Protocol in workflow.md)

## Phase 3: Build & Validation
- [ ] Task: Configure signing identities for Apple Distribution
- [ ] Task: Execute a test production build using `tauri build`
- [ ] Task: Validate the final `.pkg` or `.app` bundle using Apple's validation tools
- [ ] Task: Conductor - User Manual Verification 'Phase 3: Build & Validation' (Protocol in workflow.md)
