use tauri::Emitter;
use crate::database::Database;
use crate::models::{ClipboardItem, FolderItem, get_db_path};

#[tauri::command]
pub fn get_clips(
    folder_id: Option<String>,
    limit: i64,
    offset: i64,
) -> Result<Vec<ClipboardItem>, String> {
    let db_path = get_db_path();
    let rt = tokio::runtime::Runtime::new().map_err(|e| e.to_string())?;
    let db = rt.block_on(async { Database::new(db_path).await });

    let folder_id = match folder_id {
        Some(id) => Some(id.parse::<i64>().map_err(|_| "Invalid folder ID")?),
        None => None,
    };

    let clips = rt.block_on(async {
        db.get_clips(folder_id, limit, offset).await
    }).map_err(|e| e.to_string())?;

    let items: Vec<ClipboardItem> = clips.iter().map(|clip| {
        let content_str = String::from_utf8_lossy(&clip.content).to_string();

        ClipboardItem {
            id: clip.uuid.clone(),
            clip_type: clip.clip_type.clone(),
            content: content_str.clone(),
            preview: clip.text_preview.clone(),
            is_pinned: clip.is_pinned,
            folder_id: clip.folder_id.map(|id| id.to_string()),
            created_at: clip.created_at.to_rfc3339(),
            source_app: clip.source_app.clone(),
        }
    }).collect();

    Ok(items)
}

#[tauri::command]
pub fn get_clip(
    id: String,
) -> Result<ClipboardItem, String> {
    let db_path = get_db_path();
    let rt = tokio::runtime::Runtime::new().map_err(|e| e.to_string())?;
    let db = rt.block_on(async { Database::new(db_path).await });

    let clip = rt.block_on(async {
        db.get_clip_by_uuid(&id).await
    }).map_err(|e| e.to_string())?;

    match clip {
        Some(clip) => {
            let content_str = String::from_utf8_lossy(&clip.content).to_string();

            Ok(ClipboardItem {
                id: clip.uuid,
                clip_type: clip.clip_type,
                content: content_str,
                preview: clip.text_preview,
                is_pinned: clip.is_pinned,
                folder_id: clip.folder_id.map(|id| id.to_string()),
                created_at: clip.created_at.to_rfc3339(),
                source_app: clip.source_app,
            })
        }
        None => Err("Clip not found".to_string()),
    }
}

#[tauri::command]
pub fn paste_clip(
    id: String,
    window: tauri::Window,
) -> Result<(), String> {
    let db_path = get_db_path();
    let rt = tokio::runtime::Runtime::new().map_err(|e| e.to_string())?;
    let db = rt.block_on(async { Database::new(db_path).await });

    let clip = rt.block_on(async {
        db.get_clip_by_uuid(&id).await
    }).map_err(|e| e.to_string())?;

    match clip {
        Some(clip) => {
            let content = String::from_utf8_lossy(&clip.content).to_string();
            let _ = window.emit("clipboard-write", &content);
            Ok(())
        }
        None => Err("Clip not found".to_string()),
    }
}

