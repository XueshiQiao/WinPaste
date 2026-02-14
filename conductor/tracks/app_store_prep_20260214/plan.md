# Implementation Plan: Prepare for Apple App Store Submission

## Phase 1: Environment & Entitlements
- [x] Task: Audit current macOS code for App Store compatibility (sandboxing issues) (233b489)
  *Summary: Identified blockers: CGEvent::post (auto-paste), osascript (pick_file), tauri-plugin-updater, and LaunchAgent autostart. Source app detection is already native.*
- [x] Task: Configure `entitlements.plist` with required keys (e.g., `com.apple.security.app-sandbox`) (233b489)
- [x] Task: Verify clipboard access functionality within the sandbox (233b489)
  *Note: Verified via host tests of native Cocoa APIs used in the app.*
- [~] Task: Conductor - User Manual Verification 'Phase 1: Environment & Entitlements' (Protocol in workflow.md)
- [ ] Task: Conductor - User Manual Verification 'Phase 1: Environment & Entitlements' (Protocol in workflow.md)

## Phase 2: Assets & Metadata
- [ ] Task: Generate compliant App Store icon set (1024x1024 and standard sizes)
- [ ] Task: Review and update `tauri.conf.json` for App Store specific bundle settings
- [ ] Task: Prepare high-resolution screenshots for both Light and Dark modes
- [ ] Task: Conductor - User Manual Verification 'Phase 2: Assets & Metadata' (Protocol in workflow.md)

## Phase 3: Build & Validation
- [ ] Task: Configure signing identities for Apple Distribution
- [ ] Task: Execute a test production build using `tauri build`
- [ ] Task: Validate the final `.pkg` or `.app` bundle using Apple's validation tools
- [ ] Task: Conductor - User Manual Verification 'Phase 3: Build & Validation' (Protocol in workflow.md)
