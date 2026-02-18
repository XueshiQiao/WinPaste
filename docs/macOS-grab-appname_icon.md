Plan to implement

Make source app tracking App Store compliant on macOS

Context

The "grab application name and icon" feature is already fully implemented end-to-end: backend captures source app info, stores it in the DB (source_app, source_icon columns), and the ClipCard UI already displays them in the header with color-coded backgrounds.

However, the macOS implementation uses 5 subprocess calls (osascript, mdfind, PlistBuddy, sips) via std::process::Command, which violates App Store sandbox rules. These must be replaced with native Cocoa APIs.

The Windows implementation is unaffected — it already uses native Windows APIs.

Current macOS code (to be replaced)

clipboard.rs:522-547 — get_clipboard_owner_app_info(): uses osascript to run AppleScript
clipboard.rs:549-613 — extract_macos_app_icon(): uses mdfind, PlistBuddy, sips
Changes

Create src-tauri/src/source_app_macos.rs (new file)
A dedicated module for macOS source app detection using native Cocoa APIs. Contains two public functions:

pub fn get_frontmost_app_info() -> (Option<String>, Option<String>, Option<String>, Option<String>, bool)
Returns (app_name, icon_base64, bundle_id, bundle_id, true) — same signature as current function.

Implementation using objc/cocoa crates (already in Cargo.toml):

NSWorkspace.sharedWorkspace.frontmostApplication → NSRunningApplication
.localizedName → app name string
.bundleIdentifier → bundle ID string
.icon → NSImage for icon
fn nsimage_to_base64_png(image: id, size: f64) -> Option<String>
Private helper to convert NSImage to a base64 PNG string:

Create a new NSImage at target size (64×64)
lockFocus, draw source icon into it, unlockFocus
Get TIFFRepresentation → NSBitmapImageRep
representationUsingType:properties: with PNG type (4)
Read bytes, base64-encode
Update src-tauri/src/clipboard.rs
Remove the two #[cfg(target_os = "macos")] functions: get_clipboard_owner_app_info() (lines 522-547) and extract_macos_app_icon() (lines 549-613)
Replace with a #[cfg(target_os = "macos")] wrapper that delegates to the new module:
#[cfg(target_os = "macos")]
fn get_clipboard_owner_app_info() -> (Option<String>, Option<String>, Option<String>, Option<String>, bool) {
crate::source_app_macos::get_frontmost_app_info()
}
Register module in src-tauri/src/lib.rs or src-tauri/src/main.rs
Add #[cfg(target_os = "macos")] mod source_app_macos; alongside the other module declarations.

Update docs/TODO.md
Mark "grab application name" as done.

Additional note: pick_file also uses osascript

commands.rs:950-965 has a pick_file command that uses osascript to show a file picker on macOS. This is a separate App Store violation to address in a future task (out of scope here, but noted). It should use Tauri's dialog plugin or native NSOpenPanel instead.

What does NOT change

Windows implementation in clipboard.rs (lines 268-469) — untouched
Database schema — no changes needed
Frontend (ClipCard, types) — no changes needed, already displays source_app and source_icon
send_paste_input() on macOS — already uses native core_graphics CGEvent, no issue
Verification

cargo build succeeds with no new warnings related to our changes
pnpm build still succeeds (no frontend changes)
Launch app on macOS, copy text from different apps (Safari, Terminal, etc.), verify clip cards show correct app name and icon in header

