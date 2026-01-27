
use tauri::{AppHandle, Manager, Listener, Emitter};
use tauri_plugin_clipboard::Clipboard as ClipboardPlugin;
use std::sync::Arc;
use crate::database::Database;
use uuid::Uuid;
use sha2::{Digest, Sha256};
use std::io::Cursor;
use image::{ImageOutputFormat, GenericImageView, DynamicImage};
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use windows::Win32::Foundation::{HWND, MAX_PATH};
use windows::Win32::System::Threading::{OpenProcess, PROCESS_QUERY_INFORMATION, PROCESS_VM_READ};
use windows::Win32::System::ProcessStatus::GetModuleBaseNameW;
use windows::Win32::UI::WindowsAndMessaging::{GetForegroundWindow, GetWindowThreadProcessId};

pub fn init(app: &AppHandle, db: Arc<Database>) {
    let app_clone = app.clone();
    let db_clone = db.clone();

    // Attempt to start the monitor
    if let Some(plugin) = app.try_state::<ClipboardPlugin>() {
         if let Err(e) = plugin.start_monitor(app.clone()) {
             eprintln!("CLIPBOARD: Failed to start monitor: {}", e);
         }
    } else {
        eprintln!("CLIPBOARD: Plugin state not found");
    }

    // Listen to the generic update event from the plugin
    let event_name = "plugin:clipboard://clipboard-monitor/update";

    app.listen(event_name, move |_event| {
        let app = app_clone.clone();
        let db = db_clone.clone();

        tauri::async_runtime::spawn(async move {
            process_clipboard_change(app, db).await;
        });
    });
}

async fn process_clipboard_change(app: AppHandle, db: Arc<Database>) {
    // Initialize Clipboard struct
    let clipboard = app.state::<ClipboardPlugin>();

    // capture source app immediately
    let source_app = get_active_app_name();

    let mut clip_type = "text";
    let mut clip_content = Vec::new();
    let mut clip_preview = String::new();
    let mut clip_hash = String::new();
    let mut metadata = String::new();
    let mut found_content = false;

    // Try Image first using base64
    if let Ok(base64_image) = clipboard.read_image_base64() {
         if let Ok(bytes) = BASE64.decode(base64_image) {
             if let Ok(image) = image::load_from_memory(&bytes) {
                 let width = image.width();
                 let height = image.height();

                 let size_bytes = bytes.len();
                 clip_hash = calculate_hash(&bytes);
                 clip_content = bytes;
                 clip_type = "image";
                 clip_preview = "[Image]".to_string();
                 metadata = serde_json::json!({
                     "width": width,
                     "height": height,
                     "format": "png",
                     "size_bytes": size_bytes
                 }).to_string();
                 found_content = true;
             }
         }
    }

    if !found_content {
        // Try Text
        if let Ok(text) = clipboard.read_text() {
             let text: String = text; // Force type
             let text = text.trim();
             if !text.is_empty() {
                 clip_content = text.as_bytes().to_vec();
                 clip_hash = calculate_hash(&clip_content);
                 clip_type = "text";
                 clip_preview = text.chars().take(200).collect::<String>();
                 found_content = true;
             }
        }
    }

    if !found_content {
        return;
    }

    // DB Logic
    let pool = &db.pool;

    // Check if exists
    let existing_uuid: Option<String> = sqlx::query_scalar::<_, String>(r#"SELECT uuid FROM clips WHERE content_hash = ?"#)
        .bind(&clip_hash)
        .fetch_optional(pool)
        .await
        .unwrap_or(None);

    if let Some(existing_id) = existing_uuid {
        let _ = sqlx::query(r#"UPDATE clips SET created_at = CURRENT_TIMESTAMP, is_deleted = 0 WHERE uuid = ?"#)
            .bind(&existing_id)
            .execute(pool)
            .await;

        let _ = app.emit("clipboard-change", &serde_json::json!({
            "id": existing_id,
            "content": clip_preview,
            "clip_type": clip_type,
            "is_pinned": false,
            "created_at": chrono::Utc::now().to_rfc3339()
        }));
    } else {
        let clip_uuid = Uuid::new_v4().to_string();

        let _ = sqlx::query(r#"
            INSERT INTO clips (uuid, clip_type, content, text_preview, content_hash, folder_id, is_pinned, is_deleted, source_app, metadata, created_at, last_accessed)
            VALUES (?, ?, ?, ?, ?, NULL, 0, 0, ?, ?, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
        "#)
        .bind(&clip_uuid)
        .bind(clip_type)
        .bind(&clip_content)
        .bind(&clip_preview)
        .bind(&clip_hash)
        .bind(&source_app)
        .bind(if clip_type == "image" { Some(metadata) } else { None })
        .execute(pool)
        .await;

        let _ = app.emit("clipboard-change", &serde_json::json!({
            "id": clip_uuid,
            "content": clip_preview,
            "clip_type": clip_type,
            "is_pinned": false,
            "created_at": chrono::Utc::now().to_rfc3339()
        }));
    }
}

fn calculate_hash(content: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content);
    let result = hasher.finalize();
    format!("{:x}", result)
}

fn get_active_app_name() -> Option<String> {
    unsafe {
        let hwnd = GetForegroundWindow();
        if hwnd.0.is_null() {
            return None;
        }

        let mut process_id = 0;
        GetWindowThreadProcessId(hwnd, Some(&mut process_id));

        if process_id == 0 {
            return None;
        }

        let process_handle = OpenProcess(PROCESS_QUERY_INFORMATION | PROCESS_VM_READ, false, process_id).ok()?;

        let mut buffer = [0u16; MAX_PATH as usize];
        let size = GetModuleBaseNameW(process_handle, None, &mut buffer);

        if size == 0 {
            return None;
        }

        let name = String::from_utf16_lossy(&buffer[..size as usize]);
        Some(name)
    }
}
