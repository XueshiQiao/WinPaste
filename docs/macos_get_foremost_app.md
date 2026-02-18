# macOS Source App Detection — Troubleshooting & Optimization

## Goal

When the user copies something to the clipboard, identify which app the content came from and display its name and icon on the clip card.

## Original Implementation (Subprocess-based)

The initial macOS implementation used **5 subprocess calls** via `std::process::Command`:

- `osascript` — ran AppleScript to get the frontmost app name and bundle ID from System Events
- `mdfind` — searched Spotlight for the `.app` bundle path by bundle ID
- `PlistBuddy` — read `CFBundleIconFile` from the app's `Info.plist`
- `sips` — converted the `.icns` icon file to a 64×64 PNG in `/tmp`
- Then read the PNG file and base64-encoded it

**Problem:** All of these violate App Store sandbox rules. Sandboxed apps cannot spawn arbitrary subprocesses.

## Step 1: Replace with Native Cocoa APIs

Created `src-tauri/src/source_app_macos.rs` using the `cocoa` and `objc` crates (already in `Cargo.toml`):

- `NSWorkspace.sharedWorkspace.frontmostApplication` → `NSRunningApplication`
- `.localizedName` → app name
- `.bundleIdentifier` → bundle ID
- `.icon` → `NSImage`, then converted to base64 PNG via:
  - Create a new 64×64 `NSImage`
  - `lockFocus` → draw source icon → `unlockFocus`
  - `TIFFRepresentation` → `NSBitmapImageRep` → `representationUsingType:properties:` (PNG type = 4)
  - Read `NSData` bytes → base64-encode

Updated `clipboard.rs` to delegate the macOS `get_clipboard_owner_app_info()` to the new module.

## Step 2: Race Condition — PastePaw Detected as Source App

**Problem:** User copies in App A, then quickly uses the hotkey to bring up PastePaw. By the time `get_frontmost_app_info()` runs (after a 150ms debounce delay), PastePaw is already frontmost, so it's incorrectly reported as the source.

**First attempt:** Capture source app info **immediately** in the clipboard event listener callback (before the debounce delay), then pass it to `process_clipboard_change()`. This helped but didn't fully solve it — the user could switch before even the event fired.

**Second attempt — filter out PastePaw:** If `frontmostApplication` returns PastePaw, return `(None, ..., false)` instead. This prevented wrong attribution but lost the source info entirely.

## Step 3: NSWorkspace App Activation Observer (Final Solution)

**Insight:** Instead of querying the frontmost app reactively, **track it continuously** so the correct answer is always ready.

Registered an observer for `NSWorkspaceDidActivateApplicationNotification`:

- Created an Objective-C class (`PastePawAppObserver`) via `objc::declare::ClassDecl`
- On every app activation, if the app is NOT PastePaw, cache its name, icon (base64 PNG), and bundle ID in a `Mutex<Option<CachedAppInfo>>` static
- The observer is leaked intentionally (lives for the entire app lifetime)
- Called `start_frontmost_app_observer()` once at startup in `lib.rs`

Updated `get_frontmost_app_info()` logic:

1. Check `frontmostApplication` — if it's NOT PastePaw, use it directly
2. If it IS PastePaw, fall back to the cached `LAST_FOREGROUND_APP`

This eliminates the race condition entirely because the observer fires via macOS notifications in real-time, so the cached value is always the correct previous app.

## Step 4: PastePaw Bundle ID is `None` in Dev Mode

**Problem:** In development builds (not running from a `.app` bundle), `NSRunningApplication.bundleIdentifier` returns `nil`. The `OWN_BUNDLE_ID` (`"me.xueshi.pastepaw"`) check never matched, so PastePaw was not filtered out.

**Fix:** Added `is_own_app()` helper that checks **both** the bundle ID and the app name (`"PastePaw"`). Works in both dev and production.

## Step 5: Duplicate Clips Showing Stale Source App

**Problem:** Copy text "hello" from VSCode → clip card shows "Code". Later copy the same "hello" from WeChat → clip card still shows "Code".

**Root cause:** In `clipboard.rs`, the duplicate detection path (`UPDATE clips SET created_at = ... WHERE uuid = ?`) only updated the timestamp — it did NOT update `source_app` or `source_icon`.

**Fix:** Changed the UPDATE query to also set `source_app = ?` and `source_icon = ?` so re-copied content reflects the new source.

## Step 6: Cleanup — Removed Experimental Code

During debugging, we added `org.nspasteboard.source` pasteboard reading (a convention where some apps write their bundle ID to the pasteboard). After investigation:

- Most apps do NOT set this, so it returned `None` in practice
- It was never wired into the actual source detection logic
- Removed `get_pasteboard_source_bundle_id()` and its log

Also removed the redundant `"Captured source app at event time"` debug log from `clipboard.rs` since `get_frontmost_app_info()` already logs the result.

## Final Architecture

```
App startup (lib.rs)
  └─ start_frontmost_app_observer()
       └─ Observes NSWorkspaceDidActivateApplicationNotification
       └─ Caches last non-PastePaw app in LAST_FOREGROUND_APP

Clipboard event fires (clipboard.rs)
  └─ get_clipboard_owner_app_info()  [captured at event time, before debounce]
       └─ get_frontmost_app_info()  (source_app_macos.rs)
            ├─ frontmostApplication is NOT PastePaw → use directly
            └─ frontmostApplication IS PastePaw → use LAST_FOREGROUND_APP cache
  └─ 150ms debounce
  └─ process_clipboard_change() uses the pre-captured info
       ├─ New clip → INSERT with source_app, source_icon
       └─ Duplicate → UPDATE source_app, source_icon, created_at
```

## Files Changed

| File | Change |
|------|--------|
| `src-tauri/src/source_app_macos.rs` | **New** — native Cocoa source app detection + app activation observer |
| `src-tauri/src/clipboard.rs` | Removed old subprocess functions; delegates to new module; captures source app before debounce; updates source on duplicates |
| `src-tauri/src/lib.rs` | Registered `source_app_macos` module; calls `start_frontmost_app_observer()` at startup |
| `docs/TODO.md` | Marked "grab application name" as done |

## Note: Remaining `osascript` Usage

`commands.rs` has a `pick_file` command that uses `osascript` to show a file picker on macOS. This is a separate App Store violation to address in a future task — should use Tauri's dialog plugin or native `NSOpenPanel` instead.
