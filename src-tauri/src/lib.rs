use tauri::{
    image::Image,
    menu::{Menu, MenuItem},
    tray::TrayIconBuilder,
    Manager,
};
use tauri_plugin_global_shortcut::GlobalShortcutExt;
use std::fs;
use std::sync::Arc;

mod clipboard;
mod database;
mod models;
mod commands;

use models::get_runtime;
use database::Database;

pub fn run_app() {
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
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .manage(db_arc.clone())
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
                            let _ = win.show();
                            let _ = win.set_focus();
                        }
                    }
                })
                .on_tray_icon_event(|tray, event| {
                    if let tauri::tray::TrayIconEvent::Click { button: tauri::tray::MouseButton::Left, .. } = event {
                        if let Some(win) = tray.app_handle().get_webview_window("main") {
                            let _ = win.show();
                            let _ = win.set_focus();
                        }
                    }
                })
                .build(app)?;

            let app_handle = handle.clone();
            let win = app_handle.get_webview_window("main").unwrap();
            let _ = app_handle.plugin(tauri_plugin_global_shortcut::Builder::new().build())?;
            
            let _ = app_handle.global_shortcut().on_shortcut("Ctrl+Alt+V", move |_, _, _| {
                let _ = win.show();
                let _ = win.set_focus();
            });

            std::thread::spawn(move || {
                clipboard::start_clipboard_monitor(handle, db_for_clipboard);
            });

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
            commands::show_window
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn get_data_dir() -> std::path::PathBuf {
    let current_dir = std::env::current_dir().unwrap_or(std::path::PathBuf::from("."));
    match dirs::data_dir() {
        Some(path) => path.join("WinPaste"),
        None => current_dir.join("WinPaste"),
    }
}
