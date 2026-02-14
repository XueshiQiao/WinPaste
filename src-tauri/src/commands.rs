use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_clipboard_x::{write_text, stop_listening, start_listening};

use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut};
use std::str::FromStr;
use crate::database::Database;
use std::sync::Arc;
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use crate::models::{Clip, Folder, ClipboardItem, FolderItem};
use crate::ai::{self, AiConfig, AiAction};
use dark_light::Mode;

#[tauri::command]
pub async fn ai_process_clip(clip_id: String, action: String, db: tauri::State<'_, Arc<Database>>) -> Result<String, String> {
    let pool = &db.pool;

    // 1. Get Clip
    let clip: Clip = sqlx::query_as(r#"SELECT * FROM clips WHERE uuid = ?"#)
        .bind(&clip_id)
        .fetch_optional(pool).await.map_err(|e| e.to_string())?
        .ok_or("Clip not found")?;

    let text_content = if clip.clip_type == "text" || clip.clip_type == "html" || clip.clip_type == "url" {
         String::from_utf8_lossy(&clip.content).to_string()
    } else {
        return Err("AI processing only supported for text content".to_string());
    };

    // 2. Get AI Config
    let provider = db.get_setting("ai_provider").await.unwrap_or(None).unwrap_or("openai".to_string());
    let api_key = db.get_setting("ai_api_key").await.unwrap_or(None).unwrap_or_default();
    let model = db.get_setting("ai_model").await.unwrap_or(None).unwrap_or("gpt-3.5-turbo".to_string());
    let base_url = db.get_setting("ai_base_url").await.unwrap_or(None);

    if api_key.is_empty() {
        return Err("AI API Key is missing in settings".to_string());
    }

    let config = AiConfig {
        provider,
        api_key,
        model,
        base_url,
    };

    let ai_action = match action.as_str() {
        "summarize" => AiAction::Summarize,
        "translate" => AiAction::Translate,
        "explain_code" => AiAction::ExplainCode,
        "fix_grammar" => AiAction::FixGrammar,
        _ => return Err("Invalid AI action".to_string()),
    };

    let prompt_key = match ai_action {
        AiAction::Summarize => "ai_prompt_summarize",
        AiAction::Translate => "ai_prompt_translate",
        AiAction::ExplainCode => "ai_prompt_explain_code",
        AiAction::FixGrammar => "ai_prompt_fix_grammar",
    };

    let custom_prompt = db.get_setting(prompt_key).await.unwrap_or(None);

    // 3. Call AI
    let result = ai::process_text(&text_content, ai_action.clone(), &config, custom_prompt).await.map_err(|e| e.to_string())?;

    // 4. Update Metadata
    let mut metadata: serde_json::Value = if let Some(meta_str) = &clip.metadata {
        serde_json::from_str(meta_str).unwrap_or(serde_json::json!({}))
    } else {
        serde_json::json!({})
    };

    let key = match ai_action {
        AiAction::Summarize => "ai_summary",
        AiAction::Translate => "ai_translation",
        AiAction::ExplainCode => "ai_explanation",
        AiAction::FixGrammar => "ai_grammar_fix",
    };

    metadata[key] = serde_json::json!(result);
    let new_metadata_str = metadata.to_string();

    sqlx::query("UPDATE clips SET metadata = ? WHERE uuid = ?")
        .bind(&new_metadata_str)
        .bind(&clip_id)
        .execute(pool)
        .await
        .map_err(|e| e.to_string())?;

    Ok(result)
}

#[tauri::command]
pub async fn get_clips(filter_id: Option<String>, limit: i64, offset: i64, preview_only: Option<bool>, db: tauri::State<'_, Arc<Database>>) -> Result<Vec<ClipboardItem>, String> {
    let pool = &db.pool;
    let preview_only = preview_only.unwrap_or(false);

    log::info!("get_clips called with filter_id: {:?}, preview_only: {}", filter_id, preview_only);

    let clips: Vec<Clip> = match filter_id.as_deref() {
        Some(id) => {
            let folder_id_num = id.parse::<i64>().ok();
            if let Some(numeric_id) = folder_id_num {
                log::info!("Querying for folder_id: {}", numeric_id);
                sqlx::query_as(r#"
                    SELECT * FROM clips WHERE is_deleted = 0 AND folder_id = ?
                    ORDER BY created_at DESC LIMIT ? OFFSET ?
                "#)
                .bind(numeric_id)
                .bind(limit)
                .bind(offset)
                .fetch_all(pool).await.map_err(|e| e.to_string())?
            } else {
                log::info!("Unknown folder_id, returning empty");
                Vec::new()
            }
        }
        None => {
            log::info!("Querying for items, offset: {}, limit: {}", offset, limit);
            sqlx::query_as(r#"
                SELECT * FROM clips WHERE is_deleted = 0
                ORDER BY created_at DESC LIMIT ? OFFSET ?
            "#)
            .bind(limit)
            .bind(offset)
            .fetch_all(pool).await.map_err(|e| e.to_string())?
        }
    };

    log::info!("DB: Found {} clips", clips.len());

    let items: Vec<ClipboardItem> = clips.iter().enumerate().map(|(idx, clip)| {
        let content_str = if preview_only && clip.clip_type == "image" {
            // In preview mode, don't send full image data - just empty string
            String::new()
        } else if clip.clip_type == "image" {
            BASE64.encode(&clip.content)
        } else {
            String::from_utf8_lossy(&clip.content).to_string()
        };

        // Only log first 10 clips to reduce noise
        if idx < 10 {
            log::trace!("{} Clip {}: type='{}', content_len={}", idx, clip.uuid, clip.clip_type, content_str.len());
        }

        ClipboardItem {
            id: clip.uuid.clone(),
            clip_type: clip.clip_type.clone(),
            content: content_str,
            preview: clip.text_preview.clone(),
            folder_id: clip.folder_id.map(|id| id.to_string()),
            created_at: clip.created_at.to_rfc3339(),
            source_app: clip.source_app.clone(),
            source_icon: clip.source_icon.clone(),
            metadata: clip.metadata.clone(),
        }
    }).collect();

    Ok(items)
}

#[tauri::command]
pub async fn get_clip(id: String, db: tauri::State<'_, Arc<Database>>) -> Result<ClipboardItem, String> {
    let pool = &db.pool;

    let clip: Option<Clip> = sqlx::query_as(r#"SELECT * FROM clips WHERE uuid = ?"#)
        .bind(&id)
        .fetch_optional(pool).await.map_err(|e| e.to_string())?;

    match clip {
        Some(clip) => {
            let content_str = if clip.clip_type == "image" {
                BASE64.encode(&clip.content)
            } else {
                String::from_utf8_lossy(&clip.content).to_string()
            };

            Ok(ClipboardItem {
                id: clip.uuid,
                clip_type: clip.clip_type,
                content: content_str,
                preview: clip.text_preview,
                folder_id: clip.folder_id.map(|id| id.to_string()),
                created_at: clip.created_at.to_rfc3339(),
                source_app: clip.source_app,
                source_icon: clip.source_icon,
                metadata: clip.metadata,
            })
        }
        None => Err("Clip not found".to_string()),
    }
}

#[tauri::command]
pub async fn paste_clip(id: String, app: AppHandle, window: tauri::WebviewWindow, db: tauri::State<'_, Arc<Database>>) -> Result<(), String> {
    let pool = &db.pool;

    let clip: Option<Clip> = sqlx::query_as(r#"SELECT * FROM clips WHERE uuid = ?"#)
        .bind(&id)
        .fetch_optional(pool).await.map_err(|e| e.to_string())?;

    match clip {
        Some(clip) => {
            // Synchronize clipboard access across the app
            let _guard = crate::clipboard::CLIPBOARD_SYNC.lock().await;

            let content_hash = clip.content_hash.clone();
            let uuid = clip.uuid.clone();

            // Stop monitor
            if let Err(e) = stop_listening().await {
                 log::error!("Failed to stop listener: {}", e);
            }

            let mut final_res = Ok(());

            if clip.clip_type == "image" {
                crate::clipboard::set_ignore_hash(content_hash.clone());
                crate::clipboard::set_last_stable_hash(content_hash.clone());

                #[cfg(target_os = "macos")]
                {
                    // Write PNG to temp file + file URL on pasteboard (fast path via disk)
                    if let Err(e) = crate::clipboard::write_png_to_pasteboard(&clip.content) {
                        final_res = Err(format!("Failed to write image to clipboard: {}", e));
                    }
                }

                #[cfg(not(target_os = "macos"))]
                {
                    // On Windows, frontend writes image via navigator.clipboard API
                }

            } else {
                let content_str = String::from_utf8_lossy(&clip.content).to_string();
                crate::clipboard::set_ignore_hash(content_hash.clone());
                crate::clipboard::set_last_stable_hash(content_hash.clone());

                let mut last_err = String::new();
                for i in 0..5 {
                    match write_text(content_str.clone()).await {
                        Ok(_) => { last_err.clear(); break; },
                        Err(e) => {
                            last_err = e.to_string();
                            log::warn!("Clipboard write (text) attempt {} failed: {}. Retrying...", i+1, last_err);
                            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                        }
                    }
                }
                if !last_err.is_empty() {
                    final_res = Err(format!("Failed to set clipboard text: {}", last_err));
                }
            }

            // Manually perform the LRU bump (update created_at)
            let _ = sqlx::query(r#"UPDATE clips SET created_at = CURRENT_TIMESTAMP WHERE uuid = ?"#)
                .bind(&uuid)
                .execute(pool)
                .await;

            // Restart monitor
            let app_clone = app.clone();
            if let Err(e) = start_listening(app_clone).await {
                log::error!("Failed to restart listener: {}", e);
            }

            if final_res.is_ok() {
                let content = if clip.clip_type == "image" { "[Image]".to_string() } else { String::from_utf8_lossy(&clip.content).to_string() };
                let _ = window.emit("clipboard-write", &content);

                // Check auto_paste setting
                let auto_paste = sqlx::query_scalar::<_, String>(r#"SELECT value FROM settings WHERE key = 'auto_paste'"#)
                    .fetch_optional(pool)
                    .await
                    .unwrap_or(None)
                    .and_then(|v| v.parse::<bool>().ok())
                    .unwrap_or(true); // Default true

                if auto_paste {
                    // Auto-Paste Logic
                    // 1. Hide window immediately to trigger focus switch to previous app
                    crate::animate_window_hide(&window, Some(Box::new(move || {
                        // 2. Callback executed AFTER window is hidden
                        #[cfg(target_os = "windows")]
                        {
                            // Small buffer to ensure OS focus switch is complete
                            std::thread::sleep(std::time::Duration::from_millis(200));
                            crate::clipboard::send_paste_input();
                        }
                        #[cfg(target_os = "macos")]
                        {
                            std::thread::sleep(std::time::Duration::from_millis(100));
                            crate::clipboard::send_paste_input();
                        }
                    })));
                } else {
                     // If auto_paste is disabled, we still hide the window (as requested by original "copy to text field" intent,
                     // but maybe user just wants to copy?)
                     // Actually, if auto_paste is OFF, standard behavior for "Enter/Double Click" in clipboard managers is usually "Copy & Close".
                     crate::animate_window_hide(&window, None);
                }
            }
            final_res
        }
        None => Err("Clip not found".to_string()),
    }
}

#[tauri::command]
pub async fn delete_clip(id: String, hard_delete: bool, db: tauri::State<'_, Arc<Database>>) -> Result<(), String> {
    let pool = &db.pool;

    if hard_delete {
        sqlx::query(r#"DELETE FROM clips WHERE uuid = ?"#)
            .bind(&id)
            .execute(pool).await.map_err(|e| e.to_string())?;
    } else {
        sqlx::query(r#"UPDATE clips SET is_deleted = 1 WHERE uuid = ?"#)
            .bind(&id)
            .execute(pool).await.map_err(|e| e.to_string())?;
    }
    Ok(())
}



#[tauri::command]
pub async fn move_to_folder(clip_id: String, folder_id: Option<String>, db: tauri::State<'_, Arc<Database>>) -> Result<(), String> {
    let pool = &db.pool;

    let folder_id = match folder_id {
        Some(id) => Some(id.parse::<i64>().map_err(|_| "Invalid folder ID")?),
        None => None,
    };

    sqlx::query(r#"UPDATE clips SET folder_id = ? WHERE uuid = ?"#)
        .bind(folder_id)
        .bind(&clip_id)
        .execute(pool).await.map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn create_folder(name: String, icon: Option<String>, color: Option<String>, db: tauri::State<'_, Arc<Database>>, window: tauri::WebviewWindow) -> Result<FolderItem, String> {
    let pool = &db.pool;

    // Check if folder with same name exists (excluding system folders if we wanted, but name uniqueness is good generally)
    let exists: Option<i64> = sqlx::query_scalar("SELECT 1 FROM folders WHERE name = ?")
        .bind(&name)
        .fetch_optional(pool).await.map_err(|e| e.to_string())?;

    if exists.is_some() {
        return Err("A folder with this name already exists".to_string());
    }

    let id = sqlx::query(r#"INSERT INTO folders (name, icon, color) VALUES (?, ?, ?)"#)
        .bind(&name)
        .bind(icon.as_ref())
        .bind(color.as_ref())
        .execute(pool).await.map_err(|e| e.to_string())?
        .last_insert_rowid();

    let _ = window.emit("clipboard-change", ());

    Ok(FolderItem {
        id: id.to_string(),
        name,
        icon,
        color,
        is_system: false,
        item_count: 0,
    })
}

#[tauri::command]
pub async fn delete_folder(id: String, db: tauri::State<'_, Arc<Database>>, window: tauri::WebviewWindow) -> Result<(), String> {
    let pool = &db.pool;

    let folder_id: i64 = id.parse().map_err(|_| "Invalid folder ID")?;
    sqlx::query(r#"DELETE FROM folders WHERE id = ?"#)
        .bind(folder_id)
        .execute(pool).await.map_err(|e| e.to_string())?;

    let _ = window.emit("clipboard-change", ());
    Ok(())
}

#[tauri::command]
pub async fn rename_folder(id: String, name: String, db: tauri::State<'_, Arc<Database>>, window: tauri::WebviewWindow) -> Result<(), String> {
    let pool = &db.pool;

    let folder_id: i64 = id.parse().map_err(|_| "Invalid folder ID")?;

    // Check availability
    let exists: Option<i64> = sqlx::query_scalar("SELECT 1 FROM folders WHERE name = ? AND id != ?")
        .bind(&name)
        .bind(folder_id)
        .fetch_optional(pool).await.map_err(|e| e.to_string())?;

    if exists.is_some() {
        return Err("A folder with this name already exists".to_string());
    }

    sqlx::query(r#"UPDATE folders SET name = ? WHERE id = ?"#)
        .bind(name)
        .bind(folder_id)
        .execute(pool).await.map_err(|e| e.to_string())?;

    // Emit event so main window knows to refresh
    let _ = window.emit("clipboard-change", ());
    Ok(())
}

#[tauri::command]
pub async fn search_clips(query: String, filter_id: Option<String>, limit: i64, offset: i64, db: tauri::State<'_, Arc<Database>>) -> Result<Vec<ClipboardItem>, String> {
    let pool = &db.pool;

    let search_pattern = format!("%{}%", query);

    let clips: Vec<Clip> = match filter_id.as_deref() {
        Some(id) => {
            let folder_id_num = id.parse::<i64>().ok();
            if let Some(numeric_id) = folder_id_num {
                sqlx::query_as(r#"
                    SELECT * FROM clips WHERE is_deleted = 0 AND folder_id = ? AND (text_preview LIKE ? OR content LIKE ?)
                    ORDER BY created_at DESC LIMIT ? OFFSET ?
                "#)
                .bind(numeric_id)
                .bind(&search_pattern)
                .bind(&search_pattern)
                .bind(limit)
                .bind(offset)
                .fetch_all(pool).await.map_err(|e| e.to_string())?
            } else {
                Vec::new()
            }
        }
        None => {
            sqlx::query_as(r#"
                SELECT * FROM clips WHERE is_deleted = 0 AND (text_preview LIKE ? OR content LIKE ?)
                ORDER BY created_at DESC LIMIT ? OFFSET ?
            "#)
            .bind(&search_pattern)
            .bind(&search_pattern)
            .bind(limit)
            .bind(offset)
            .fetch_all(pool).await.map_err(|e| e.to_string())?
        }
    };

    let items: Vec<ClipboardItem> = clips.iter().map(|clip| {
        let content_str = if clip.clip_type == "image" {
            BASE64.encode(&clip.content)
        } else {
            String::from_utf8_lossy(&clip.content).to_string()
        };

        ClipboardItem {
            id: clip.uuid.clone(),
            clip_type: clip.clip_type.clone(),
            content: content_str,
            preview: clip.text_preview.clone(),
            folder_id: clip.folder_id.map(|id| id.to_string()),
            created_at: clip.created_at.to_rfc3339(),
            source_app: clip.source_app.clone(),
            source_icon: clip.source_icon.clone(),
            metadata: clip.metadata.clone(),
        }
    }).collect();

    Ok(items)
}

#[tauri::command]
pub async fn get_folders(db: tauri::State<'_, Arc<Database>>) -> Result<Vec<FolderItem>, String> {
    let pool = &db.pool;

    let folders: Vec<Folder> = sqlx::query_as(r#"SELECT * FROM folders ORDER BY created_at"#)
        .fetch_all(pool).await.map_err(|e| e.to_string())?;

    // Get counts for all folders in one query
    let counts: Vec<(i64, i64)> = sqlx::query_as(r#"
        SELECT folder_id, COUNT(*) as count
        FROM clips
        WHERE is_deleted = 0 AND folder_id IS NOT NULL
        GROUP BY folder_id
    "#)
    .fetch_all(pool).await.map_err(|e| e.to_string())?;

    // Create a map for easier lookup
    use std::collections::HashMap;
    let count_map: HashMap<i64, i64> = counts.into_iter().collect();

    let items: Vec<FolderItem> = folders.iter().map(|folder| {
        FolderItem {
            id: folder.id.to_string(),
            name: folder.name.clone(),
            icon: folder.icon.clone(),
            color: folder.color.clone(),
            is_system: folder.is_system,
            item_count: *count_map.get(&folder.id).unwrap_or(&0),
        }
    }).collect();

    //println!("folder items: {:#?}", items);

    Ok(items)
}

#[tauri::command]
#[allow(unused_variables)]
pub async fn get_settings(app: AppHandle, db: tauri::State<'_, Arc<Database>>) -> Result<serde_json::Value, String> {
    #[cfg(not(feature = "app-store"))]
    use tauri_plugin_autostart::ManagerExt;
    let pool = &db.pool;

    let mut settings = serde_json::json!({
        "max_items": 1000,
        "auto_delete_days": 30,
        "startup_with_windows": false, // Default, will override below
        "show_in_taskbar": false,
        "hotkey": if cfg!(target_os = "macos") { "Cmd+Shift+V" } else { "Ctrl+Shift+V" },
        "theme": "dark",
        "mica_effect": "clear",
        "auto_paste": true,
        "ignore_ghost_clips": false
    });

    // ... (rest of get_settings until autostart check)

    if let Ok(Some(value)) = sqlx::query_scalar::<_, String>(r#"SELECT value FROM settings WHERE key = 'mica_effect'"#)
        .fetch_optional(pool).await.map_err(|e| e.to_string())
    {
        settings["mica_effect"] = serde_json::json!(value);
    }

    if let Ok(Some(value)) = sqlx::query_scalar::<_, String>(r#"SELECT value FROM settings WHERE key = 'ignore_ghost_clips'"#)
        .fetch_optional(pool).await.map_err(|e| e.to_string())
    {
        if let Ok(b) = value.parse::<bool>() {
            settings["ignore_ghost_clips"] = serde_json::json!(b);
        }
    }

    if let Ok(Some(value)) = sqlx::query_scalar::<_, String>(r#"SELECT value FROM settings WHERE key = 'auto_paste'"#)
        .fetch_optional(pool).await.map_err(|e| e.to_string())
    {
        if let Ok(b) = value.parse::<bool>() {
            settings["auto_paste"] = serde_json::json!(b);
        }
    }

    if let Ok(Some(value)) = sqlx::query_scalar::<_, String>(r#"SELECT value FROM settings WHERE key = 'max_items'"#)
        .fetch_optional(pool).await.map_err(|e| e.to_string())
    {
        if let Ok(num) = value.parse::<i64>() {
            settings["max_items"] = serde_json::json!(num);
        }
    }

    if let Ok(Some(value)) = sqlx::query_scalar::<_, String>(r#"SELECT value FROM settings WHERE key = 'auto_delete_days'"#)
        .fetch_optional(pool).await.map_err(|e| e.to_string())
    {
        if let Ok(num) = value.parse::<i64>() {
            settings["auto_delete_days"] = serde_json::json!(num);
        }
    }

    if let Ok(Some(value)) = sqlx::query_scalar::<_, String>(r#"SELECT value FROM settings WHERE key = 'theme'"#)
        .fetch_optional(pool).await.map_err(|e| e.to_string())
    {
        settings["theme"] = serde_json::json!(value);
    }

    if let Ok(Some(value)) = sqlx::query_scalar::<_, String>(r#"SELECT value FROM settings WHERE key = 'hotkey'"#)
        .fetch_optional(pool).await.map_err(|e| e.to_string())
    {
        settings["hotkey"] = serde_json::json!(value);
    }

    if let Ok(Some(value)) = sqlx::query_scalar::<_, String>(r#"SELECT value FROM settings WHERE key = 'language'"#)
        .fetch_optional(pool).await.map_err(|e| e.to_string())
    {
        settings["language"] = serde_json::json!(value);
    }

    // AI Settings
    if let Ok(Some(value)) = sqlx::query_scalar::<_, String>(r#"SELECT value FROM settings WHERE key = 'ai_provider'"#)
        .fetch_optional(pool).await.map_err(|e| e.to_string()) { settings["ai_provider"] = serde_json::json!(value); }

    if let Ok(Some(value)) = sqlx::query_scalar::<_, String>(r#"SELECT value FROM settings WHERE key = 'ai_api_key'"#)
        .fetch_optional(pool).await.map_err(|e| e.to_string()) { settings["ai_api_key"] = serde_json::json!(value); }

    if let Ok(Some(value)) = sqlx::query_scalar::<_, String>(r#"SELECT value FROM settings WHERE key = 'ai_model'"#)
        .fetch_optional(pool).await.map_err(|e| e.to_string()) { settings["ai_model"] = serde_json::json!(value); }

    if let Ok(Some(value)) = sqlx::query_scalar::<_, String>(r#"SELECT value FROM settings WHERE key = 'ai_base_url'"#)
        .fetch_optional(pool).await.map_err(|e| e.to_string()) { settings["ai_base_url"] = serde_json::json!(value); }

    if let Ok(Some(value)) = sqlx::query_scalar::<_, String>(r#"SELECT value FROM settings WHERE key = 'ai_prompt_summarize'"#)
        .fetch_optional(pool).await.map_err(|e| e.to_string()) { settings["ai_prompt_summarize"] = serde_json::json!(value); }

    if let Ok(Some(value)) = sqlx::query_scalar::<_, String>(r#"SELECT value FROM settings WHERE key = 'ai_prompt_translate'"#)
        .fetch_optional(pool).await.map_err(|e| e.to_string()) { settings["ai_prompt_translate"] = serde_json::json!(value); }

    if let Ok(Some(value)) = sqlx::query_scalar::<_, String>(r#"SELECT value FROM settings WHERE key = 'ai_prompt_explain_code'"#)
        .fetch_optional(pool).await.map_err(|e| e.to_string()) { settings["ai_prompt_explain_code"] = serde_json::json!(value); }

    if let Ok(Some(value)) = sqlx::query_scalar::<_, String>(r#"SELECT value FROM settings WHERE key = 'ai_prompt_fix_grammar'"#)
        .fetch_optional(pool).await.map_err(|e| e.to_string()) { settings["ai_prompt_fix_grammar"] = serde_json::json!(value); }

    if let Ok(Some(value)) = sqlx::query_scalar::<_, String>(r#"SELECT value FROM settings WHERE key = 'ai_title_summarize'"#)
        .fetch_optional(pool).await.map_err(|e| e.to_string()) { settings["ai_title_summarize"] = serde_json::json!(value); }

    if let Ok(Some(value)) = sqlx::query_scalar::<_, String>(r#"SELECT value FROM settings WHERE key = 'ai_title_translate'"#)
        .fetch_optional(pool).await.map_err(|e| e.to_string()) { settings["ai_title_translate"] = serde_json::json!(value); }

    if let Ok(Some(value)) = sqlx::query_scalar::<_, String>(r#"SELECT value FROM settings WHERE key = 'ai_title_explain_code'"#)
        .fetch_optional(pool).await.map_err(|e| e.to_string()) { settings["ai_title_explain_code"] = serde_json::json!(value); }

    if let Ok(Some(value)) = sqlx::query_scalar::<_, String>(r#"SELECT value FROM settings WHERE key = 'ai_title_fix_grammar'"#)
        .fetch_optional(pool).await.map_err(|e| e.to_string()) { settings["ai_title_fix_grammar"] = serde_json::json!(value); }

    // Check actual autostart status
    #[cfg(not(feature = "app-store"))]
    {
        if let Ok(is_enabled) = app.autolaunch().is_enabled() {
            settings["startup_with_windows"] = serde_json::json!(is_enabled);
            log::info!("autostart enabled: {}", is_enabled);
        } else {
            log::info!("autostart not enabled");
        }
    }

    #[cfg(all(feature = "app-store", target_os = "macos"))]
    {
        use smappservice_rs::{AppService, ServiceType, ServiceStatus};
        let app_service = AppService::new(ServiceType::MainApp);
        let is_enabled = matches!(app_service.status(), ServiceStatus::Enabled);
        settings["startup_with_windows"] = serde_json::json!(is_enabled);
        log::info!("autostart (SMAppService) enabled: {}", is_enabled);
    }

    Ok(settings)
}

#[tauri::command]
pub async fn save_settings(app: AppHandle, settings: serde_json::Value, db: tauri::State<'_, Arc<Database>>) -> Result<(), String> {
    #[cfg(not(feature = "app-store"))]
    use tauri_plugin_autostart::ManagerExt;
    let pool = &db.pool;

    if let Some(max_items) = settings.get("max_items").and_then(|v| v.as_i64()) {
        sqlx::query(r#"INSERT OR REPLACE INTO settings (key, value) VALUES ('max_items', ?)"#)
            .bind(max_items.to_string())
            .execute(pool).await.ok();
    }

    if let Some(days) = settings.get("auto_delete_days").and_then(|v| v.as_i64()) {
        sqlx::query(r#"INSERT OR REPLACE INTO settings (key, value) VALUES ('auto_delete_days', ?)"#)
            .bind(days.to_string())
            .execute(pool).await.ok();
    }

    if let Some(theme) = settings.get("theme").and_then(|v| v.as_str()) {
        sqlx::query(r#"INSERT OR REPLACE INTO settings (key, value) VALUES ('theme', ?)"#)
            .bind(theme)
            .execute(pool).await.ok();
    }

    if let Some(mica_effect) = settings.get("mica_effect").and_then(|v| v.as_str()) {
        sqlx::query(r#"INSERT OR REPLACE INTO settings (key, value) VALUES ('mica_effect', ?)"#)
            .bind(mica_effect)
            .execute(pool).await.ok();
    }

    if let Some(language) = settings.get("language").and_then(|v| v.as_str()) {
        sqlx::query(r#"INSERT OR REPLACE INTO settings (key, value) VALUES ('language', ?)"#)
            .bind(language)
            .execute(pool).await.ok();
    }

    // Always re-apply window effect when theme or mica_effect might have changed
    let theme_str = settings.get("theme").and_then(|v| v.as_str()).unwrap_or("system");
    let mica_effect = settings.get("mica_effect").and_then(|v| v.as_str()).unwrap_or("clear");
    if let Some(win) = app.get_webview_window("main") {
        // get current system theme
        let current_theme = if theme_str == "light" {
            tauri::Theme::Light
        } else if theme_str == "dark" {
            tauri::Theme::Dark
        } else {
            let mode = dark_light::detect().map_err(|e| {
                log::error!("THEME: Failed to detect system theme: {:?} via dark_light::detect()", e);
                e.to_string()
            })?;

            let theme2 = match mode {
                Mode::Dark => tauri::Theme::Dark,
                Mode::Light => tauri::Theme::Light,
                _ => tauri::Theme::Light,
            };

            log::info!("THEME: win.theme(): {:?}, dark_light::detectd(): {:?}", win.theme(), theme2);

            // sometimes win.theme() is not right. don't why for now..
            // win.theme().unwrap_or_else(|err| {
            //     log::error!("THEME: Failed to get system theme: {:?}, defaulting to Light", err);
            //     tauri::Theme::Light
            // })
            theme2
        };
        log::info!("THEME:Applying window effect: {} with theme: {:?} (setting:{:?}", mica_effect, current_theme, theme_str);
        crate::apply_window_effect(&win, mica_effect, &current_theme);
    }

    if let Some(ai_provider) = settings.get("ai_provider").and_then(|v| v.as_str()) {
        sqlx::query(r#"INSERT OR REPLACE INTO settings (key, value) VALUES ('ai_provider', ?)"#)
            .bind(ai_provider)
            .execute(pool).await.ok();
    }
    if let Some(ai_api_key) = settings.get("ai_api_key").and_then(|v| v.as_str()) {
        sqlx::query(r#"INSERT OR REPLACE INTO settings (key, value) VALUES ('ai_api_key', ?)"#)
            .bind(ai_api_key)
            .execute(pool).await.ok();
    }
    if let Some(ai_model) = settings.get("ai_model").and_then(|v| v.as_str()) {
        sqlx::query(r#"INSERT OR REPLACE INTO settings (key, value) VALUES ('ai_model', ?)"#)
            .bind(ai_model)
            .execute(pool).await.ok();
    }
    if let Some(ai_base_url) = settings.get("ai_base_url").and_then(|v| v.as_str()) {
        sqlx::query(r#"INSERT OR REPLACE INTO settings (key, value) VALUES ('ai_base_url', ?)"#)
            .bind(ai_base_url)
            .execute(pool).await.ok();
    }

    if let Some(ai_prompt_summarize) = settings.get("ai_prompt_summarize").and_then(|v| v.as_str()) {
        sqlx::query(r#"INSERT OR REPLACE INTO settings (key, value) VALUES ('ai_prompt_summarize', ?)"#)
            .bind(ai_prompt_summarize)
            .execute(pool).await.ok();
    }

    if let Some(ai_prompt_translate) = settings.get("ai_prompt_translate").and_then(|v| v.as_str()) {
        sqlx::query(r#"INSERT OR REPLACE INTO settings (key, value) VALUES ('ai_prompt_translate', ?)"#)
            .bind(ai_prompt_translate)
            .execute(pool).await.ok();
    }

    if let Some(ai_prompt_explain_code) = settings.get("ai_prompt_explain_code").and_then(|v| v.as_str()) {
        sqlx::query(r#"INSERT OR REPLACE INTO settings (key, value) VALUES ('ai_prompt_explain_code', ?)"#)
            .bind(ai_prompt_explain_code)
            .execute(pool).await.ok();
    }

    if let Some(ai_prompt_fix_grammar) = settings.get("ai_prompt_fix_grammar").and_then(|v| v.as_str()) {
        sqlx::query(r#"INSERT OR REPLACE INTO settings (key, value) VALUES ('ai_prompt_fix_grammar', ?)"#)
            .bind(ai_prompt_fix_grammar)
            .execute(pool).await.ok();
    }

    if let Some(ai_title_summarize) = settings.get("ai_title_summarize").and_then(|v| v.as_str()) {
        sqlx::query(r#"INSERT OR REPLACE INTO settings (key, value) VALUES ('ai_title_summarize', ?)"#)
            .bind(ai_title_summarize)
            .execute(pool).await.ok();
    }

    if let Some(ai_title_translate) = settings.get("ai_title_translate").and_then(|v| v.as_str()) {
        sqlx::query(r#"INSERT OR REPLACE INTO settings (key, value) VALUES ('ai_title_translate', ?)"#)
            .bind(ai_title_translate)
            .execute(pool).await.ok();
    }

    if let Some(ai_title_explain_code) = settings.get("ai_title_explain_code").and_then(|v| v.as_str()) {
        sqlx::query(r#"INSERT OR REPLACE INTO settings (key, value) VALUES ('ai_title_explain_code', ?)"#)
            .bind(ai_title_explain_code)
            .execute(pool).await.ok();
    }

    if let Some(ai_title_fix_grammar) = settings.get("ai_title_fix_grammar").and_then(|v| v.as_str()) {
        sqlx::query(r#"INSERT OR REPLACE INTO settings (key, value) VALUES ('ai_title_fix_grammar', ?)"#)
            .bind(ai_title_fix_grammar)
            .execute(pool).await.ok();
    }

    if let Some(hotkey) = settings.get("hotkey").and_then(|v| v.as_str()) {
        sqlx::query(r#"INSERT OR REPLACE INTO settings (key, value) VALUES ('hotkey', ?)"#)
            .bind(hotkey)
            .execute(pool).await.ok();
    }

    if let Some(auto_paste) = settings.get("auto_paste").and_then(|v| v.as_bool()) {
        sqlx::query(r#"INSERT OR REPLACE INTO settings (key, value) VALUES ('auto_paste', ?)"#)
            .bind(auto_paste.to_string())
            .execute(pool).await.ok();
    }

    if let Some(ignore_ghost) = settings.get("ignore_ghost_clips").and_then(|v| v.as_bool()) {
        sqlx::query(r#"INSERT OR REPLACE INTO settings (key, value) VALUES ('ignore_ghost_clips', ?)"#)
            .bind(ignore_ghost.to_string())
            .execute(pool).await.ok();
    }

    #[cfg(not(feature = "app-store"))]
    {
        if let Some(startup) = settings.get("startup_with_windows").and_then(|v| v.as_bool()) {
            let current_state = app.autolaunch().is_enabled().unwrap_or(false);
            if startup != current_state {
                if startup {
                    if let Err(e) = app.autolaunch().enable() {
                        log::warn!("Failed to enable autostart: {}", e);
                    }
                } else {
                    if let Err(e) = app.autolaunch().disable() {
                        log::warn!("Failed to disable autostart: {}", e);
                    }
                }
            }
        }
    }

    #[cfg(all(feature = "app-store", target_os = "macos"))]
    {
        if let Some(startup) = settings.get("startup_with_windows").and_then(|v| v.as_bool()) {
            use smappservice_rs::{AppService, ServiceType, ServiceStatus};
            let app_service = AppService::new(ServiceType::MainApp);
            let current_state = matches!(app_service.status(), ServiceStatus::Enabled);

            if startup != current_state {
                if startup {
                    if let Err(e) = app_service.register() {
                        log::error!("Failed to register SMAppService: {}", e);
                        return Err(format!("Failed to enable autostart: {}", e));
                    }
                } else {
                    if let Err(e) = app_service.unregister() {
                        log::error!("Failed to unregister SMAppService: {}", e);
                        return Err(format!("Failed to disable autostart: {}", e));
                    }
                }
            }
        }
    }

    Ok(())
}

#[tauri::command]
pub fn hide_window(window: tauri::WebviewWindow) -> Result<(), String> {
    window.hide().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn ping() -> Result<String, String> {
    Ok("pong".to_string())
}

#[tauri::command]
pub fn test_log() -> Result<String, String> {
    log::trace!("[TEST] Trace level log");
    log::debug!("[TEST] Debug level log");
    log::info!("[TEST] Info level log");
    log::warn!("[TEST] Warn level log");
    log::error!("[TEST] Error level log");
    Ok("Logs emitted - check console".to_string())
}

#[tauri::command]
pub async fn get_clipboard_history_size(db: tauri::State<'_, Arc<Database>>) -> Result<i64, String> {
    let pool = &db.pool;

    let count: i64 = sqlx::query_scalar::<_, i64>(r#"SELECT COUNT(*) FROM clips WHERE is_deleted = 0"#)
        .fetch_one(pool).await.map_err(|e| e.to_string())?;
    Ok(count)
}

#[tauri::command]
pub async fn clear_clipboard_history(db: tauri::State<'_, Arc<Database>>) -> Result<(), String> {
    let pool = &db.pool;

    sqlx::query(r#"DELETE FROM clips WHERE is_deleted = 1"#)
        .execute(pool).await.map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn clear_all_clips(db: tauri::State<'_, Arc<Database>>) -> Result<(), String> {
    let pool = &db.pool;

    sqlx::query(r#"DELETE FROM clips"#)
        .execute(pool).await.map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn remove_duplicate_clips(db: tauri::State<'_, Arc<Database>>) -> Result<i64, String> {
    let pool = &db.pool;

    let result = sqlx::query(r#"
        DELETE FROM clips
        WHERE id NOT IN (
            SELECT MIN(id)
            FROM clips
            GROUP BY content_hash
        )
    "#)
    .execute(pool).await.map_err(|e| e.to_string())?;

    Ok(result.rows_affected() as i64)
}

#[tauri::command]
pub async fn register_global_shortcut(hotkey: String, window: tauri::WebviewWindow) -> Result<(), String> {
    use tauri_plugin_global_shortcut::ShortcutState;

    let app = window.app_handle();
    let shortcut = Shortcut::from_str(&hotkey).map_err(|e| format!("Invalid hotkey: {:?}", e))?;

    // Unregister all existing shortcuts first
    if let Err(e) = app.global_shortcut().unregister_all() {
        log::warn!("Failed to unregister existing shortcuts: {:?}", e);
    }

    // Get the main window for the handler
    let main_window = app.get_webview_window("main")
        .ok_or_else(|| "Main window not found".to_string())?;

    // Register the new shortcut with the window show handler
    let win_clone = main_window.clone();
    if let Err(e) = app.global_shortcut().on_shortcut(shortcut, move |_app, _shortcut, event| {
        if event.state() == ShortcutState::Pressed {
            crate::position_window_at_bottom(&win_clone);
        }
    }) {
        return Err(format!("Failed to register hotkey: {:?}", e));
    }

    log::info!("Registered global shortcut: {}", hotkey);
    Ok(())
}

#[tauri::command]
pub async fn focus_window(app: AppHandle, label: String) -> Result<(), String> {
    if let Some(window) = app.get_webview_window(&label) {
        if let Err(e) = window.unminimize() {
            log::warn!("Failed to unminimize window {}: {:?}", label, e);
        }
        if let Err(e) = window.show() {
            log::warn!("Failed to show window {}: {:?}", label, e);
        }
        if let Err(e) = window.set_focus() {
            log::warn!("Failed to focus window {}: {:?}", label, e);
        }

        Ok(())
    } else {
        Err(format!("Window {} not found", label))
    }
}

#[tauri::command]
pub fn show_window(window: tauri::WebviewWindow) -> Result<(), String> {
    crate::position_window_at_bottom(&window);
    Ok(())
}

#[tauri::command]
pub async fn add_ignored_app(app_name: String, db: tauri::State<'_, Arc<Database>>) -> Result<(), String> {
    db.add_ignored_app(&app_name).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn remove_ignored_app(app_name: String, db: tauri::State<'_, Arc<Database>>) -> Result<(), String> {
    db.remove_ignored_app(&app_name).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_ignored_apps(db: tauri::State<'_, Arc<Database>>) -> Result<Vec<String>, String> {
    db.get_ignored_apps().await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn pick_file(app: AppHandle) -> Result<String, String> {
    use tauri_plugin_dialog::DialogExt;

    let file_path = app
        .dialog()
        .file()
        .add_filter("Executables", &["exe", "app"])
        .blocking_pick_file();

    match file_path {
        Some(path) => Ok(path.to_string()),
        None => Err("No file selected".to_string()),
    }
}

#[tauri::command]
pub fn get_layout_config() -> serde_json::Value {
    serde_json::json!({
        "window_height": crate::constants::WINDOW_HEIGHT,
    })
}

#[tauri::command]
pub async fn check_accessibility_permissions() -> Result<bool, String> {
    #[cfg(target_os = "macos")]
    {
        Ok(crate::source_app_macos::is_accessibility_enabled())
    }
    #[cfg(not(target_os = "macos"))]
    {
        Ok(true)
    }
}

#[tauri::command]
pub async fn request_accessibility_permissions() -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        crate::source_app_macos::open_accessibility_settings();
        Ok(())
    }
    #[cfg(not(target_os = "macos"))]
    {
        Ok(())
    }
}
