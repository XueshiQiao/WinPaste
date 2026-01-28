use tauri::{Emitter, Manager, AppHandle};
use tauri_plugin_clipboard::Clipboard;
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut};
use std::str::FromStr;
use crate::database::Database;
use std::sync::Arc;
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};

#[derive(Debug, Clone, sqlx::FromRow)]
struct Clip {
    id: i64,
    uuid: String,
    clip_type: String,
    content: Vec<u8>,
    text_preview: String,
    content_hash: String,
    folder_id: Option<i64>,
    is_pinned: bool,
    is_deleted: bool,
    source_app: Option<String>,
    source_icon: Option<String>,
    metadata: Option<String>,
    created_at: chrono::DateTime<chrono::Utc>,
    last_accessed: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, sqlx::FromRow)]
struct Folder {
    id: i64,
    name: String,
    icon: Option<String>,
    color: Option<String>,
    is_system: bool,
    created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub(crate) struct ClipboardItem {
    pub id: String,
    pub clip_type: String,
    pub content: String,
    pub preview: String,
    pub is_pinned: bool,
    pub folder_id: Option<String>,
    pub created_at: String,
    pub source_app: Option<String>,
    pub source_icon: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub(crate) struct FolderItem {
    id: String,
    name: String,
    icon: Option<String>,
    color: Option<String>,
    is_system: bool,
    item_count: i64,
}

#[tauri::command]
pub async fn get_clips(filter_id: Option<String>, limit: i64, offset: i64, preview_only: Option<bool>, db: tauri::State<'_, Arc<Database>>) -> Result<Vec<ClipboardItem>, String> {
    let pool = &db.pool;
    let preview_only = preview_only.unwrap_or(false);

    eprintln!("get_clips called with filter_id: {:?}, preview_only: {}", filter_id, preview_only);

    let clips: Vec<Clip> = match filter_id.as_deref() {
        Some("pinned") => {
            eprintln!("Querying for pinned items");
            sqlx::query_as(r#"
                SELECT * FROM clips WHERE is_deleted = 0 AND is_pinned = 1
                ORDER BY created_at DESC LIMIT ? OFFSET ?
            "#)
            .bind(limit)
            .bind(offset)
            .fetch_all(pool).await.map_err(|e| e.to_string())?
        }
        Some(id) => {
            let folder_id_num = id.parse::<i64>().ok();
            if let Some(numeric_id) = folder_id_num {
                eprintln!("Querying for folder_id: {}", numeric_id);
                sqlx::query_as(r#"
                    SELECT * FROM clips WHERE is_deleted = 0 AND folder_id = ?
                    ORDER BY created_at DESC LIMIT ? OFFSET ?
                "#)
                .bind(numeric_id)
                .bind(limit)
                .bind(offset)
                .fetch_all(pool).await.map_err(|e| e.to_string())?
            } else {
                eprintln!("Unknown folder_id, returning empty");
                Vec::new()
            }
        }
        None => {
            eprintln!("Querying for items, offset: {}, limit: {}", offset, limit);
            sqlx::query_as(r#"
                SELECT * FROM clips WHERE is_deleted = 0
                ORDER BY created_at DESC LIMIT ? OFFSET ?
            "#)
            .bind(limit)
            .bind(offset)
            .fetch_all(pool).await.map_err(|e| e.to_string())?
        }
    };

    eprintln!("DB: Found {} clips", clips.len());

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
            eprintln!("{} Clip {}: type='{}', content_len={}", idx, clip.uuid, clip.clip_type, content_str.len());
        }

        ClipboardItem {
            id: clip.uuid.clone(),
            clip_type: clip.clip_type.clone(),
            content: content_str,
            preview: clip.text_preview.clone(),
            is_pinned: clip.is_pinned,
            folder_id: clip.folder_id.map(|id| id.to_string()),
            created_at: clip.created_at.to_rfc3339(),
            source_app: clip.source_app.clone(),
            source_icon: clip.source_icon.clone(),
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
                is_pinned: clip.is_pinned,
                folder_id: clip.folder_id.map(|id| id.to_string()),
                created_at: clip.created_at.to_rfc3339(),
                source_app: clip.source_app,
                source_icon: clip.source_icon,
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
            let clipboard = app.state::<Clipboard>();

            if clip.clip_type == "image" {
                let base64_img = BASE64.encode(&clip.content);
                clipboard.write_image_base64(base64_img).map_err(|e| e.to_string())?;
            } else {
                let content = String::from_utf8_lossy(&clip.content).to_string();
                clipboard.write_text(content).map_err(|e| e.to_string())?;
            }

            let content = String::from_utf8_lossy(&clip.content).to_string();
            let _ = window.emit("clipboard-write", &content);
            Ok(())
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
pub async fn pin_clip(id: String, db: tauri::State<'_, Arc<Database>>) -> Result<(), String> {
    let pool = &db.pool;

    sqlx::query(r#"UPDATE clips SET is_pinned = NOT is_pinned WHERE uuid = ?"#)
        .bind(&id)
        .execute(pool).await.map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn unpin_clip(id: String, db: tauri::State<'_, Arc<Database>>) -> Result<(), String> {
    let pool = &db.pool;

    sqlx::query(r#"UPDATE clips SET is_pinned = 0 WHERE uuid = ?"#)
        .bind(&id)
        .execute(pool).await.map_err(|e| e.to_string())?;
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
pub async fn create_folder(name: String, icon: Option<String>, color: Option<String>, db: tauri::State<'_, Arc<Database>>) -> Result<FolderItem, String> {
    let pool = &db.pool;

    let id = sqlx::query(r#"INSERT INTO folders (name, icon, color) VALUES (?, ?, ?)"#)
        .bind(&name)
        .bind(icon.as_ref())
        .bind(color.as_ref())
        .execute(pool).await.map_err(|e| e.to_string())?
        .last_insert_rowid();

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
pub async fn delete_folder(id: String, db: tauri::State<'_, Arc<Database>>) -> Result<(), String> {
    let pool = &db.pool;

    let folder_id: i64 = id.parse().map_err(|_| "Invalid folder ID")?;
    sqlx::query(r#"DELETE FROM folders WHERE id = ?"#)
        .bind(folder_id)
        .execute(pool).await.map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn search_clips(query: String, filter_id: Option<String>, limit: i64, db: tauri::State<'_, Arc<Database>>) -> Result<Vec<ClipboardItem>, String> {
    let pool = &db.pool;

    let search_pattern = format!("%{}%", query);

    let clips: Vec<Clip> = match filter_id.as_deref() {
        Some("pinned") => {
            sqlx::query_as(r#"
                SELECT * FROM clips WHERE is_deleted = 0 AND is_pinned = 1 AND (text_preview LIKE ? OR content LIKE ?)
                ORDER BY created_at DESC LIMIT ?
            "#)
            .bind(&search_pattern)
            .bind(&search_pattern)
            .bind(limit)
            .fetch_all(pool).await.map_err(|e| e.to_string())?
        }
        Some(id) => {
            let folder_id_num = id.parse::<i64>().ok();
            if let Some(numeric_id) = folder_id_num {
                sqlx::query_as(r#"
                    SELECT * FROM clips WHERE is_deleted = 0 AND folder_id = ? AND (text_preview LIKE ? OR content LIKE ?)
                    ORDER BY created_at DESC LIMIT ?
                "#)
                .bind(numeric_id)
                .bind(&search_pattern)
                .bind(&search_pattern)
                .bind(limit)
                .fetch_all(pool).await.map_err(|e| e.to_string())?
            } else {
                Vec::new()
            }
        }
        None => {
            sqlx::query_as(r#"
                SELECT * FROM clips WHERE is_deleted = 0 AND (text_preview LIKE ? OR content LIKE ?)
                ORDER BY created_at DESC LIMIT ?
            "#)
            .bind(&search_pattern)
            .bind(&search_pattern)
            .bind(limit)
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
            is_pinned: clip.is_pinned,
            folder_id: clip.folder_id.map(|id| id.to_string()),
            created_at: clip.created_at.to_rfc3339(),
            source_app: clip.source_app.clone(),
            source_icon: clip.source_icon.clone(),
        }
    }).collect();

    Ok(items)
}

#[tauri::command]
pub async fn get_folders(db: tauri::State<'_, Arc<Database>>) -> Result<Vec<FolderItem>, String> {
    let pool = &db.pool;

    let folders: Vec<Folder> = sqlx::query_as(r#"SELECT * FROM folders ORDER BY name"#)
        .fetch_all(pool).await.map_err(|e| e.to_string())?;

    let items: Vec<FolderItem> = folders.iter().map(|folder| {
        FolderItem {
            id: folder.id.to_string(),
            name: folder.name.clone(),
            icon: folder.icon.clone(),
            color: folder.color.clone(),
            is_system: folder.is_system,
            item_count: 0,
        }
    }).collect();

    Ok(items)
}

#[tauri::command]
pub async fn get_settings(db: tauri::State<'_, Arc<Database>>) -> Result<serde_json::Value, String> {
    let pool = &db.pool;

    let mut settings = serde_json::json!({
        "max_items": 1000,
        "auto_delete_days": 30,
        "startup_with_windows": true,
        "show_in_taskbar": false,
        "hotkey": "Ctrl+Alt+V",
        "theme": "dark",
    });

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

    Ok(settings)
}

#[tauri::command]
pub async fn save_settings(settings: serde_json::Value, db: tauri::State<'_, Arc<Database>>) -> Result<(), String> {
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
    let app = window.app_handle();

    let shortcut = Shortcut::from_str(&hotkey).map_err(|e| format!("Invalid hotkey: {:?}", e))?;

    if let Err(e) = app.global_shortcut().register(shortcut) {
        return Err(format!("Failed to register hotkey: {:?}", e));
    }

    Ok(())
}

#[tauri::command]
pub fn show_window(window: tauri::WebviewWindow) -> Result<(), String> {
    crate::position_window_at_bottom(&window);
    if let Err(e) = window.show() {
        return Err(format!("Failed to show window: {:?}", e));
    }
    if let Err(e) = window.set_focus() {
        return Err(format!("Failed to focus window: {:?}", e));
    }
    Ok(())
}

#[tauri::command]
pub fn get_layout_config() -> serde_json::Value {
    serde_json::json!({
        "window_height": crate::constants::WINDOW_HEIGHT,
    })
}