#[tauri::command]
pub fn delete_clip(
    id: String,
    hard_delete: bool,
) -> Result<(), String> {
    let db_path = get_db_path();
    let rt = tokio::runtime::Runtime::new().map_err(|e| e.to_string())?;
    let db = rt.block_on(async { Database::new(db_path).await });

    if hard_delete {
        let clip = rt.block_on(async {
            db.get_clip_by_uuid(&id).await
        }).map_err(|e| e.to_string())?;
        if let Some(clip) = clip {
            rt.block_on(async {
                db.hard_delete_clip(clip.id).await
            }).map_err(|e| e.to_string())?;
        }
    } else {
        let clip = rt.block_on(async {
            db.get_clip_by_uuid(&id).await
        }).map_err(|e| e.to_string())?;
        if let Some(clip) = clip {
            rt.block_on(async {
                db.delete_clip(clip.id).await
            }).map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}

#[tauri::command]
pub fn pin_clip(
    id: String,
) -> Result<(), String> {
    let db_path = get_db_path();
    let rt = tokio::runtime::Runtime::new().map_err(|e| e.to_string())?;
    let db = rt.block_on(async { Database::new(db_path).await });

    let clip = rt.block_on(async {
        db.get_clip_by_uuid(&id).await
    }).map_err(|e| e.to_string())?;
    match clip {
        Some(clip) => {
            rt.block_on(async {
                db.pin_clip(clip.id).await
            }).map_err(|e| e.to_string())?;
            Ok(())
        }
        None => Err("Clip not found".to_string()),
    }
}

#[tauri::command]
pub fn unpin_clip(
    id: String,
) -> Result<(), String> {
    let db_path = get_db_path();
    let rt = tokio::runtime::Runtime::new().map_err(|e| e.to_string())?;
    let db = rt.block_on(async { Database::new(db_path).await });

    let clip = rt.block_on(async {
        db.get_clip_by_uuid(&id).await
    }).map_err(|e| e.to_string())?;
    match clip {
        Some(clip) => {
            rt.block_on(async {
                db.unpin_clip(clip.id).await
            }).map_err(|e| e.to_string())?;
            Ok(())
        }
        None => Err("Clip not found".to_string()),
    }
}

#[tauri::command]
pub fn move_to_folder(
    clip_id: String,
    folder_id: Option<String>,
) -> Result<(), String> {
    let db_path = get_db_path();
    let rt = tokio::runtime::Runtime::new().map_err(|e| e.to_string())?;
    let db = rt.block_on(async { Database::new(db_path).await });

    let folder_id = match folder_id {
        Some(id) => Some(id.parse::<i64>().map_err(|_| "Invalid folder ID")?),
        None => None,
    };

    let clip = rt.block_on(async {
        db.get_clip_by_uuid(&clip_id).await
    }).map_err(|e| e.to_string())?;
    match clip {
        Some(clip) => {
            rt.block_on(async {
                db.update_clip_folder(clip.id, folder_id).await
            }).map_err(|e| e.to_string())?;
            Ok(())
        }
        None => Err("Clip not found".to_string()),
    }
}

#[tauri::command]
pub fn create_folder(
    name: String,
    icon: Option<String>,
    color: Option<String>,
) -> Result<FolderItem, String> {
    let db_path = get_db_path();
    let rt = tokio::runtime::Runtime::new().map_err(|e| e.to_string())?;
    let db = rt.block_on(async { Database::new(db_path).await });

    let folder_id = rt.block_on(async {
        db.create_folder(&name, icon.as_deref(), color.as_deref()).await
    }).map_err(|e| e.to_string())?;

    Ok(FolderItem {
        id: folder_id.to_string(),
        name,
        icon,
        color,
        is_system: false,
        item_count: 0,
    })
}

#[tauri::command]
pub fn delete_folder(
    id: String,
) -> Result<(), String> {
    let db_path = get_db_path();
    let rt = tokio::runtime::Runtime::new().map_err(|e| e.to_string())?;
    let db = rt.block_on(async { Database::new(db_path).await });

    let folder_id: i64 = id.parse().map_err(|_| "Invalid folder ID")?;
    rt.block_on(async {
        db.delete_folder(folder_id).await
    }).map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub fn search_clips(
    query: String,
    limit: i64,
) -> Result<Vec<ClipboardItem>, String> {
    let db_path = get_db_path();
    let rt = tokio::runtime::Runtime::new().map_err(|e| e.to_string())?;
    let db = rt.block_on(async { Database::new(db_path).await });

    let clips = rt.block_on(async {
        db.search_clips(&query, limit).await
    }).map_err(|e| e.to_string())?;

    let items: Vec<ClipboardItem> = clips.iter().map(|clip| {
        let content_str = String::from_utf8_lossy(&clip.content).to_string();

        ClipboardItem {
            id: clip.uuid.clone(),
            clip_type: clip.clip_type.clone(),
            content: content_str.clone(),
            preview: clip.text_preview.clone(),
            is_pinned: clip.is_pinned,
            folder_id: clip.folder_id.map(|id| id.to_string()),
            created_at: clip.created_at.to_rfc3339(),
            source_app: clip.source_app.clone(),
        }
    }).collect();

    Ok(items)
}

#[tauri::command]
pub fn get_folders() -> Result<Vec<FolderItem>, String> {
    let db_path = get_db_path();
    let rt = tokio::runtime::Runtime::new().map_err(|e| e.to_string())?;
    let db = rt.block_on(async { Database::new(db_path).await });

    let folders = rt.block_on(async {
        db.get_folders().await
    }).map_err(|e| e.to_string())?;

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
pub fn get_settings() -> Result<super::models::Settings, String> {
    let db_path = get_db_path();
    let rt = tokio::runtime::Runtime::new().map_err(|e| e.to_string())?;
    let db = rt.block_on(async { Database::new(db_path).await });

    let mut settings = super::models::Settings::default();

    if let Ok(Some(value)) = rt.block_on(async {
        db.get_setting("max_items").await
    }) {
        settings.max_items = value.parse().unwrap_or(1000);
    }

    if let Ok(Some(value)) = rt.block_on(async {
        db.get_setting("auto_delete_days").await
    }) {
        settings.auto_delete_days = value.parse().unwrap_or(30);
    }

    if let Ok(Some(value)) = rt.block_on(async {
        db.get_setting("theme").await
    }) {
        settings.theme = value;
    }

    Ok(settings)
}

#[tauri::command]
pub fn save_settings(
    settings: super::models::Settings,
) -> Result<(), String> {
    let db_path = get_db_path();
    let rt = tokio::runtime::Runtime::new().map_err(|e| e.to_string())?;
    let db = rt.block_on(async { Database::new(db_path).await });

    rt.block_on(async {
        db.set_setting("max_items", &settings.max_items.to_string()).await
    }).map_err(|e| e.to_string())?;
    rt.block_on(async {
        db.set_setting("auto_delete_days", &settings.auto_delete_days.to_string()).await
    }).map_err(|e| e.to_string())?;
    rt.block_on(async {
        db.set_setting("theme", &settings.theme).await
    }).map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub fn hide_window(window: tauri::Window) -> Result<(), String> {
    window.hide().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_clipboard_history_size() -> Result<i64, String> {
    let db_path = get_db_path();
    let rt = tokio::runtime::Runtime::new().map_err(|e| e.to_string())?;
    let db = rt.block_on(async { Database::new(db_path).await });

    let size = rt.block_on(async {
        db.get_clipboard_history_size().await
    }).map_err(|e| e.to_string())?;
    Ok(size)
}

#[tauri::command]
pub fn clear_clipboard_history() -> Result<(), String> {
    let db_path = get_db_path();
    let rt = tokio::runtime::Runtime::new().map_err(|e| e.to_string())?;
    let db = rt.block_on(async { Database::new(db_path).await });

    rt.block_on(async {
        db.clear_history().await
    }).map_err(|e| e.to_string())?;
    Ok(())
}
