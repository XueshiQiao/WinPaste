#![allow(non_snake_case)]
#![allow(unexpected_cfgs)] // objc crate macros check cfg(feature = "cargo-clippy") internally
#![allow(deprecated)] // cocoa crate deprecated its API in favor of objc2; suppress until migration
use tauri::{
    image::Image,
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::TrayIconBuilder,
    Manager,
};
#[cfg(not(feature = "app-store"))]
use tauri_plugin_autostart::MacosLauncher;
use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};
use tauri_plugin_aptabase::EventTracker;
use std::fs;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicI64, Ordering};


static IS_ANIMATING: AtomicBool = AtomicBool::new(false);
static LAST_SHOW_TIME: AtomicI64 = AtomicI64::new(0);

mod clipboard;
mod database;
mod models;
mod commands;
mod constants;
mod ai;
#[cfg(target_os = "macos")]
mod source_app_macos;

use models::get_runtime;
use database::Database;

pub fn run_app() {
    let data_dir = get_data_dir();
    fs::create_dir_all(&data_dir).ok();
    let db_path = data_dir.join("paste_paw.db");
    let db_path_str = db_path.to_str().unwrap_or("paste_paw.db").to_string();

    let rt = get_runtime().expect("Failed to get global tokio runtime");
    let _guard = rt.enter();

    let db = rt.block_on(async {
        Database::new(&db_path_str).await
    });

    rt.block_on(async {
        db.migrate().await.ok();
    });

    let db_arc = Arc::new(db);

    let mut log_builder = tauri_plugin_log::Builder::default()
        .format(|out, message, record| {
            out.finish(format_args!(
                "[{}][{}][{}] {}",
                chrono::Local::now().format("%Y-%m-%d %H:%M:%S%.3f"),
                record.target(),
                record.level(),
                message
            ))
        })
        .level(log::LevelFilter::Debug)
        .level_for("sqlx", log::LevelFilter::Warn);

    #[cfg(debug_assertions)]
    {
        log_builder = log_builder.targets([
            tauri_plugin_log::Target::new(tauri_plugin_log::TargetKind::Stdout),
            tauri_plugin_log::Target::new(tauri_plugin_log::TargetKind::Webview),
        ]);
    }

    #[cfg(not(debug_assertions))]
    {
        log_builder = log_builder.targets([
            tauri_plugin_log::Target::new(tauri_plugin_log::TargetKind::LogDir { file_name: None }),
            tauri_plugin_log::Target::new(tauri_plugin_log::TargetKind::Webview),
        ]);
    }

    #[allow(unused_mut)]
    let mut builder = tauri::Builder::default();

    #[cfg(not(feature = "app-store"))]
    {
        builder = builder
            .plugin(tauri_plugin_autostart::init(MacosLauncher::LaunchAgent, Some(vec!["--flag1", "--flag2"])))
            .plugin(tauri_plugin_updater::Builder::new().build());
    }

    builder
        .plugin(log_builder.build())
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            log::info!("Second instance detected. Sending notification and exiting.");
            use tauri_plugin_notification::NotificationExt;
            if let Err(e) = app.notification()
                .builder()
                .title("PastePaw")
                .body("PastePaw is already running")
                .show() {
                log::error!("Failed to send notification: {:?}", e);
            }
        }))
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_clipboard_x::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_aptabase::Builder::new("A-US-2920723583").build())
        .manage(db_arc.clone())
        .on_window_event(|window, event| {
            #[cfg(target_os = "macos")]
            {
                if let Ok(handle) = window.ns_window() {
                    crate::setup_macos_window(handle as cocoa::base::id);
                }
            }

            match event {
                tauri::WindowEvent::ThemeChanged(theme) => {
                    log::info!("THEME:System theme changed to: {:?}, win.theme(): {:?}", theme, window.theme());
                    let label = window.label().to_string();
                    let app_handle = window.app_handle().clone();
                    let db = window.state::<Arc<Database>>().inner().clone();
                    let theme_ = theme.clone();

                    tauri::async_runtime::spawn(async move {
                        let current_theme = db.get_setting("theme").await.ok().flatten().unwrap_or_else(|| "system".to_string());
                        let mica_effect = db.get_setting("mica_effect").await.ok().flatten().unwrap_or_else(|| "clear".to_string());

                        log::info!("THEME:Re-applying window effect due to theme change. Current theme setting: {:?}, system theme: {:?}, mica_effect setting: {:?}", current_theme, theme_, mica_effect);
                        // If app is set to follow system, we re-apply based on the NEW system theme
                        if current_theme == "system" {
                            if let Some(webview_win) = app_handle.get_webview_window(&label) {
                                crate::apply_window_effect(&webview_win, &mica_effect, &theme_);
                            }
                        }
                    });
                }
                tauri::WindowEvent::Focused(focused) => {
                    if !focused {
                        let label = window.label();
                        // Only auto-hide the main window
                        if label == "main" {
                            if window.app_handle().get_webview_window("settings").is_some() {
                                // Settings window is open, keep main window visible
                                return;
                            }

                            // Debounce: Ignore blur events immediately after showing (150ms grace period)
                            let last_show = LAST_SHOW_TIME.load(Ordering::SeqCst);
                            let now = chrono::Local::now().timestamp_millis();
                            if now - last_show < 150 {
                                return;
                            }

                        if let Some(win) = window.app_handle().get_webview_window(label) {
                                 // Safety checks:
                                 // 1. If we are already animating (e.g. hiding via hotkey), don't interfere.
                                 if IS_ANIMATING.load(Ordering::SeqCst) {
                                     return;
                                 }
                                 // 2. If the window is not visible (e.g. just hidden programmatically), don't try to move/show it.
                                 if !win.is_visible().unwrap_or(false) {
                                     return;
                                 }

                                 // Normal blur handling (hide)
                                 crate::animate_window_hide(&win, None);
                            }
                        }
                    }
                }
                _ => {}
            }
        })
        .setup(move |app| {
            log::info!("PastePaw starting...");
            let _ = app.track_event("startup", None);
            log::info!("Database path: {}", db_path_str);
            if let Ok(log_dir) = app.path().app_log_dir() {
                log::info!("Log directory: {:?}", log_dir);
            }
            let handle = app.handle().clone();
            let db_for_clipboard = db_arc.clone();

            let version = env!("CARGO_PKG_VERSION");
            let title = format!("v{}", version);
            let title_i = MenuItem::with_id(app, "title", &title, false, None::<&str>)?;
            let quit_i = MenuItem::with_id(app, "quit", "Quit PastePaw", true, None::<&str>)?;
            let show_i = MenuItem::with_id(app, "show", "Show", true, None::<&str>)?;
            let separator_i = PredefinedMenuItem::separator(app)?;
            let menu = Menu::with_items(app, &[&title_i, &show_i, &separator_i, &quit_i])?;

            let icon_data = include_bytes!("../icons/tray.png");
            let icon = Image::from_bytes(icon_data).map_err(|e| {
                log::info!("Failed to load icon: {:?}", e);
                e
            })?;

            let tray_builder = TrayIconBuilder::new()
                .icon(icon)
                .menu(&menu);

            #[cfg(target_os = "macos")]
            let tray_builder = tray_builder.icon_as_template(true);

            let _tray = tray_builder
                .tooltip("PastePaw")
                .on_menu_event(move |app, event| {
                    if event.id.as_ref() == "quit" {
                        app.exit(0);
                    } else if event.id.as_ref() == "show" {
                        if let Some(win) = app.get_webview_window("main") {
                            position_window_at_bottom(&win);
                        }
                    }
                })
                .on_tray_icon_event(|tray, event| {
                    if let tauri::tray::TrayIconEvent::Click { button: tauri::tray::MouseButton::Left, .. } = event {
                        if let Some(win) = tray.app_handle().get_webview_window("main") {
                            position_window_at_bottom(&win);
                        }
                    }
                })
                .build(app)?;

            let app_handle = handle.clone();
            let win = app_handle.get_webview_window("main").unwrap();

            #[cfg(target_os = "windows")]
            {
                let db_for_mica = db_for_clipboard.clone();
                let (mica_effect, theme) = get_runtime().unwrap().block_on(async {
                    let m = db_for_mica.get_setting("mica_effect").await.ok().flatten().unwrap_or_else(|| "clear".to_string());
                    let t = db_for_mica.get_setting("theme").await.ok().flatten().unwrap_or_else(|| "system".to_string());
                    (m, t)
                });

                // get current system theme
                let current_theme = if theme == "light" {
                    tauri::Theme::Light
                } else if theme == "dark" {
                    tauri::Theme::Dark
                } else {
                    win.theme().unwrap_or_else(|err| {
                        log::error!("THEME:Failed to get system theme: {:?}, defaulting to Light", err);
                        tauri::Theme::Light
                    })
                };

                log::info!("THEME:Applying window effect: {} with theme: {:?} (setting:{:?}", mica_effect, current_theme, theme);

                crate::apply_window_effect(&win, &mica_effect, &current_theme);
            }

            #[cfg(target_os = "macos")]
            {
                // Set activation policy to Accessory to hide dock icon and menu bar
                app.set_activation_policy(tauri::ActivationPolicy::Accessory);
                // Apply custom native window styling
                if let Ok(handle) = win.ns_window() {
                    crate::setup_macos_window(handle as cocoa::base::id);
                }
                // Set window level to NSStatusWindowLevel (25) to be above the Dock
                crate::set_window_level(&win, 25);
            }

            // Load saved hotkey from database or use default
            let db_for_hotkey = db_for_clipboard.clone();
            let saved_hotkey = get_runtime().unwrap().block_on(async {
                db_for_hotkey.get_setting("hotkey").await.ok().flatten()
            }).unwrap_or_else(|| {
                if cfg!(target_os = "macos") {
                    "Cmd+Shift+V".to_string()
                } else {
                    "Ctrl+Shift+V".to_string()
                }
            });

            log::info!("Registering hotkey: {}", saved_hotkey);

            // Parse the hotkey string into a Shortcut
            use std::str::FromStr;
            use tauri_plugin_global_shortcut::Shortcut;

            if let Ok(shortcut) = Shortcut::from_str(&saved_hotkey) {
                let win_clone = win.clone();
                let _ = app_handle.global_shortcut().on_shortcut(shortcut, move |_app, _shortcut, event| {
                    if event.state() == ShortcutState::Pressed {
                        if win_clone.is_visible().unwrap_or(false) && win_clone.is_focused().unwrap_or(false) {
                            crate::animate_window_hide(&win_clone, None);
                        } else {
                            position_window_at_bottom(&win_clone);
                        }
                    }
                });
            } else {
                log::error!("Failed to parse hotkey: {}", saved_hotkey);
            }

            #[cfg(target_os = "macos")]
            source_app_macos::start_frontmost_app_observer();

            let handle_for_clip = app_handle.clone();
            let db_for_clip = db_for_clipboard.clone();
            clipboard::init(&handle_for_clip, db_for_clip);

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::ping,
            commands::get_clips,
            commands::get_clip,
            commands::paste_clip,
            commands::delete_clip,
            commands::move_to_folder,
            commands::create_folder,
            commands::rename_folder,
            commands::delete_folder,
            commands::search_clips,
            commands::get_folders,
            commands::get_settings,
            commands::save_settings,
            commands::hide_window,
            commands::get_clipboard_history_size,
            commands::clear_clipboard_history,
            commands::clear_all_clips,
            commands::remove_duplicate_clips,
            commands::register_global_shortcut,
            commands::show_window,
            commands::add_ignored_app,
            commands::remove_ignored_app,
            commands::get_ignored_apps,
            commands::pick_file,
            commands::get_layout_config,
            commands::test_log,
            commands::ai_process_clip,
            commands::focus_window,
            commands::check_accessibility_permissions,
            commands::request_accessibility_permissions
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

pub fn position_window_at_bottom(window: &tauri::WebviewWindow) {
    animate_window_show(window);
}

fn ease_linear(x: f64) -> f64 {
    x
}

pub fn animate_window_show(window: &tauri::WebviewWindow) {
    // Atomically check if false and set to true. If already true, return.
    if IS_ANIMATING.compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst).is_err() {
        return;
    }

    LAST_SHOW_TIME.store(chrono::Local::now().timestamp_millis(), Ordering::SeqCst);

    let window = window.clone();
    tauri::async_runtime::spawn(async move {
        if let Some(monitor) = get_monitor_at_cursor(&window) {
            let scale_factor = monitor.scale_factor();
            let screen_size = monitor.size();
            let monitor_pos = monitor.position();
            let work_area = monitor.work_area();

            let window_height_px = (constants::WINDOW_HEIGHT * scale_factor) as u32;
            let window_margin_px = (constants::WINDOW_MARGIN * scale_factor) as i32;

            let screen_bottom = monitor_pos.y + screen_size.height as i32;

            #[cfg(target_os = "macos")]
            let (target_y, start_y) = (
                screen_bottom - (window_height_px as i32) - window_margin_px,
                screen_bottom,
            );
            #[cfg(not(target_os = "macos"))]
            let (target_y, start_y) = {
                let work_area_bottom = work_area.position.y + work_area.size.height as i32;
                (
                    work_area_bottom - (window_height_px as i32) - window_margin_px,
                    work_area_bottom,
                )
            };

            let x = work_area.position.x + window_margin_px;
            let width = work_area.size.width - (window_margin_px as u32 * 2);

            // Set initial size, position, and show window — all on main thread
            {
                let win = window.clone();
                let _ = window.run_on_main_thread(move || {
                    let _ = win.set_size(tauri::Size::Physical(tauri::PhysicalSize {
                        width,
                        height: window_height_px,
                    }));
                    let _ = win.set_position(tauri::Position::Physical(tauri::PhysicalPosition {
                        x,
                        y: start_y,
                    }));
                    let _ = win.show();
                    let _ = win.set_focus();
                });
            }

            // Animation loop: ~=60ms total (30 steps * 2ms)
            let steps = 30;
            let step_duration = std::time::Duration::from_millis(2);
            let total_dist = (target_y - start_y) as f64;

            for i in 1..=steps {
                let progress = i as f64 / steps as f64;
                let eased_progress = ease_linear(progress);
                let current_y = start_y as f64 + total_dist * eased_progress;

                let win = window.clone();
                let _ = window.run_on_main_thread(move || {
                    let _ = win.set_position(tauri::Position::Physical(tauri::PhysicalPosition {
                        x,
                        y: current_y as i32,
                    }));
                });
                tokio::time::sleep(step_duration).await;
            }

            // Ensure final position is exact
            {
                let win = window.clone();
                let _ = window.run_on_main_thread(move || {
                    let _ = win.set_position(tauri::Position::Physical(tauri::PhysicalPosition {
                        x,
                        y: target_y,
                    }));
                });
            }
        }
        IS_ANIMATING.store(false, Ordering::SeqCst);
    });
}

pub fn animate_window_hide(window: &tauri::WebviewWindow, on_done: Option<Box<dyn FnOnce() + Send>>) {
    // Atomically check if false and set to true. If already true, return.
    if IS_ANIMATING.compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst).is_err() {
        return;
    }

    let window = window.clone();
    tauri::async_runtime::spawn(async move {
        if let Some(monitor) = window.current_monitor().ok().flatten() {
            let scale_factor = monitor.scale_factor();
            let work_area = monitor.work_area();
            let screen_size = monitor.size();
            let monitor_pos = monitor.position();

            let window_height_px = (constants::WINDOW_HEIGHT * scale_factor) as u32;
            let window_margin_px = (constants::WINDOW_MARGIN * scale_factor) as i32;

            let screen_bottom = monitor_pos.y + screen_size.height as i32;

            #[cfg(target_os = "macos")]
            let (start_y, target_y) = (
                screen_bottom - (window_height_px as i32) - window_margin_px,
                screen_bottom,
            );
            #[cfg(not(target_os = "macos"))]
            let (start_y, target_y) = {
                let work_area_bottom = work_area.position.y + work_area.size.height as i32;
                let dock_gap = screen_bottom - work_area_bottom;
                (
                    work_area_bottom - (window_height_px as i32) - window_margin_px,
                    work_area_bottom + dock_gap,
                )
            };

            // Windows-specific setup
            #[cfg(target_os = "windows")]
            let mut taskbar_top_y = 0;
            #[cfg(target_os = "windows")]
            let taskbar_hwnd = {
                use windows::Win32::UI::WindowsAndMessaging::{FindWindowW, GetWindowRect};
                use windows::Win32::Foundation::{HWND, RECT};
                use windows::core::PCWSTR;

                let class_name: Vec<u16> = "Shell_TrayWnd".encode_utf16().chain(std::iter::once(0)).collect();
                let hwnd = unsafe { FindWindowW(PCWSTR(class_name.as_ptr()), PCWSTR::null()) }.unwrap_or(HWND(std::ptr::null_mut()));

                if !hwnd.0.is_null() {
                    let mut rect = RECT::default();
                    if unsafe { GetWindowRect(hwnd, &mut rect).is_ok() } {
                        taskbar_top_y = rect.top;
                    }
                }
                hwnd
            };

            #[cfg(target_os = "windows")]
            if let Ok(handle) = window.hwnd() {
                 use windows::Win32::UI::WindowsAndMessaging::{SetWindowPos, SWP_NOMOVE, SWP_NOSIZE, SWP_NOACTIVATE};
                 use windows::Win32::Foundation::HWND;
                 let hwnd = HWND(handle.0 as _);
                 let hwnd_topmost = HWND(-1 as _);
                 unsafe {
                    let _ = SetWindowPos(hwnd, Some(hwnd_topmost), 0, 0, 0, 0, SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE);
                 }
            }

            // Hide animation matched to show duration: ~60ms (30 steps * 2ms)
            let steps = 30;
            let step_duration = std::time::Duration::from_millis(2);
            let total_dist = (target_y - start_y) as f64;
            #[cfg(target_os = "windows")]
            let mut z_order_switched = false;

            for i in 1..=steps {
                let progress = i as f64 / steps as f64;
                let eased_progress = ease_linear(progress);
                let current_y = start_y as f64 + total_dist * eased_progress;

                let win = window.clone();
                let x = work_area.position.x + window_margin_px;
                let _ = window.run_on_main_thread(move || {
                    let _ = win.set_position(tauri::Position::Physical(tauri::PhysicalPosition {
                        x,
                        y: current_y as i32,
                    }));
                });

                #[cfg(target_os = "windows")]
                if !z_order_switched && taskbar_top_y > 0 && current_y as i32 >= taskbar_top_y {
                     if let Ok(handle) = window.hwnd() {
                         use windows::Win32::UI::WindowsAndMessaging::{SetWindowPos, SWP_NOMOVE, SWP_NOSIZE, SWP_NOACTIVATE};
                         use windows::Win32::Foundation::HWND;
                         let hwnd = HWND(handle.0 as _);
                         if !taskbar_hwnd.0.is_null() {
                             unsafe {
                                let _ = SetWindowPos(hwnd, Some(taskbar_hwnd), 0, 0, 0, 0, SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE);
                             }
                             z_order_switched = true;
                         }
                    }
                }

                tokio::time::sleep(step_duration).await;
            }

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

            if let Some(callback) = on_done {
                callback();
            }
        }
        IS_ANIMATING.store(false, Ordering::SeqCst);
    });
}


