# PastePaw Distribution: App Store vs Direct Sales

## App Store Obstacles

### Blockers

#### 1. `macos-private-api` Flag

```toml
# Cargo.toml
tauri = { version = "2", features = ["macos-private-api", ...] }
```

```json
// tauri.conf.json
"macOSPrivateApi": true
```

Enables private macOS API access for transparent, borderless windows. Apple rejects apps using private APIs.

#### 2. Paste Simulation via `osascript`

```rust
// clipboard.rs:621
Command::new("osascript")
    .args(["-e", "tell application \"System Events\" to keystroke \"v\" using command down"])
```

Simulating keyboard events requires Accessibility permission and is blocked by App Store sandboxing. This is the core feature of a clipboard manager — and fundamentally incompatible with the sandbox.

#### 3. Source App Detection via `osascript`

```rust
// clipboard.rs:526
Command::new("osascript")
    .args(["-e", "tell application \"System Events\" to get {name, bundle identifier} of first application process whose frontmost is true"])
```

Requires Automation/Accessibility entitlements. Sandboxed apps cannot query other apps via System Events.

#### 4. Icon Extraction via Subprocesses

```rust
// clipboard.rs:554-612
Command::new("mdfind")                  // Spotlight query
Command::new("/usr/libexec/PlistBuddy") // plist reading
Command::new("sips")                    // image conversion
```

Sandboxed apps cannot spawn arbitrary subprocesses. Would need to be replaced with native Cocoa APIs (NSWorkspace, Core Spotlight, etc.).

#### 5. File Picker via `osascript`

```rust
// commands.rs:949
Command::new("osascript")
    .args(["-e", r#"POSIX path of (choose file of type {"app"} ...)"#])
```

Same subprocess restriction. Should use native file dialog APIs instead.

#### 6. No App Sandbox Entitlements

Mac App Store **requires** app sandboxing. The app has no `.entitlements` file. Many current features (global shortcuts, paste simulation, subprocess spawning) are incompatible with sandboxing.

#### 7. Custom Updater Plugin

```rust
.plugin(tauri_plugin_updater::Builder::new().build())
```

App Store apps must use Apple's update mechanism (App Store Review Guideline 2.4.5). Custom updaters are prohibited.

#### 8. Autostart via LaunchAgent

```rust
.plugin(tauri_plugin_autostart::init(MacosLauncher::LaunchAgent, ...))
```

Writing to `~/Library/LaunchAgents/` is blocked in sandbox. App Store apps must use `SMLoginItemSetEnabled` / Service Management framework.

### Risky (May Cause Extra Scrutiny)

#### 9. Global Shortcut Registration

Requires Input Monitoring permission. Not a private API, but Apple has become stricter about apps that listen to global keyboard events.

#### 10. CSP Disabled

```json
"csp": null
```

Disabling Content Security Policy may raise concerns during security review.

#### 11. Analytics (`tauri-plugin-aptabase`)

Requires a privacy policy and App Tracking Transparency declaration.

### Summary Table

| Issue                              | Severity | Fix Difficulty |
|------------------------------------|----------|----------------|
| `macos-private-api` flag           | BLOCKER  | Hard           |
| Paste simulation via `osascript`   | BLOCKER  | Very Hard      |
| Source app detection via `osascript`| BLOCKER  | Medium         |
| Icon extraction via subprocesses   | BLOCKER  | Medium         |
| File picker via `osascript`        | BLOCKER  | Easy           |
| No sandbox entitlements            | BLOCKER  | Hard           |
| Custom updater plugin              | BLOCKER  | Easy           |
| LaunchAgent autostart              | BLOCKER  | Medium         |
| Global shortcuts                   | Risky    | Low            |
| CSP disabled                       | Minor    | Easy           |

### If Targeting App Store

The most realistic path:

1. Remove auto-paste — only copy to clipboard, let the user paste manually
2. Rewrite source app detection and icon extraction using Cocoa/AppKit APIs (NSWorkspace, NSRunningApplication)
3. Replace `osascript` file picker with native file dialog
4. Remove `tauri-plugin-updater`
5. Switch autostart to Service Management framework
6. Add proper sandbox entitlements
7. Remove `macos-private-api` flag and find alternative window styling

Most serious clipboard managers (Paste, Maccy, CopyClip) are distributed either outside the App Store or on the App Store without auto-paste.

---

## Selling Outside the App Store

### Payment & License Platforms

| Platform      | Cut              | Notes                                                                 |
|---------------|------------------|-----------------------------------------------------------------------|
| Paddle        | ~5-10%           | Most popular for indie Mac apps. Handles payments, taxes, VAT, license keys. |
| LemonSqueezy  | 5-8%             | Modern Paddle alternative. Handles global tax compliance. Has license key API. |
| Gumroad       | 10%              | Simple setup, good for indie devs. No built-in license key system.    |
| FastSpring    | Similar to Paddle| Strong international tax handling.                                    |
| Stripe        | 2.9% + $0.30     | Lowest cut, but you handle tax, license keys, and storefront yourself.|

For comparison, Apple takes **30%** (15% for Small Business Program under $1M/year).

### Typical Setup

1. **Paddle or LemonSqueezy** for payment + license keys
2. **Notarize the app** with Apple (free with $99/year Developer account)
3. Distribute via **website** as a `.dmg`
4. `tauri-plugin-updater` for auto-updates (already implemented)
5. **Free trial** (e.g., 14 days or limited history) with paid unlock via license key

### What PastePaw Already Has

- `tauri-plugin-updater` handles auto-updates
- Notarization works without sandboxing
- All features (paste simulation, global hotkeys, source app detection) work fully outside the sandbox

### What Needs to Be Added

1. **License key validation** — check key on first launch, store in settings DB
2. **Trial mode** — time-limited or feature-limited free usage
3. **Storefront page** — landing page on website with checkout integration
4. **Payment integration** — Paddle/LemonSqueezy checkout flow

### Successful Mac Apps Sold Outside the App Store

- **Raycast** — free + pro subscription, direct download
- **Alfred** — Powerpack sold via their site
- **CleanShot X** — sold via Paddle
- **Bartender** — sold via Paddle
- **Keyboard Maestro** — direct sales
- **Paste** (clipboard manager) — was App Store, moved to Setapp

### Recommendation

Direct sales is the better path for PastePaw because:

- No sandbox restrictions — all features work as-is
- No review delays — ship updates instantly
- Higher revenue share (90-97% vs 70-85%)
- Full control over pricing, trials, and distribution
- Most successful Mac utilities use this model
