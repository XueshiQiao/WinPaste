use tauri::{AppHandle, Manager};
use crate::settings_manager::SettingsManager;
use dark_light::Mode;
use std::sync::Arc;

#[tauri::command]
pub async fn get_settings(app: AppHandle) -> Result<serde_json::Value, String> {
    let manager = app.state::<Arc<SettingsManager>>();
    let settings = manager.get();
    let mut value = serde_json::to_value(&settings).map_err(|e| e.to_string())?;

    #[cfg(not(feature = "app-store"))]
    {
        use tauri_plugin_autostart::ManagerExt;
        if let Ok(is_enabled) = app.autolaunch().is_enabled() {
            if let Some(obj) = value.as_object_mut() {
                obj.insert("startup_with_windows".to_string(), serde_json::json!(is_enabled));
            }
        }
    }

    #[cfg(all(feature = "app-store", target_os = "macos"))]
    {
        use smappservice_rs::{AppService, ServiceType, ServiceStatus};
        let app_service = AppService::new(ServiceType::MainApp);
        let is_enabled = matches!(app_service.status(), ServiceStatus::Enabled);
        if let Some(obj) = value.as_object_mut() {
            obj.insert("startup_with_windows".to_string(), serde_json::json!(is_enabled));
        }
    }

    Ok(value)
}

#[tauri::command]
pub async fn save_settings(app: AppHandle, settings: serde_json::Value) -> Result<(), String> {
    let manager = app.state::<Arc<SettingsManager>>();
    
    // Deserialize incoming settings (Frontend sends full object except ignored_apps)
    let mut new_settings: crate::models::AppSettings = serde_json::from_value(settings).map_err(|e| e.to_string())?;

    // Preserve ignored_apps from current state (as frontend doesn't send it in this call)
    let current = manager.get();
    new_settings.ignored_apps = current.ignored_apps;

    // Window effect
    let theme_str = new_settings.theme.clone();
    let mica_effect = new_settings.mica_effect.clone();
    log::info!("save_settings: mica_effect={}, theme={}", mica_effect, theme_str);
    match app.get_webview_window("main") {
        Some(win) => {
            let current_theme = if theme_str == "light" {
                tauri::Theme::Light
            } else if theme_str == "dark" {
                tauri::Theme::Dark
            } else {
                let mode = dark_light::detect().map_err(|e| {
                    log::error!("save_settings: dark_light::detect() failed: {:?}", e);
                    e.to_string()
                })?;
                match mode {
                    Mode::Dark => tauri::Theme::Dark,
                    _ => tauri::Theme::Light,
                }
            };
            crate::apply_window_effect(&win, &mica_effect, &current_theme);
        }
        None => {
            log::warn!("save_settings: main window not found, skipping window effect");
        }
    }

    #[cfg(not(feature = "app-store"))]
    {
        use tauri_plugin_autostart::ManagerExt;
        // Check if startup changed
        let startup = new_settings.startup_with_windows;
        let current_state = app.autolaunch().is_enabled().unwrap_or(false);
        if startup != current_state {
            if startup {
                let _ = app.autolaunch().enable();
            } else {
                let _ = app.autolaunch().disable();
            }
        }
    }
    #[cfg(all(feature = "app-store", target_os = "macos"))]
    {
        let startup = new_settings.startup_with_windows;
        use smappservice_rs::{AppService, ServiceType, ServiceStatus};
        let app_service = AppService::new(ServiceType::MainApp);
        let current_state = matches!(app_service.status(), ServiceStatus::Enabled);
        if startup != current_state {
            if startup {
                let _ = app_service.register();
            } else {
                let _ = app_service.unregister();
            }
        }
    }

    manager.save(new_settings)?;
    Ok(())
}

#[tauri::command]
pub async fn add_ignored_app(app_name: String, app: AppHandle) -> Result<(), String> {
    let manager = app.state::<Arc<SettingsManager>>();
    let mut current = manager.get();
    if current.ignored_apps.insert(app_name) {
        manager.save(current)?;
    }
    Ok(())
}

#[tauri::command]
pub async fn remove_ignored_app(app_name: String, app: AppHandle) -> Result<(), String> {
    let manager = app.state::<Arc<SettingsManager>>();
    let mut current = manager.get();
    if current.ignored_apps.remove(&app_name) {
        manager.save(current)?;
    }
    Ok(())
}

#[tauri::command]
pub async fn get_ignored_apps(app: AppHandle) -> Result<Vec<String>, String> {
    let manager = app.state::<Arc<SettingsManager>>();
    let mut apps: Vec<String> = manager.get().ignored_apps.into_iter().collect();
    apps.sort();
    Ok(apps)
}