fn get_data_dir() -> std::path::PathBuf {
    let current_dir = std::env::current_dir().unwrap_or(std::path::PathBuf::from("."));
    match dirs::data_dir() {
        Some(path) => path.join("PastePaw"),
        None => current_dir.join("PastePaw"),
    }
}

pub fn get_monitor_at_cursor(window: &tauri::WebviewWindow) -> Option<tauri::Monitor> {
    #[cfg(target_os = "windows")]
    {
        use windows::Win32::UI::WindowsAndMessaging::GetCursorPos;
        use windows::Win32::Foundation::POINT;
        let mut point = POINT { x: 0, y: 0 };
        let mut found = None;
        if unsafe { GetCursorPos(&mut point).is_ok() } {
            if let Ok(monitors) = window.available_monitors() {
                for m in monitors {
                    let pos = m.position();
                    let size = m.size();
                    if point.x >= pos.x && point.x < pos.x + size.width as i32 &&
                       point.y >= pos.y && point.y < pos.y + size.height as i32 {
                        found = Some(m);
                        break;
                    }
                }
            }
        }
        found.or_else(|| window.current_monitor().ok().flatten())
    }
    #[cfg(target_os = "macos")]
    {
        use core_graphics::event_source::{CGEventSource, CGEventSourceStateID};
        use core_graphics::event::CGEvent;

        let source = CGEventSource::new(CGEventSourceStateID::HIDSystemState).ok();
        let cursor_pos = source.and_then(|src| CGEvent::new(src).ok()).map(|e| e.location());

        if let Some(point) = cursor_pos {
            if let Ok(monitors) = window.available_monitors() {
                for m in monitors {
                    let scale = m.scale_factor();
                    let pos = m.position();
                    let size = m.size();
                    
                    let logical_x = pos.x as f64 / scale;
                    let logical_y = pos.y as f64 / scale;
                    let logical_w = size.width as f64 / scale;
                    let logical_h = size.height as f64 / scale;

                    if point.x >= logical_x && point.x < logical_x + logical_w &&
                       point.y >= logical_y && point.y < logical_y + logical_h {
                        return Some(m);
                    }
                }
            }
        }
        window.current_monitor().ok().flatten()
            .or_else(|| window.available_monitors().ok().and_then(|m| m.into_iter().next()))
    }
    #[cfg(all(not(target_os = "windows"), not(target_os = "macos")))]
    {
        window.current_monitor().ok().flatten()
            .or_else(|| window.available_monitors().ok().and_then(|m| m.into_iter().next()))
    }
}

