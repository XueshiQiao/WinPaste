use tauri::{
    image::Image,
    menu::{Menu, MenuItem},
    tray::TrayIconBuilder,
    Manager,
};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};
use std::fs;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

static IS_ANIMATING: AtomicBool = AtomicBool::new(false);

mod clipboard;
mod database;
mod models;
mod commands;
mod constants;

use models::get_runtime;
use database::Database;

pub fn run_app() {
// ... (rest of imports)

    let data_dir = get_data_dir();
    fs::create_dir_all(&data_dir).ok();
    let db_path = data_dir.join("winpaste.db");
    let db_path_str = db_path.to_str().unwrap().to_string();

    let rt = get_runtime().expect("Failed to get global tokio runtime");

    let db = rt.block_on(async {
        Database::new(&db_path_str).await
    });

    rt.block_on(async {
        db.migrate().await.ok();
    });

    let db_arc = Arc::new(db);

    tauri::Builder::default()
        .plugin(tauri_plugin_clipboard::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
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

                            if let Some(win) = window.app_handle().get_webview_window(label) {
                                 let win_clone = win.clone();
                                 std::thread::spawn(move || {
                                     crate::animate_window_hide(&win_clone);
                                 });
                            }
                        }
                    }
                }
                _ => {}
            }
        })
        .setup(move |app| {
            let handle = app.handle().clone();
            let db_for_clipboard = db_arc.clone();

            let quit_i = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let show_i = MenuItem::with_id(app, "show", "Show WinPaste", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show_i, &quit_i])?;

            let icon_data = include_bytes!("../icons/tray.png");
            let icon = Image::from_bytes(icon_data).map_err(|e| {
                eprintln!("Failed to load icon: {:?}", e);
                e
            })?;

            let _tray = TrayIconBuilder::new()
                .icon(icon)
                .menu(&menu)
                .tooltip("WinPaste")
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
            let _ = app_handle.plugin(tauri_plugin_global_shortcut::Builder::new().build())?;

            let win_clone = win.clone();
            let _ = app_handle.global_shortcut().on_shortcut("Ctrl+Alt+V", move |_app, _shortcut, event| {
                if event.state() == ShortcutState::Pressed {
                    position_window_at_bottom(&win_clone);
                    let _ = win_clone.show();
                    let _ = win_clone.set_focus();
                }
            });

            let handle_for_clip = app_handle.clone();
            let db_for_clip = db_for_clipboard.clone();
            clipboard::init(&handle_for_clip, db_for_clip);

            // Start listening for clipboard updates via the plugin's default monitor
            // The plugin needs to be initialized first (which it is in builder)
            // But we might need to verify if "start_monitor" is needed?
            // tauri-plugin-clipboard-manager auto-starts monitor on some platforms or requires explicit start.
            // Documentation says "Monitor is started automatically". Good.

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::ping,
            commands::get_clips,
            commands::get_clip,
            commands::paste_clip,
            commands::delete_clip,
            commands::pin_clip,
            commands::unpin_clip,
            commands::move_to_folder,
            commands::create_folder,
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
            commands::get_layout_config
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

    let window = window.clone();
    std::thread::spawn(move || {
        if let Some(monitor) = window.current_monitor().ok().flatten() {
            let scale_factor = monitor.scale_factor();
            let screen_size = monitor.size();
            let monitor_pos = monitor.position();
            let work_area = monitor.work_area();
            let window_height_px = (constants::WINDOW_HEIGHT * scale_factor) as u32;

            let _ = window.set_size(tauri::Size::Physical(tauri::PhysicalSize {
                width: screen_size.width,
                height: window_height_px,
            }));

            let target_y = work_area.position.y + (work_area.size.height as i32) - (window_height_px as i32);
            let start_y = work_area.position.y + (work_area.size.height as i32); // Just off screen

            // Initial setup
            let _ = window.set_position(tauri::Position::Physical(tauri::PhysicalPosition {
                x: monitor_pos.x,
                y: start_y,
            }));

            let _ = window.show();
            let _ = window.set_focus();

            // Animation loop
            let steps = 15;
            let duration = std::time::Duration::from_millis(10);
            let dy = (target_y - start_y) as f64 / steps as f64;

            for i in 1..=steps {
                let current_y = start_y as f64 + dy * i as f64;
                let _ = window.set_position(tauri::Position::Physical(tauri::PhysicalPosition {
                    x: monitor_pos.x,
                    y: current_y as i32,
                }));
                std::thread::sleep(duration);
            }

            // Ensure final position is exact
            let _ = window.set_position(tauri::Position::Physical(tauri::PhysicalPosition {
                x: monitor_pos.x,
                y: target_y,
            }));
        }
        IS_ANIMATING.store(false, Ordering::SeqCst);
    });
}

pub fn animate_window_hide(window: &tauri::WebviewWindow) {
    // Atomically check if false and set to true. If already true, return.
    if IS_ANIMATING.compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst).is_err() {
        return;
    }

    let window = window.clone();
    std::thread::spawn(move || {
        if let Some(monitor) = window.current_monitor().ok().flatten() {
            let scale_factor = monitor.scale_factor();
            let work_area = monitor.work_area();
            let monitor_pos = monitor.position();

            let window_height_px = (constants::WINDOW_HEIGHT * scale_factor) as u32;

            let start_y = work_area.position.y + (work_area.size.height as i32) - (window_height_px as i32);
            let target_y = work_area.position.y + (work_area.size.height as i32); // Off screen

            let steps = 15;
            let duration = std::time::Duration::from_millis(10);
            let dy = (target_y - start_y) as f64 / steps as f64;

            for i in 1..=steps {
                let current_y = start_y as f64 + dy * i as f64;
                let _ = window.set_position(tauri::Position::Physical(tauri::PhysicalPosition {
                    x: monitor_pos.x,
                    y: current_y as i32,
                }));
                std::thread::sleep(duration);
            }

            let _ = window.hide();
        }
        IS_ANIMATING.store(false, Ordering::SeqCst);
    });
}

fn get_data_dir() -> std::path::PathBuf {
    let current_dir = std::env::current_dir().unwrap_or(std::path::PathBuf::from("."));
    match dirs::data_dir() {
        Some(path) => path.join("WinPaste"),
        None => current_dir.join("WinPaste"),
    }
}
