#![allow(non_snake_case)]
use tauri::{
    image::Image,
    menu::{Menu, MenuItem},
    tray::TrayIconBuilder,
    Manager,
};
use tauri_plugin_autostart::MacosLauncher;
use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};
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

use models::get_runtime;
use database::Database;

pub fn run_app() {
    let builder = tauri::Builder::default();

    #[cfg(target_os = "macos")]
    {
        builder = builder.plugin(tauri_plugin_log::Builder::default().build());
    }

    let data_dir = get_data_dir();
    fs::create_dir_all(&data_dir).ok();
    let db_path = data_dir.join("paste_paw.db");
    let db_path_str = db_path.to_str().unwrap_or("paste_paw.db").to_string();

    let rt = get_runtime().expect("Failed to get global tokio runtime");

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
        .level(log::LevelFilter::Debug);

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

    builder
        .plugin(log_builder.build())
        .plugin(tauri_plugin_clipboard::init())
        .plugin(tauri_plugin_autostart::init(MacosLauncher::LaunchAgent, Some(vec!["--flag1", "--flag2"])))
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_aptabase::Builder::new("A-US-2920723583").build())
        .manage(db_arc.clone())
        .on_window_event(|window, event| {
            match event {
                tauri::WindowEvent::Focused(focused) => {
                    if !focused {
                        let label = window.label();
                        // Only auto-hide the main window
                        if label == "main" {
                            if window.app_handle().get_webview_window("settings").is_some() {
                                // Settings window is open, keep main window visible
                                return;
                            }

                            // Debounce: Ignore blur events immediately after showing (500ms grace period)
                            let last_show = LAST_SHOW_TIME.load(Ordering::SeqCst);
                            let now = chrono::Local::now().timestamp_millis();
                            if now - last_show < 500 {
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

                                 // Check if cursor is on a different monitor
                                 let current_monitor = win.current_monitor().ok().flatten();
                                 let cursor_monitor = get_monitor_at_cursor(&win);

                                 let mut moved_screens = false;
                                 if let (Some(cm), Some(crm)) = (&current_monitor, &cursor_monitor) {
                                     // Compare monitor names or positions to see if they are different
                                     // Position is usually unique enough
                                     if cm.position().x != crm.position().x || cm.position().y != crm.position().y {
                                         moved_screens = true;
                                     }
                                 }

                                 if moved_screens {
                                     // User clicked on another screen, move window there immediately
                                     position_window_at_bottom(&win);
                                     let _ = win.show();
                                     let _ = win.set_focus();
                                 } else {
                                     // Normal blur handling (hide)
                                     if win.is_visible().unwrap_or(false) {
                                         let win_clone = win.clone();
                                         std::thread::spawn(move || {
                                             crate::animate_window_hide(&win_clone, None);
                                         });
                                     }
                                 }
                            }
                        }
                    }
                }
                _ => {}
            }
        })
        .setup(move |app| {
            log::info!("PastePaw starting...");
            log::info!("Database path: {}", db_path_str);
            if let Ok(log_dir) = app.path().app_log_dir() {
                log::info!("Log directory: {:?}", log_dir);
            }
            let handle = app.handle().clone();
            let db_for_clipboard = db_arc.clone();

            let quit_i = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let show_i = MenuItem::with_id(app, "show", "Show PastePaw", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show_i, &quit_i])?;

            let icon_data = include_bytes!("../icons/tray.png");
            let icon = Image::from_bytes(icon_data).map_err(|e| {
                log::info!("Failed to load icon: {:?}", e);
                e
            })?;

            let _tray = TrayIconBuilder::new()
                .icon(icon)
                .menu(&menu)
                .tooltip("PastePaw")
                .on_menu_event(move |app, event| {
                    if event.id.as_ref() == "quit" {
                        app.exit(0);
                    } else if event.id.as_ref() == "show" {
                        if let Some(win) = app.get_webview_window("main") {
                            position_window_at_bottom(&win);
                            let _ = win.show();
                            let _ = win.set_focus();
                        }
                    }
                })
                .on_tray_icon_event(|tray, event| {
                    if let tauri::tray::TrayIconEvent::Click { button: tauri::tray::MouseButton::Left, .. } = event {
                        if let Some(win) = tray.app_handle().get_webview_window("main") {
                            position_window_at_bottom(&win);
                            let _ = win.show();
                            let _ = win.set_focus();
                        }
                    }
                })
                .build(app)?;

            let app_handle = handle.clone();
            let win = app_handle.get_webview_window("main").unwrap();

            #[cfg(target_os = "windows")]
            {
                // if apply_blur(&win, Some((16, 16, 16, 125))).is_err() {
                //     if apply_mica(&win, None).is_err() {
                //         let _ = apply_acrylic(&win, None);
                //     }
                // }
                //let _ = apply_blur(&win, Some((16, 16, 16, 125)));
                // let _ = apply_mica(&win, Some(true));

                //let _ = apply_acrylic(&win, Some((16, 16, 16, 125)));
            }

            #[cfg(target_os = "macos")]
            let _ = apply_vibrancy(&win, NSVisualEffectMaterial::WindowBackground, None, None);

            let _ = app_handle.plugin(tauri_plugin_global_shortcut::Builder::new().build())?;

            // Load saved hotkey from database or use default
            let db_for_hotkey = db_for_clipboard.clone();
            let saved_hotkey = get_runtime().unwrap().block_on(async {
                db_for_hotkey.get_setting("hotkey").await.ok().flatten()
            }).unwrap_or_else(|| "Ctrl+Shift+V".to_string());

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
                            let _ = win_clone.show();
                            let _ = win_clone.set_focus();
                        }
                    }
                });
            } else {
                log::error!("Failed to parse hotkey: {}", saved_hotkey);
            }

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
            commands::ai_process_clip
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