pub fn apply_window_effect(window: &tauri::WebviewWindow, effect: &str, theme: &tauri::Theme) {
    #[cfg(target_os = "windows")]
    {
        use window_vibrancy::{clear_mica, apply_mica, apply_tabbed};

        match effect {
            "clear" => {
                let _ = clear_mica(window);
                log::info!("THEME:Mica effect cleared");
            },
            "mica" | "dark" => {
                let _ = clear_mica(window);
                let _ = apply_mica(window, Some(matches!(theme, tauri::Theme::Dark)));
                log::info!("THEME:Applied Mica effect (Theme: {})", theme);
            },
            "mica_alt" | "auto" | _ => {
                let _ = clear_mica(window);
                let _ = apply_tabbed(window, Some(matches!(theme, tauri::Theme::Dark)));
                log::info!("THEME:Applied Tabbed effect (Theme: {})", theme);
            }
        }
    }

    #[cfg(target_os = "macos")]
    {
        let _ = effect;
        let _ = theme;
        let _ = window;
        // No vibrancy on macOS — solid background via CSS
    }
}

#[cfg(target_os = "macos")]
fn set_window_level(window: &tauri::WebviewWindow, level: i64) {
    use cocoa::appkit::NSWindow;
    use cocoa::base::id;
    
    if let Ok(handle) = window.ns_window() {
        unsafe {
            let ns_window: id = handle as id;
            ns_window.setLevel_(level);
        }
    }
}

