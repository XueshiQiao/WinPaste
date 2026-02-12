# PastePaw Mac App Store Submission Guide

## Table of Contents

1. [Critical Code Changes Required](#1-critical-code-changes-required)
2. [Certificates & Provisioning Profiles](#2-certificates--provisioning-profiles)
3. [App Sandbox & Entitlements](#3-app-sandbox--entitlements)
4. [Tauri 2 Configuration for App Store](#4-tauri-2-configuration-for-app-store)
5. [Building the .pkg Installer](#5-building-the-pkg-installer)
6. [Uploading to App Store Connect](#6-uploading-to-app-store-connect)
7. [Support URL & Marketing URL](#7-support-url--marketing-url)
8. [App Review Considerations](#8-app-review-considerations)
9. [Quick Reference Build Script](#9-quick-reference-build-script)

---

## 1. Critical Code Changes Required

These issues **must** be resolved before submission — they will cause immediate rejection.

### 1a. Remove `macOSPrivateApi`

**Hard blocker.** `tauri.conf.json` has `"macOSPrivateApi": true` and `Cargo.toml` has `"macos-private-api"` in features. Apps using private macOS APIs are rejected.

For the App Store build:
- Set `macOSPrivateApi` to `false`
- Set `transparent` to `false` (transparency relies on the private API)
- Set `shadow` to `true`
- Window vibrancy via `window-vibrancy` crate should still work without the private API

### 1b. Replace `osascript` + System Events Calls

`clipboard.rs` uses `osascript` to communicate with System Events in two places:

**Source app detection (~line 526):**
```rust
let output = Command::new("osascript")
    .args(["-e", "tell application \"System Events\" to get {name, bundle identifier} of first application process whose frontmost is true"])
    .output();
```

**Paste simulation (~line 621):**
```rust
let result = Command::new("osascript")
    .args(["-e", "tell application \"System Events\" to keystroke \"v\" using command down"])
    .output();
```

Both will be **rejected**. The `com.apple.security.temporary-exception.apple-events` entitlement for System Events is not granted for App Store apps.

**Alternatives:**
- **Source app detection**: Use `NSWorkspace.shared.frontmostApplication` via Objective-C bridging. Works inside the sandbox.
- **Paste simulation**: Not possible in sandbox. App Store clipboard managers (like Maccy, PastePal) copy the item to the system clipboard and let the user paste manually with Cmd+V. Adopt this pattern.

### 1c. Replace Autostart LaunchAgent

`lib.rs` uses `MacosLauncher::LaunchAgent`, which writes plist files to `~/Library/LaunchAgents/`. This is **not allowed** in sandboxed apps.

**Alternatives:**
- Use `SMAppService` (macOS 13+) via native Swift/ObjC code
- Or remove the autostart feature from the App Store build

### 1d. Remove the Updater Plugin

`tauri-plugin-updater` is not needed for App Store builds — Apple handles updates. Remove it from the App Store build configuration.

### 1e. Icon Extraction Commands

`extract_macos_app_icon` in `clipboard.rs` (~line 550-613) uses `mdfind`, `/usr/libexec/PlistBuddy`, and `sips`. These may have restricted access inside the sandbox. Test thoroughly. Fallback: use `NSWorkspace` APIs to get app icons natively.

---

## 2. Certificates & Provisioning Profiles

### Step 1: Create a Certificate Signing Request (CSR)

1. Open **Keychain Access**
2. Menu: Keychain Access > Certificate Assistant > Request a Certificate From a Certificate Authority
3. Enter your email, leave CA Email empty, select "Saved to disk"
4. Save the `.certSigningRequest` file

### Step 2: Create Certificates

Go to [Apple Developer Certificates](https://developer.apple.com/account/resources/certificates/list) and create:

| Certificate Type | Purpose | Keychain Name |
|---|---|---|
| **Apple Distribution** | Signs the `.app` bundle | `Apple Distribution: Your Name (TEAM_ID)` |
| **Mac Installer Distribution** | Signs the `.pkg` installer | `3rd Party Mac Developer Installer: Your Name (TEAM_ID)` |

For each: click "+" > select type > upload CSR > download `.cer` > double-click to install.

### Step 3: Verify Certificates

```bash
security find-identity -v -p codesigning
security find-identity -p macappstore -v
```

### Step 4: Create an App ID

1. Go to [Identifiers](https://developer.apple.com/account/resources/identifiers/list)
2. Click "+" > "App IDs" > "App"
3. Set Bundle ID to `me.xueshi.pastepaw` (must match `tauri.conf.json`)
4. Register

### Step 5: Create a Provisioning Profile

1. Go to [Profiles](https://developer.apple.com/account/resources/profiles/list)
2. Click "+" > select **"Mac App Store Connect"** under Distribution
3. Select your App ID (`me.xueshi.pastepaw`)
4. Select your **Apple Distribution** certificate
5. Name it (e.g., `PastePaw_MAS_Distribution`)
6. Generate, download, and place the `.provisionprofile` file in `src-tauri/`

---

## 3. App Sandbox & Entitlements

Every Mac App Store app **must** be sandboxed.

### Create `src-tauri/Entitlements.plist`

```xml
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN"
  "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <!-- REQUIRED: App Sandbox -->
    <key>com.apple.security.app-sandbox</key>
    <true/>

    <!-- Team and App Identifier -->
    <key>com.apple.application-identifier</key>
    <string>YOUR_TEAM_ID.me.xueshi.pastepaw</string>
    <key>com.apple.developer.team-identifier</key>
    <string>YOUR_TEAM_ID</string>

    <!-- Network access (analytics/aptabase) -->
    <key>com.apple.security.network.client</key>
    <true/>

    <!-- File read/write for user-selected files (file picker) -->
    <key>com.apple.security.files.user-selected.read-write</key>
    <true/>
</dict>
</plist>
```

> Replace `YOUR_TEAM_ID` with your actual Team ID from [Apple Developer Membership](https://developer.apple.com/account/#/membership/).

### Clipboard Access in Sandbox

Good news: `NSPasteboard` works inside the sandbox without any special entitlement. The `tauri-plugin-clipboard-x` should continue to function. Clipboard monitoring via polling `NSPasteboard.general.changeCount` is the standard sandbox-compatible approach.

### Create `src-tauri/Info.plist`

```xml
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN"
  "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>ITSAppUsesNonExemptEncryption</key>
    <false/>
</dict>
</plist>
```

This avoids export compliance review (the app doesn't use non-exempt encryption).

---

## 4. Tauri 2 Configuration for App Store

Create a separate overlay config that layers on top of `tauri.conf.json`.

### Create `src-tauri/tauri.appstore.conf.json`

```json
{
    "app": {
        "macOSPrivateApi": false,
        "windows": [
            {
                "title": "PastePaw",
                "width": 1920,
                "height": 540,
                "minWidth": 700,
                "minHeight": 400,
                "resizable": false,
                "fullscreen": false,
                "decorations": false,
                "transparent": false,
                "shadow": true,
                "visible": false,
                "center": false,
                "label": "main"
            }
        ]
    },
    "bundle": {
        "targets": ["app"],
        "createUpdaterArtifacts": false,
        "category": "public.app-category.utilities",
        "macOS": {
            "entitlements": "./Entitlements.plist",
            "signingIdentity": "Apple Distribution",
            "files": {
                "embedded.provisionprofile": "./PastePaw_MAS_Distribution.provisionprofile"
            }
        }
    },
    "plugins": {
        "updater": {
            "active": false
        }
    }
}
```

### Version Numbers

Keep these in sync (currently all `1.1.6`):
- `package.json` → `"version"`
- `src-tauri/Cargo.toml` → `version`
- `src-tauri/tauri.conf.json` → `"version"`

Each App Store submission requires an incremented version or build number.

---

## 5. Building the .pkg Installer

### Step 1: Build the Universal App Bundle

```bash
pnpm tauri build --bundles app --target universal-apple-darwin \
  --config src-tauri/tauri.appstore.conf.json
```

Output: `src-tauri/target/universal-apple-darwin/release/bundle/macos/PastePaw.app`

For Apple Silicon only (if targeting macOS 12+), use `aarch64-apple-darwin` instead.

### Step 2: Verify Code Signing

```bash
codesign --verify --deep --strict --verbose=2 \
  src-tauri/target/universal-apple-darwin/release/bundle/macos/PastePaw.app
```

If signing was not applied correctly, re-sign manually:

```bash
codesign --force --deep --sign "Apple Distribution: Your Name (TEAM_ID)" \
  --entitlements src-tauri/Entitlements.plist \
  src-tauri/target/universal-apple-darwin/release/bundle/macos/PastePaw.app
```

### Step 3: Create the .pkg

The Mac App Store requires `.pkg`, **not** `.dmg`:

```bash
xcrun productbuild \
  --sign "3rd Party Mac Developer Installer: Your Name (TEAM_ID)" \
  --component "src-tauri/target/universal-apple-darwin/release/bundle/macos/PastePaw.app" \
  /Applications \
  "PastePaw.pkg"
```

### Step 4: Validate

```bash
xcrun altool --validate-app --file "PastePaw.pkg" \
  --type macos \
  --apiKey YOUR_API_KEY_ID \
  --apiIssuer YOUR_API_ISSUER_ID
```

---

## 6. Uploading to App Store Connect

### Option A: Transporter App (Recommended for First Time)

1. Download [Transporter](https://apps.apple.com/us/app/transporter/id1450874784) from the Mac App Store
2. Sign in with your Apple Developer account
3. Drag and drop `PastePaw.pkg`
4. Click "Deliver"

Clear UI with detailed error messages — best for first-time submissions.

### Option B: Command Line (xcrun altool)

First, create an App Store Connect API Key:

1. Go to [App Store Connect > Users and Access > Integrations > API](https://appstoreconnect.apple.com/access/integrations/api)
2. Click "+" > generate key with "Developer" role
3. Download the `.p8` file (one-time download)
4. Note the **Key ID** and **Issuer ID**
5. Place `.p8` in `~/.appstoreconnect/private_keys/`

Then upload:

```bash
xcrun altool --upload-app \
  --type macos \
  --file "PastePaw.pkg" \
  --apiKey YOUR_API_KEY_ID \
  --apiIssuer YOUR_API_ISSUER_ID
```

### After Upload

1. Build appears in **App Store Connect > Your App > TestFlight** within minutes
2. Apple runs automated checks (~30 minutes)
3. If it passes, submit for review from App Store Connect
4. Can also distribute via TestFlight for testing first

---

## 7. Support URL & Marketing URL

### What Apple Requires

| Field | Required? | Description |
|---|---|---|
| **Support URL** | Yes | Where users go for help / bug reports |
| **Marketing URL** | No | Landing page showcasing the app |
| **Privacy Policy URL** | Yes | Your app's privacy policy |

### Support URL Requirements

- Must clearly identify the app name
- Must include at least one contact method (email, contact form, or ticket system)
- Must load on all devices including mobile
- Cannot be a social media profile
- Cannot redirect to the App Store listing itself

### Easiest Approach: GitHub Pages (Free)

1. Create a `docs/site/` folder (or use a `gh-pages` branch)
2. Add `index.html` with:
   - App name, icon, description
   - Features list + screenshots
   - "Support" section with contact email
   - Link to GitHub Issues
3. Add `privacy.html` for the privacy policy
4. Enable GitHub Pages in repo settings (Settings > Pages > Source: Deploy from branch)

URLs become:
- **Support / Marketing URL**: `https://xueshiqiao.github.io/PastePaw/`
- **Privacy Policy URL**: `https://xueshiqiao.github.io/PastePaw/privacy.html`

Alternative: use the GitHub repo URL (`https://github.com/XueshiQiao/PastePaw`) as the Support URL — it works since users can file Issues there.

### Privacy Policy Content

PastePaw's privacy policy should cover:
- Clipboard data is stored **locally on the user's device only**
- No clipboard data is transmitted to any server
- Anonymous usage analytics are collected via Aptabase (describe what's collected)
- No personal information is sold or shared
- Users can clear all stored data from within the app

---

## 8. App Review Considerations

### Common Rejection Reasons for Clipboard Managers

1. **Private API Usage** — `macOSPrivateApi: true` causes immediate rejection. Automated scanning catches undocumented APIs.

2. **Accessibility Permission in Sandbox** — Auto-paste via `osascript` + System Events requires Accessibility permission, which is unavailable in the sandbox. Solution: copy to clipboard only, let the user paste.

3. **Sandbox Violations** — Writing to `~/Library/LaunchAgents/` (autostart), executing commands that access restricted resources, file access outside sandbox container.

4. **Metadata Mismatch** — Screenshots must accurately represent the app. Description must match actual functionality.

5. **Privacy Concerns** — Apple is strict about clipboard access. Clearly explain why your app reads the clipboard in the App Store description.

6. **Guideline 4.2 (Minimum Functionality)** — Apple may reject if the app is too simple or duplicative. Ensure clear, unique value.

### macOS Clipboard Privacy (macOS 16+)

Starting with macOS 16, the system shows a privacy prompt when apps read the clipboard without direct user interaction (similar to iOS). Users can set it to "Always Allow" for clipboard managers. Be prepared to handle this prompt gracefully.

### Feature Comparison: Current vs App Store Version

| Feature | Current | App Store Version |
|---|---|---|
| Window transparency | Private API | Vibrancy without transparency |
| Clipboard monitoring | `tauri-plugin-clipboard-x` | Same (verify in sandbox) |
| Source app detection | `osascript` + System Events | `NSWorkspace.shared.frontmostApplication` |
| Paste simulation | `osascript` keystroke | Remove; copy to clipboard only |
| Autostart | LaunchAgent plist | `SMAppService` (macOS 13+) or remove |
| Auto-update | `tauri-plugin-updater` | Remove; App Store handles updates |
| Global hotkey | `tauri-plugin-global-shortcut` | Should work (test in sandbox) |
| Icon extraction | `mdfind` + `PlistBuddy` + `sips` | Test in sandbox; may need NSWorkspace fallback |
| Analytics | `tauri-plugin-aptabase` | Should work (uses network) |

---

## 9. Quick Reference Build Script

Once all code changes are complete:

```bash
#!/bin/bash
set -e

APP_NAME="PastePaw"
TEAM_ID="YOUR_TEAM_ID"
SIGN_APP="Apple Distribution: Your Name ($TEAM_ID)"
SIGN_PKG="3rd Party Mac Developer Installer: Your Name ($TEAM_ID)"
API_KEY="YOUR_API_KEY_ID"
API_ISSUER="YOUR_API_ISSUER_ID"

# 1. Build
cd /Users/joey/Code/PastePaw
pnpm tauri build --bundles app --target universal-apple-darwin \
  --config src-tauri/tauri.appstore.conf.json

# 2. Verify signing
APP_PATH="src-tauri/target/universal-apple-darwin/release/bundle/macos/$APP_NAME.app"
codesign --verify --deep --strict --verbose=2 "$APP_PATH"

# 3. Build .pkg
xcrun productbuild \
  --sign "$SIGN_PKG" \
  --component "$APP_PATH" /Applications \
  "$APP_NAME.pkg"

# 4. Validate
xcrun altool --validate-app --file "$APP_NAME.pkg" \
  --type macos --apiKey "$API_KEY" --apiIssuer "$API_ISSUER"

# 5. Upload
xcrun altool --upload-app --file "$APP_NAME.pkg" \
  --type macos --apiKey "$API_KEY" --apiIssuer "$API_ISSUER"

echo "Upload complete. Check App Store Connect for build status."
```

---

## Work Summary

| Priority | Task | Effort |
|---|---|---|
| **P0** | Remove `macOSPrivateApi`, rework window styling | Medium |
| **P0** | Replace `osascript` System Events with `NSWorkspace` API | Medium |
| **P0** | Remove auto-paste, switch to copy-only | Medium |
| **P1** | Fix autostart (`SMAppService` or remove) | Medium |
| **P1** | Create certificates + provisioning profile | Small |
| **P1** | Create entitlements + App Store config | Small |
| **P1** | Create Support / Privacy pages (GitHub Pages) | Small |
| **P2** | Remove updater plugin for App Store build | Small |