pub fn position_window_at_bottom(window: &tauri::WebviewWindow) {
    animate_window_show(window);
}

pub fn animate_window_show(window: &tauri::WebviewWindow) {
    // Atomically check if false and set to true. If already true, return.
    if IS_ANIMATING.compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst).is_err() {
        return;
    }

    LAST_SHOW_TIME.store(chrono::Local::now().timestamp_millis(), Ordering::SeqCst);

    let window = window.clone();
    std::thread::spawn(move || {
        let monitor = get_monitor_at_cursor(&window);

        if let Some(monitor) = monitor {
            let scale_factor = monitor.scale_factor();

            let _screen_size = monitor.size();
            let _monitor_pos = monitor.position();

            log::debug!("Monitor size: {:?}, Monitor position: {:?}, Scale factor: {:?}", _screen_size, _monitor_pos, scale_factor);

            let work_area = monitor.work_area();
            let window_height_px = (constants::WINDOW_HEIGHT * scale_factor) as u32;
            let window_margin_px = (constants::WINDOW_MARGIN * scale_factor) as i32;

            let _ = window.set_size(tauri::Size::Physical(tauri::PhysicalSize {
                width: work_area.size.width - (window_margin_px as u32 * 2),
                height: window_height_px,
            }));

            let target_y = work_area.position.y + (work_area.size.height as i32) - (window_height_px as i32) - window_margin_px;
            let start_y = work_area.position.y + (work_area.size.height as i32); // Just off screen

            // Initial setup
            let _ = window.set_position(tauri::Position::Physical(tauri::PhysicalPosition {
                x: work_area.position.x + window_margin_px,
                y: start_y,
            }));

            // Ensure window is fully opaque (if any previous transparency was applied)
            // No set_opacity here to avoid feature dependency for now.

            let _ = window.show();
            let _ = window.set_focus();

            // Animation loop
            let steps = 15;
            let duration = std::time::Duration::from_millis(10);
            let dy = (target_y - start_y) as f64 / steps as f64;

            for i in 1..=steps {
                let current_y = start_y as f64 + dy * i as f64;
                let _ = window.set_position(tauri::Position::Physical(tauri::PhysicalPosition {
                    x: work_area.position.x + window_margin_px,
                    y: current_y as i32,
                }));
                std::thread::sleep(duration);
            }

            // Ensure final position is exact
            let _ = window.set_position(tauri::Position::Physical(tauri::PhysicalPosition {
                x: work_area.position.x + window_margin_px,
                y: target_y,
            }));
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
    std::thread::spawn(move || {
        if let Some(monitor) = window.current_monitor().ok().flatten() {
            let scale_factor = monitor.scale_factor();
            let work_area = monitor.work_area();

            let window_height_px = (constants::WINDOW_HEIGHT * scale_factor) as u32;
            let window_margin_px = (constants::WINDOW_MARGIN * scale_factor) as i32;

            let start_y = work_area.position.y + (work_area.size.height as i32) - (window_height_px as i32) - window_margin_px;
            let target_y = work_area.position.y + (work_area.size.height as i32); // Off screen (starts at bottom of work area)

            // Fix Z-Order: Dynamic Switch & Fade Out
            #[cfg(target_os = "windows")]
            {
                use windows::Win32::UI::WindowsAndMessaging::{SetWindowPos, FindWindowW, GetWindowRect, SWP_NOMOVE, SWP_NOSIZE, SWP_NOACTIVATE};
                use windows::Win32::Foundation::{HWND, RECT};
                use windows::core::PCWSTR;

                // 1. Find the Taskbar
                let class_name: Vec<u16> = "Shell_TrayWnd".encode_utf16().chain(std::iter::once(0)).collect();
                let taskbar_hwnd = unsafe { FindWindowW(PCWSTR(class_name.as_ptr()), PCWSTR::null()) }.unwrap_or(HWND(std::ptr::null_mut()));

                // 2. Get Taskbar Position (Top Y)
                let mut taskbar_top_y = 0;
                if !taskbar_hwnd.0.is_null() {
                    let mut rect = RECT::default();
                    if unsafe { GetWindowRect(taskbar_hwnd, &mut rect).is_ok() } {
                        taskbar_top_y = rect.top;
                    }
                }

                // 3. Initially Ensure Topmost
                if let Ok(handle) = window.hwnd() {
                     let hwnd = HWND(handle.0 as _);
                     let hwnd_topmost = HWND(-1 as _); // HWND_TOPMOST
                     unsafe {
                        let _ = SetWindowPos(hwnd, Some(hwnd_topmost), 0, 0, 0, 0, SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE);
                     }
                }

                let steps = 15;
                let duration = std::time::Duration::from_millis(10);
                let dy = (target_y - start_y) as f64 / steps as f64;

                let mut z_order_switched = false;

                for i in 1..=steps {
                    let current_y = start_y as f64 + dy * i as f64;
                    let _ = window.set_position(tauri::Position::Physical(tauri::PhysicalPosition {
                        x: work_area.position.x + window_margin_px,
                        y: current_y as i32,
                    }));

                    // Animation Loop for Hide (Slide only for now)

                    // Dynamic Z-Order Switch: When we hit the taskbar, drop BEHIND it
                    if !z_order_switched && taskbar_top_y > 0 && current_y as i32 >= taskbar_top_y {
                         if let Ok(handle) = window.hwnd() {
                             let hwnd = HWND(handle.0 as _);
                             if !taskbar_hwnd.0.is_null() {
                                 unsafe {
                                    let _ = SetWindowPos(hwnd, Some(taskbar_hwnd), 0, 0, 0, 0, SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE);
                                 }
                                 z_order_switched = true;
                             }
                        }
                    }
                    std::thread::sleep(duration);
                }
            }

            #[cfg(not(target_os = "windows"))]
            {
                let steps = 15;
                let duration = std::time::Duration::from_millis(10);
                let dy = (target_y - start_y) as f64 / steps as f64;

                for i in 1..=steps {
                    let current_y = start_y as f64 + dy * i as f64;
                    let _ = window.set_position(tauri::Position::Physical(tauri::PhysicalPosition {
                        x: work_area.position.x + window_margin_px,
                        y: current_y as i32,
                    }));
                    std::thread::sleep(duration);
                }
            }

            let _ = window.hide();
            
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
    #[cfg(not(target_os = "windows"))]
    {
        window.current_monitor().ok().flatten()
    }
}
