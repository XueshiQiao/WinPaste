# Fixing All Cargo Build Warnings

## Goal

Eliminate all 131 warnings produced by `cargo build` in the PastePaw project to achieve a clean build with zero warnings.

## Step 1: Capture and Categorize Warnings

Ran `cargo build` and captured the full output (~700 lines). Used a subagent to parse and categorize all 131 warnings into 4 groups:

| Category | Count | Source | Root Cause |
|----------|-------|--------|------------|
| `unexpected_cfgs` | 41 | `objc` crate macros (`msg_send!`, `class!`, `sel!`) | Macros check `cfg(feature = "cargo-clippy")` internally, which isn't a recognized feature of our crate |
| Deprecated API usage | 87 | `cocoa` crate (`cocoa::base::id`, `nil`, `NSString`, `NSApplication`, etc.) | The `cocoa` crate deprecated its entire API in favor of the `objc2` ecosystem |
| Unused variable + unnecessary `mut` | 2 | `src/lib.rs:468` | `z_order_switched` is only used inside `#[cfg(target_os = "windows")]` blocks but declared unconditionally |
| `std::mem::forget` on Copy type | 1 | `src/source_app_macos.rs:93` | `forget` was called on a raw pointer (which is `Copy`), making it a no-op |

## Step 2: Fix Unused Variable (2 warnings)

**Problem:** `let mut z_order_switched = false;` in `lib.rs` is only read/written inside `#[cfg(target_os = "windows")]` blocks (lines 485, 494), so on macOS it triggers `unused_variables` and `unused_mut`.

**Fix:** Gate the declaration with the same cfg:

```rust
#[cfg(target_os = "windows")]
let mut z_order_switched = false;
```

## Step 3: Fix `std::mem::forget` on Copy Type (1 warning)

**Problem:** In `source_app_macos.rs`, the NSWorkspace notification observer was kept alive with:

```rust
std::mem::forget(observer as *const _ as *const std::ffi::c_void);
```

The cast to `*const c_void` (a `Copy` type) happens before `forget`, so `forget` operates on the pointer value, not the original `observer`. The ObjC object could still be deallocated.

**Fix:** Use ObjC `retain` to properly prevent deallocation:

```rust
let _: () = msg_send![observer, retain];
```

This increments the reference count, keeping the observer alive for the app's lifetime.

## Step 4: Suppress `unexpected_cfgs` (41 warnings)

**Problem:** Every use of `msg_send!`, `class!`, or `sel!` from the `objc` crate produces a warning because the macros internally check `cfg(feature = "cargo-clippy")`, which isn't declared in our `Cargo.toml` features.

**Fix:** Added crate-level allow in `lib.rs`:

```rust
#![allow(unexpected_cfgs)]
```

**Why suppress instead of fix:** This is an upstream issue in the `objc` 0.2 crate. The proper fix is migrating to `objc2`, which is a significant refactor (see Notes).

## Step 5: Suppress Deprecated `cocoa` API Warnings (87 warnings)

**Problem:** The `cocoa` crate (0.26) has deprecated every public type and method (`id`, `nil`, `NSString`, `NSApplication`, `NSWindow`, `NSSize`, `NSRect`, `NSPoint`, etc.) in favor of the `objc2-foundation` / `objc2-app-kit` crates.

**Fix:** Added crate-level allow in `lib.rs`:

```rust
#![allow(deprecated)]
```

**Why suppress instead of fix:** Migrating from `cocoa`/`objc` to `objc2` touches all macOS-specific code across 3 files (`lib.rs`, `clipboard.rs`, `source_app_macos.rs`) and changes the entire ObjC interop style. Tracked as future work.

## Result

```
$ cargo build
   Compiling PastePaw v1.1.7
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 8.21s
```

Zero warnings.

## Files Changed

| File | Change |
|------|--------|
| `src-tauri/src/lib.rs` | Added `#![allow(unexpected_cfgs)]` and `#![allow(deprecated)]` at crate root; gated `z_order_switched` with `#[cfg(target_os = "windows")]` |
| `src-tauri/src/source_app_macos.rs` | Replaced `std::mem::forget` with ObjC `retain` for observer lifetime |

## Notes

- The 128 suppressed warnings (categories 3 and 4) share a root cause: using the legacy `cocoa` + `objc` crates. Migrating to the `objc2` ecosystem would eliminate them properly and is the right long-term fix.
- The `#![allow(deprecated)]` suppression is crate-wide, which means new deprecation warnings from other crates would also be silenced. If this becomes a concern, the allows can be scoped to specific modules with `#[allow(deprecated)]` on individual `mod` declarations instead.