#[cfg(target_os = "macos")]
fn setup_macos_window(ns_window: cocoa::base::id) {
    use cocoa::appkit::{NSWindow, NSWindowStyleMask};
    use cocoa::foundation::NSString;
    use cocoa::base::{id, nil, NO, YES};
    use objc::{msg_send, sel, sel_impl, class};

    unsafe {
        // 1. Set window to non-opaque and clear background to allow rounded corners
        ns_window.setOpaque_(NO);
        ns_window.setBackgroundColor_(msg_send![class!(NSColor), clearColor]);

        // 2. Ensure content fills the window area (fixes padding issues)
        let mut style_mask: NSWindowStyleMask = ns_window.styleMask();
        style_mask.insert(NSWindowStyleMask::NSFullSizeContentViewWindowMask);
        ns_window.setStyleMask_(style_mask);

        // 3. Access the WKWebView and disable its background drawing
        let content_view: id = ns_window.contentView();
        let subviews: id = msg_send![content_view, subviews];
        let count: usize = msg_send![subviews, count];
        
        if count > 0 {
            let webview: id = msg_send![subviews, objectAtIndex:0];
            
            // Fix white background in WKWebView
            let no_num: id = msg_send![class!(NSNumber), numberWithBool:NO];
            let draws_bg_key = NSString::alloc(nil).init_str("drawsBackground");
            let _: () = msg_send![webview, setValue:no_num forKey:draws_bg_key];
        }

        // 4. Hide the titlebar completely
        ns_window.setTitlebarAppearsTransparent_(YES);
        ns_window.setTitleVisibility_(cocoa::appkit::NSWindowTitleVisibility::NSWindowTitleHidden);

        // 5. Disable native shadow to remove the black border artifact
        ns_window.setHasShadow_(NO);
    }
}
