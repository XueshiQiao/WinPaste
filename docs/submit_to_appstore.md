# PastePaw Mac App Store Submission Guide

This document summarizes the technical changes and steps required to build and submit PastePaw to the Mac App Store.

## 1. Technical Changes Implemented

The following changes were made to ensure compatibility with the macOS App Sandbox:

### A. Autostart (Login Items)
- **Old Method**: `LaunchAgent` (Prohibited in sandbox).
- **New Method**: `SMAppService` (macOS 13+).
- **Implementation**: Uses the `smappservice-rs` crate. When the `app-store` feature is enabled, the app uses native Service Management APIs to register as a login item.

### B. File Picker
- **Old Method**: `osascript` / `powershell` (Prohibited subprocess calls).
- **New Method**: `tauri-plugin-dialog`.
- **Implementation**: Uses native system dialogs which are sandbox-compatible.

### C. Source App Detection
- **Status**: Already uses native Cocoa `NSWorkspace` APIs. Verified as sandbox-compatible.

### D. Auto-Paste (Accessibility API)
- **Status**: Implemented via `AXIsProcessTrusted` and `CGEvent`.
- **Sandbox Compliance**: Instead of using prohibited `osascript` calls, the app now uses the official Apple-recommended Accessibility API path.
- **User Flow**: If permissions are missing, a banner appears in Settings guiding the user to *System Settings > Privacy & Security > Accessibility*. Once granted, the app can simulate the `Cmd+V` keystroke from within the sandbox.

### E. Conditional Compilation
- Added an `app-store` feature flag in `Cargo.toml`.
- When building with `--features app-store`:
    - `tauri-plugin-updater` is disabled (App Store handles updates).
    - `tauri-plugin-autostart` (LaunchAgent version) is disabled.
    - Private macOS APIs are disabled.

---

## 2. Configuration Files

### Entitlements.plist (`src-tauri/Entitlements.plist`)
This file defines the permissions for the sandboxed app.
**Action Required**: Replace `YOUR_TEAM_ID` with your actual Apple Team ID.

```xml
<key>com.apple.security.app-sandbox</key>
<true/>
<key>com.apple.application-identifier</key>
<string>YOUR_TEAM_ID.me.xueshi.pastepaw</string>
<key>com.apple.developer.team-identifier</key>
<string>YOUR_TEAM_ID</string>
```

### App Store Config (`src-tauri/tauri.appstore.conf.json`)
An overlay configuration used during the build process to override default settings for the App Store.
- Disables `macOSPrivateApi`.
- Disables `transparent` windows (not allowed without private APIs).
- Enables `shadows`.
- Sets the category to `public.app-category.utilities`.

---

## 3. Build Instructions

### Prerequisites
1.  **Certificates**: Ensure you have "Apple Distribution" and "Mac Installer Distribution" certificates in your Keychain.
2.  **Provisioning Profile**: Download a Mac App Store Distribution profile and place it in `src-tauri/embedded.provisionprofile` if required.

### Execution
Run the automated build script:
```bash
./build_appstore.sh
```

This script will:
1.  Build a **Universal Binary** (Intel + Apple Silicon).
2.  Apply the App Store entitlements.
3.  Create a `.pkg` installer (required for App Store submission).

---

## 4. Submission Steps

1.  **Validation**: Use the **Transporter** app (available on Mac App Store) to drag and drop your `PastePaw.pkg`. Click "Verify".
2.  **Upload**: If verification passes, click "Deliver" in Transporter.
3.  **App Store Connect**:
    - Select the uploaded build.
    - Use the high-resolution screenshots found in `docs/screenshot_macos_light.png` and `docs/screenshot_macos_dark.png`.
    - Provide the Support URL (e.g., your GitHub repo issues page).

---

## 5. Verification Commands

To verify the app is ready for App Store without a full build:

**Check Compilation:**
```bash
cd src-tauri
cargo check --features app-store
```

**Test Native APIs:**
```bash
cd src-tauri
cargo test test_get_frontmost_app_info -- --nocapture
```
