# macOS Main Thread Crash Fix

## Problem

When pressing the hotkey, the app could crash with `EXC_BREAKPOINT (SIGTRAP)` and the message:

```
Must only be used from the main thread
```

The crash occurred on a `tokio-runtime-worker` thread (Thread 16 in the crash log), inside AppKit's `NSWMWindowCoordinator performTransactionUsingBlock:`.

### Crash Stack (abbreviated)

```
Thread 16 Crashed:: tokio-runtime-worker

0  AppKit  -[NSWMWindowCoordinator performTransactionUsingBlock:]   <-- CRASH
1  AppKit  -[NSWindow(NSWMWindowManagement) window:didUpdateWithChangedProperties:]
2  WindowManagement  -[_WMWindow performUpdatesUsingBlock:]
3  WindowManagement  -[_WMWindow applyTags:mask:]
4  AppKit  -[NSWindow _tempHide:relWin:]
5  AppKit  -[NSApplication _doHideMaybeFakingIt:]
6  AppKit  -[NSApplication hide:]
7  PastePaw  objc::message::MessageArguments::invoke
8  PastePaw  objc::message::platform::send_unverified
9  PastePaw  PastePaw::animate_window_hide::{{closure}}
   ... (tokio runtime frames)
```

## Analysis

### Root Cause

macOS AppKit strictly requires **all UI operations to run on the main thread**. Both `animate_window_show()` and `animate_window_hide()` used `tauri::async_runtime::spawn()` to drive frame-by-frame slide animations with `tokio::time::sleep()`. This meant every UI call inside these async blocks executed on a **tokio worker thread** — not the main thread.

The direct trigger was a raw Objective-C call `msg_send![app, hide:nil]` (`[NSApplication hide:]`) in `animate_window_hide`, but all other window operations (`set_position`, `set_size`, `show`, `hide`, `set_focus`) were also called from the wrong thread.

### Why It Didn't Always Crash

Some Tauri wrapper methods (like `set_position`, `show`) may internally dispatch to the main thread in certain cases, so the crash was intermittent. The raw `msg_send!` call had no such safety net and crashed consistently once macOS's internal assertion was hit.

### Affected Functions

| Function | File | UI Operations on Wrong Thread |
|---|---|---|
| `animate_window_show` | `src-tauri/src/lib.rs` | `set_size`, `set_position` (x3), `show`, `set_focus` |
| `animate_window_hide` | `src-tauri/src/lib.rs` | `set_position` (x30 loop), `hide`, `[NSApplication hide:]` |

### Locations Confirmed Safe (no changes needed)

| Location | Reason |
|---|---|
| `set_window_level()` | Called from `setup()` which runs on the main thread |
| `apply_window_effect()` | No-op on macOS (solid background via CSS) |
| Tauri `#[tauri::command]` handlers | Tauri executes commands on the main thread |
| `clipboard.rs` async spawns | No UI operations, only clipboard monitoring and keyboard simulation |

## Fix

Wrapped every window UI operation inside `run_on_main_thread()` while keeping the async sleep-based animation timing on the tokio thread.

### Pattern

```
tokio worker thread              main thread
─────────────────────            ───────────
compute position
  │
  ├─ run_on_main_thread(|| {
  │     set_position(...)  ───────> executes here
  │  })
  │
  sleep(2ms)
  │
  ├─ run_on_main_thread(|| {
  │     set_position(...)  ───────> executes here
  │  })
  │
  sleep(2ms)
  ...
```

### Changes in `animate_window_show`

**Before** — UI calls directly on tokio thread:
```rust
tauri::async_runtime::spawn(async move {
    let _ = window.set_size(...);
    let _ = window.set_position(...);
    let _ = window.show();
    let _ = window.set_focus();

    for i in 1..=steps {
        let _ = window.set_position(...);
        tokio::time::sleep(step_duration).await;
    }
    let _ = window.set_position(...);  // final
});
```

**After** — UI calls dispatched to main thread:
```rust
tauri::async_runtime::spawn(async move {
    {
        let win = window.clone();
        let _ = window.run_on_main_thread(move || {
            let _ = win.set_size(...);
            let _ = win.set_position(...);
            let _ = win.show();
            let _ = win.set_focus();
        });
    }

    for i in 1..=steps {
        let win = window.clone();
        let _ = window.run_on_main_thread(move || {
            let _ = win.set_position(...);
        });
        tokio::time::sleep(step_duration).await;
    }

    {
        let win = window.clone();
        let _ = window.run_on_main_thread(move || {
            let _ = win.set_position(...);  // final
        });
    }
});
```

### Changes in `animate_window_hide`

Same pattern for `set_position` in the animation loop. Additionally, `window.hide()` and the raw `[NSApplication hide:]` call were merged into a single `run_on_main_thread` block:

```rust
{
    let win = window.clone();
    let _ = window.run_on_main_thread(move || {
        let _ = win.hide();

        #[cfg(target_os = "macos")]
        {
            use cocoa::appkit::NSApplication;
            use cocoa::base::nil;
            use objc::{msg_send, sel, sel_impl};
            unsafe {
                let app = NSApplication::sharedApplication(nil);
                let _: () = msg_send![app, hide:nil];
            }
        }
    });
}
```

### Note on `on_done` Callback Timing

`run_on_main_thread` is non-blocking — it queues the closure and returns immediately. The `on_done` callback (used for auto-paste) fires right after `hide()` is *dispatched*, not after it *completes*. This is safe because the callback already includes a 300ms sleep before sending paste input, giving the main thread more than enough time to execute the hide.

## Key Takeaway

When using `tauri::async_runtime::spawn` (tokio) on macOS, **never** call AppKit/window UI operations directly. Always use `window.run_on_main_thread()` to dispatch them to the main thread. This applies to both Tauri wrapper methods and raw `objc` / `cocoa` calls.
