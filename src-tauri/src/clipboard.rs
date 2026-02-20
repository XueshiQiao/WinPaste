use tauri::{AppHandle, Emitter, Listener};
// Import functions directly from the crate root
use crate::database::Database;
use clipboard_rs::common::RustImage;
use clipboard_rs::{Clipboard, ClipboardContext};
#[cfg(target_os = "windows")]
use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use once_cell::sync::Lazy;
use sha2::{Digest, Sha256};
#[cfg(target_os = "windows")]
use std::ffi::OsStr;
#[cfg(target_os = "windows")]
use std::os::windows::ffi::OsStrExt;
use std::sync::Arc;
use tauri_plugin_clipboard_x::{read_text, start_listening};
use uuid::Uuid;
#[cfg(target_os = "windows")]
use windows::Win32::Foundation::MAX_PATH;
#[cfg(target_os = "windows")]
use windows::Win32::Graphics::Gdi::{
    CreateCompatibleBitmap, CreateCompatibleDC, DeleteDC, DeleteObject, GetDC, GetDIBits,
    GetObjectW, ReleaseDC, SelectObject, BITMAP, BITMAPINFO, BITMAPINFOHEADER, BI_RGB,
    DIB_RGB_COLORS, HBITMAP,
};
#[cfg(target_os = "windows")]
use windows::Win32::Storage::FileSystem::{
    GetFileVersionInfoSizeW, GetFileVersionInfoW, VerQueryValueW,
};
#[cfg(target_os = "windows")]
use windows::Win32::System::DataExchange::GetClipboardOwner;
#[cfg(target_os = "windows")]
use windows::Win32::System::ProcessStatus::{GetModuleBaseNameW, GetModuleFileNameExW};
#[cfg(target_os = "windows")]
use windows::Win32::System::Threading::{OpenProcess, PROCESS_QUERY_INFORMATION, PROCESS_VM_READ};
#[cfg(target_os = "windows")]
use windows::Win32::UI::Input::KeyboardAndMouse::{
    SendInput, INPUT, INPUT_KEYBOARD, KEYBDINPUT, KEYEVENTF_KEYUP, VK_INSERT, VK_SHIFT,
};
#[cfg(target_os = "windows")]
use windows::Win32::UI::Shell::{
    SHGetFileInfoW, SHFILEINFOW, SHGFI_ICON, SHGFI_LARGEICON, SHGFI_USEFILEATTRIBUTES,
};
#[cfg(target_os = "windows")]
use windows::Win32::UI::WindowsAndMessaging::{
    DestroyIcon, DrawIconEx, GetForegroundWindow, GetIconInfo, GetWindowThreadProcessId, DI_NORMAL,
    ICONINFO,
};

// GLOBAL STATE: Store the hash of the clip we just pasted ourselves.
// If the next clipboard change matches this hash, we ignore it (don't update timestamp).
static IGNORE_HASH: Lazy<parking_lot::Mutex<Option<String>>> =
    Lazy::new(|| parking_lot::Mutex::new(None));
static LAST_STABLE_HASH: Lazy<parking_lot::Mutex<Option<String>>> =
    Lazy::new(|| parking_lot::Mutex::new(None));
pub static CLIPBOARD_SYNC: Lazy<Arc<tokio::sync::Mutex<()>>> =
    Lazy::new(|| Arc::new(tokio::sync::Mutex::new(())));

use std::sync::atomic::{AtomicU64, Ordering};
static DEBOUNCE_COUNTER: AtomicU64 = AtomicU64::new(0);

pub fn set_ignore_hash(hash: String) {
    let mut lock = IGNORE_HASH.lock();
    *lock = Some(hash);
}

pub fn set_last_stable_hash(hash: String) {
    let mut lock = LAST_STABLE_HASH.lock();
    *lock = Some(hash);
}

pub fn init(app: &AppHandle, db: Arc<Database>) {
    let app_clone = app.clone();
    let db_clone = db.clone();

    // Start monitor
    // tauri-plugin-clipboard-x exposes start_listening(app_handle)
    // It returns impl Future, so we need to spawn it or block.
    // Since init is synchronous here, we spawn it.
    let app_for_start = app.clone();
    tauri::async_runtime::spawn(async move {
        if let Err(e) = start_listening(app_for_start).await {
            log::error!("CLIPBOARD: Failed to start listener: {}", e);
        }
    });

    // Listen to clipboard changes
    // The event name found in source code: "plugin:clipboard-x://clipboard_changed"
    let event_name = "plugin:clipboard-x://clipboard_changed";

    app.listen(event_name, move |_event| {
        let app = app_clone.clone();
        let db = db_clone.clone();

        // Capture source app info IMMEDIATELY at event time, before debounce delay.
        // If we wait until after the delay, the user may have already switched to PastePaw,
        // causing frontmostApplication to return our own app instead of the real source.
        let source_app_info = get_clipboard_owner_app_info();

        // DEBOUNCE LOGIC:
        let current_count = DEBOUNCE_COUNTER.fetch_add(1, Ordering::SeqCst) + 1;

        tauri::async_runtime::spawn(async move {
            tokio::time::sleep(std::time::Duration::from_millis(150)).await;

            if DEBOUNCE_COUNTER.load(Ordering::SeqCst) != current_count {
                log::debug!(
                    "CLIPBOARD: Debounce: Aborting older event, current_count:{}",
                    current_count
                );
                return;
            }

            process_clipboard_change(app, db, source_app_info).await;
        });
    });
}

type SourceAppInfo = (
    Option<String>,
    Option<String>,
    Option<String>,
    Option<String>,
    bool,
);

struct ClipboardImageRead {
    png_bytes: Vec<u8>,
    width: u32,
    height: u32,
    raw_hash: String,
    decode_ms: u128,
    source_type: &'static str,
}

fn image_dimensions_from_bytes(bytes: &[u8]) -> Result<(u32, u32), String> {
    use image::io::Reader as ImageReader;
    use std::io::Cursor;

    let reader = ImageReader::new(Cursor::new(bytes))
        .with_guessed_format()
        .map_err(|e| e.to_string())?;
    reader.into_dimensions().map_err(|e| e.to_string())
}

fn read_clipboard_image_with_clipboard_rs(
    source_type: &'static str,
) -> Result<ClipboardImageRead, String> {
    let ctx = ClipboardContext::new().map_err(|e| e.to_string())?;
    let image = ctx.get_image().map_err(|e| e.to_string())?;
    let (width, height) = image.get_size();

    let dynamic_image = image.get_dynamic_image().map_err(|e| e.to_string())?;
    let raw_hash = calculate_hash(dynamic_image.as_bytes());

    let png_bytes = image
        .to_png()
        .map_err(|e| e.to_string())?
        .get_bytes()
        .to_vec();

    Ok(ClipboardImageRead {
        png_bytes,
        width,
        height,
        raw_hash,
        decode_ms: 0,
        source_type,
    })
}

#[cfg(target_os = "macos")]
fn read_clipboard_image_fast() -> Result<ClipboardImageRead, String> {
    use cocoa::base::{id, nil};
    use cocoa::foundation::NSString;
    use image::codecs::png::PngEncoder;
    use image::{ColorType, ImageEncoder};
    use objc::{class, msg_send, sel, sel_impl};
    use std::ffi::CStr;

    fn decode_encoded_image_to_png(encoded: &[u8]) -> Result<(Vec<u8>, u32, u32), String> {
        let img = image::load_from_memory(encoded).map_err(|e| e.to_string())?;
        let (width, height) = (img.width(), img.height());
        let rgba = img.to_rgba8();
        let mut png_bytes = Vec::new();
        PngEncoder::new(&mut png_bytes)
            .write_image(
                rgba.as_raw(),
                rgba.width(),
                rgba.height(),
                ColorType::Rgba8,
            )
            .map_err(|e| e.to_string())?;
        Ok((png_bytes, width, height))
    }

    unsafe fn nsstring_to_string(ns: id) -> Option<String> {
        if ns == nil {
            return None;
        }
        let utf8: *const std::os::raw::c_char = msg_send![ns, UTF8String];
        if utf8.is_null() {
            return None;
        }
        Some(CStr::from_ptr(utf8).to_string_lossy().into_owned())
    }

    unsafe fn data_for_type(pasteboard: id, uti: &str) -> Option<Vec<u8>> {
        let ns_type = NSString::alloc(nil).init_str(uti);
        let data: id = msg_send![pasteboard, dataForType:ns_type];
        if data == nil {
            return None;
        }

        let len: usize = msg_send![data, length];
        if len == 0 {
            return None;
        }
        let ptr: *const u8 = msg_send![data, bytes];
        if ptr.is_null() {
            return None;
        }

        Some(std::slice::from_raw_parts(ptr, len).to_vec())
    }

    unsafe fn read_image_from_file_url_type(pasteboard: id) -> Option<Vec<u8>> {
        let file_url_type = NSString::alloc(nil).init_str("public.file-url");
        let url_string: id = msg_send![pasteboard, stringForType:file_url_type];
        let url_string = nsstring_to_string(url_string)?;
        if url_string.is_empty() {
            return None;
        }
        let ns_url_string = NSString::alloc(nil).init_str(&url_string);
        let nsurl: id = msg_send![class!(NSURL), URLWithString:ns_url_string];
        if nsurl == nil {
            return None;
        }
        let ns_path: id = msg_send![nsurl, path];
        let path = nsstring_to_string(ns_path)?;
        std::fs::read(path).ok()
    }

    unsafe {
        let pasteboard: id = msg_send![class!(NSPasteboard), generalPasteboard];
        let preferred_types: [&str; 13] = [
            "public.png",
            "public.jpeg",
            "public.jpg",
            "org.webmproject.webp",
            "public.webp",
            "public.heic",
            "public.heif",
            "public.jpeg-xl",
            "public.gif",
            "com.compuserve.gif",
            "public.bmp",
            "com.microsoft.bmp",
            "public.tiff",
        ];

        for selected_type in preferred_types {
            if let Some(encoded_bytes) = data_for_type(pasteboard, selected_type) {
                if selected_type == "public.png" {
                    let (width, height) = image_dimensions_from_bytes(&encoded_bytes)?;
                    log::debug!("find public.png directly, width:{}, height:{}", width, height);
                    return Ok(ClipboardImageRead {
                        raw_hash: calculate_hash(&encoded_bytes),
                        png_bytes: encoded_bytes,
                        width,
                        height,
                        decode_ms: 0,
                        source_type: "public.png",
                    });
                }

                let decode_started = std::time::Instant::now();
                log::debug!("try re-encode {} to png", selected_type);
                if let Ok((png_bytes, width, height)) = decode_encoded_image_to_png(&encoded_bytes)
                {
                    let decode_ms = decode_started.elapsed().as_millis();

                    log::debug!("re-encode {} successfully, takes {} ms", selected_type, decode_ms);
                    return Ok(ClipboardImageRead {
                        raw_hash: calculate_hash(&encoded_bytes),
                        png_bytes,
                        width,
                        height,
                        decode_ms,
                        source_type: selected_type,
                    });
                } else {
                    log::debug!("re-encode {} failed", selected_type);
                }
            }
        }

        log::debug!("No suitable image found, trying file URL");
        // Some apps provide image as file URL only.
        if let Some(file_bytes) = read_image_from_file_url_type(pasteboard) {
            log::debug!("find public.file-url, will decode");
            let decode_started = std::time::Instant::now();
            let (png_bytes, width, height) = decode_encoded_image_to_png(&file_bytes)?;
            let decode_ms = decode_started.elapsed().as_millis();
            log::debug!("decode public.file-url successfully, takes {} ms", decode_ms);
            return Ok(ClipboardImageRead {
                raw_hash: calculate_hash(&file_bytes),
                png_bytes,
                width,
                height,
                decode_ms,
                source_type: "public.file-url",
            });
        } else {
            log::debug!("No public.file-url found");
        }
    }

    // Last-resort safety net to avoid capture regressions for unusual pasteboard providers.
    // This path goes through clipboard-rs image conversion and can be significantly slower.
    log::warn!(
        "CLIPBOARD: Falling back to clipboard-rs get_image slow path; this may be noticeably slow"
    );
    if let Ok(result) = read_clipboard_image_with_clipboard_rs("clipboard-rs-image-fallback") {
        return Ok(result);
    }

    Err(
        "No supported image content in macOS pasteboard (png/jpeg/webp/heic/heif/gif/bmp/tiff/file-url)".to_string(),
    )
}

#[cfg(not(target_os = "macos"))]
fn read_clipboard_image_fast() -> Result<ClipboardImageRead, String> {
    read_clipboard_image_with_clipboard_rs("clipboard-rs-image")
}

async fn process_clipboard_change(
    app: AppHandle,
    db: Arc<Database>,
    source_app_info: SourceAppInfo,
) {
    let started = std::time::Instant::now();
    let mut image_read_ms = 0u128;
    let mut image_decode_ms = 0u128;
    let mut image_thumbnail_ms = 0u128;
    let mut text_read_ms = 0u128;
    let mut was_existing = false;
    let _guard = CLIPBOARD_SYNC.lock().await;

    let mut clip_type = "text";
    let mut clip_content = Vec::new();
    let mut full_image_content: Option<Vec<u8>> = None;
    let mut clip_preview = String::new();
    let mut clip_hash = String::new();
    let mut metadata = String::new();
    let mut found_content = false;

    // Try Image (in-memory path, no temp file write).
    log::debug!("CLIPBOARD: Attempting to read image from clipboard");
    let image_read_started = std::time::Instant::now();
    if let Ok(read_image_result) = read_clipboard_image_fast() {

        image_read_ms = image_read_started.elapsed().as_millis();
        log::debug!(
            "CLIPBOARD: Image read successfully, source_type={}, takes {} ms",
            read_image_result.source_type,
            image_read_ms
        );

        let bytes = read_image_result.png_bytes;
        let width = read_image_result.width;
        let height = read_image_result.height;
        image_decode_ms = read_image_result.decode_ms;
        let size_bytes = bytes.len();
        let thumbnail_started = std::time::Instant::now();
        let thumbnail = create_image_thumbnail(&bytes, 256).unwrap_or_else(|_| bytes.clone());
        image_thumbnail_ms = thumbnail_started.elapsed().as_millis();

        clip_hash = read_image_result.raw_hash;
        clip_content = thumbnail.clone();
        full_image_content = Some(bytes);
        clip_type = "image";
        clip_preview = "[Image]".to_string();
        metadata = serde_json::json!({
            "width": width,
            "height": height,
            "format": "png",
            "size_bytes": size_bytes,
            "thumbnail_size_bytes": thumbnail.len()
        })
        .to_string();
        found_content = true;
        log::debug!(
            "CLIPBOARD: Found image: {}x{}, source_type={}, png_bytes={}, thumb_bytes={}",
            width,
            height,
            read_image_result.source_type,
            size_bytes,
            thumbnail.len()
        );
    }

    if !found_content {
        // Try Text
        let text_read_started = std::time::Instant::now();
        if let Ok(text) = read_text().await {
            text_read_ms = text_read_started.elapsed().as_millis();
            let text = text.trim();
            if !text.is_empty() {
                clip_content = text.as_bytes().to_vec();
                clip_hash = calculate_hash(&clip_content);
                clip_type = "text";
                clip_preview = text.chars().take(200).collect::<String>();
                found_content = true;
                log::debug!("CLIPBOARD: Found text: {}", clip_preview);
            }
        }
    }

    if !found_content {
        return;
    }

    // Stable Hash Check
    {
        let mut lock = LAST_STABLE_HASH.lock();
        if let Some(ref last_hash) = *lock {
            if last_hash == &clip_hash {
                return;
            }
        }
        *lock = Some(clip_hash.clone());
    }

    // Check ignore self-paste
    {
        let mut lock = IGNORE_HASH.lock();
        if let Some(ignore_hash) = lock.take() {
            if ignore_hash == clip_hash {
                log::info!(
                    "CLIPBOARD: Detected self-paste for hash {}, proceeding to update timestamp",
                    ignore_hash
                );
            }
        }
    }

    // Source app info was captured at event time (before debounce) to avoid race conditions
    let (source_app, source_icon, exe_name, full_path, is_explicit_owner) = source_app_info;
    log::info!(
        "CLIPBOARD: Source app: {:?}, exe_name: {:?}, full_path: {:?}, explicit: {}",
        source_app,
        exe_name,
        full_path,
        is_explicit_owner
    );

    // Check settings (cached via SettingsManager)
    use crate::settings_manager::SettingsManager;
    use tauri::Manager;
    let manager = app.state::<Arc<SettingsManager>>();
    let settings = manager.get();

    if settings.ignore_ghost_clips && !is_explicit_owner {
        log::info!("CLIPBOARD: Ignoring ghost clip (unknown owner)");
        return;
    }

    // Check if the app is in the ignore list (Case Insensitive)
    let is_ignored = |name: &str| {
        let name_lower = name.to_lowercase();
        settings
            .ignored_apps
            .iter()
            .any(|app| app.to_lowercase() == name_lower)
    };

    if let Some(ref path) = full_path {
        if is_ignored(path) {
            log::info!(
                "CLIPBOARD: Ignoring content from ignored app (path match): {}",
                path
            );
            return;
        }
    }

    if let Some(ref exe) = exe_name {
        if is_ignored(exe) {
            log::info!(
                "CLIPBOARD: Ignoring content from ignored app (exe match): {}",
                exe
            );
            return;
        }
    }

    // DB Logic
    let pool = &db.pool;

    let db_lookup_started = std::time::Instant::now();
    let existing_uuid: Option<String> =
        sqlx::query_scalar::<_, String>(r#"SELECT uuid FROM clips WHERE content_hash = ?"#)
            .bind(&clip_hash)
            .fetch_optional(pool)
            .await
            .unwrap_or(None);
    let db_lookup_ms = db_lookup_started.elapsed().as_millis();

    let db_write_started = std::time::Instant::now();
    let emitted_id = if let Some(existing_id) = existing_uuid {
        was_existing = true;
        if clip_type == "image" {
            let _ = sqlx::query(
                r#"
                UPDATE clips
                SET created_at = CURRENT_TIMESTAMP,
                    is_deleted = 0,
                    source_app = ?,
                    source_icon = ?,
                    content = ?,
                    text_preview = ?,
                    metadata = ?,
                    is_thumbnail = 1
                WHERE uuid = ?
                "#,
            )
            .bind(&source_app)
            .bind(&source_icon)
            .bind(&clip_content)
            .bind(&clip_preview)
            .bind(Some(metadata.clone()))
            .bind(&existing_id)
            .execute(pool)
            .await;

            if let Some(full_bytes) = &full_image_content {
                match persist_full_image_file(&existing_id, full_bytes) {
                    Ok(file_path) => {
                        let _ = sqlx::query(
                            r#"
                            INSERT OR REPLACE INTO clip_images (clip_uuid, full_content, file_path, file_size, storage_kind, mime_type, created_at)
                            VALUES (?, x'', ?, ?, 'file', 'image/png', CURRENT_TIMESTAMP)
                            "#,
                        )
                        .bind(&existing_id)
                        .bind(&file_path)
                        .bind(full_bytes.len() as i64)
                        .execute(pool)
                        .await;
                    }
                    Err(e) => {
                        log::warn!("Failed to persist full image file, fallback to DB blob: {}", e);
                        let _ = sqlx::query(
                            r#"
                            INSERT OR REPLACE INTO clip_images (clip_uuid, full_content, file_path, file_size, storage_kind, mime_type, created_at)
                            VALUES (?, ?, NULL, ?, 'db', 'image/png', CURRENT_TIMESTAMP)
                            "#,
                        )
                        .bind(&existing_id)
                        .bind(full_bytes)
                        .bind(full_bytes.len() as i64)
                        .execute(pool)
                        .await;
                    }
                }
            }
        } else {
            let _ = sqlx::query(r#"UPDATE clips SET created_at = CURRENT_TIMESTAMP, is_deleted = 0, source_app = ?, source_icon = ? WHERE uuid = ?"#)
                .bind(&source_app)
                .bind(&source_icon)
                .bind(&existing_id)
                .execute(pool)
                .await;
        }
        existing_id
    } else {
        let clip_uuid = Uuid::new_v4().to_string();

        let _ = sqlx::query(
            r#"
            INSERT INTO clips (uuid, clip_type, content, text_preview, content_hash, folder_id, is_deleted, is_thumbnail, source_app, source_icon, metadata, created_at, last_accessed)
            VALUES (?, ?, ?, ?, ?, NULL, 0, ?, ?, ?, ?, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
            "#,
        )
        .bind(&clip_uuid)
        .bind(clip_type)
        .bind(&clip_content)
        .bind(&clip_preview)
        .bind(&clip_hash)
        .bind(clip_type == "image")
        .bind(&source_app)
        .bind(&source_icon)
        .bind(if clip_type == "image" {
            Some(metadata)
        } else {
            None
        })
        .execute(pool)
        .await;

        if clip_type == "image" {
            if let Some(full_bytes) = &full_image_content {
                match persist_full_image_file(&clip_uuid, full_bytes) {
                    Ok(file_path) => {
                        let _ = sqlx::query(
                            r#"
                            INSERT OR REPLACE INTO clip_images (clip_uuid, full_content, file_path, file_size, storage_kind, mime_type, created_at)
                            VALUES (?, x'', ?, ?, 'file', 'image/png', CURRENT_TIMESTAMP)
                            "#,
                        )
                        .bind(&clip_uuid)
                        .bind(&file_path)
                        .bind(full_bytes.len() as i64)
                        .execute(pool)
                        .await;
                    }
                    Err(e) => {
                        log::warn!("Failed to persist full image file, fallback to DB blob: {}", e);
                        let _ = sqlx::query(
                            r#"
                            INSERT OR REPLACE INTO clip_images (clip_uuid, full_content, file_path, file_size, storage_kind, mime_type, created_at)
                            VALUES (?, ?, NULL, ?, 'db', 'image/png', CURRENT_TIMESTAMP)
                            "#,
                        )
                        .bind(&clip_uuid)
                        .bind(full_bytes)
                        .bind(full_bytes.len() as i64)
                        .execute(pool)
                        .await;
                    }
                }
            }
        }
        clip_uuid
    };
    let db_write_ms = db_write_started.elapsed().as_millis();

    let emit_started = std::time::Instant::now();
    let _ = app.emit(
        "clipboard-change",
        &serde_json::json!({
            "id": emitted_id,
            "content": clip_preview,
            "clip_type": clip_type,
            "source_app": source_app,
            "source_icon": source_icon,
            "created_at": chrono::Utc::now().to_rfc3339()
        }),
    );
    let emit_ms = emit_started.elapsed().as_millis();

    log::info!(
        "[perf][clipboard_ingest] type={} existing={} full_bytes={} thumb_bytes={} image_read_ms={} decode_ms={} thumb_ms={} text_read_ms={} db_lookup_ms={} db_write_ms={} emit_ms={} total_ms={}",
        clip_type,
        was_existing,
        full_image_content.as_ref().map(|v| v.len()).unwrap_or(0),
        if clip_type == "image" { clip_content.len() } else { 0 },
        image_read_ms,
        image_decode_ms,
        image_thumbnail_ms,
        text_read_ms,
        db_lookup_ms,
        db_write_ms,
        emit_ms,
        started.elapsed().as_millis()
    );
}
fn calculate_hash(content: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content);
    let result = hasher.finalize();
    format!("{:x}", result)
}

fn get_image_store_dir() -> std::path::PathBuf {
    let current_dir = std::env::current_dir().unwrap_or(std::path::PathBuf::from("."));
    let app_data_dir = match dirs::data_dir() {
        Some(path) => path.join("PastePaw"),
        None => current_dir.join("PastePaw"),
    };
    app_data_dir.join("images")
}

pub fn persist_full_image_file(clip_uuid: &str, png_bytes: &[u8]) -> Result<String, String> {
    let dir = get_image_store_dir();
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let file_path = dir.join(format!("{}.png", clip_uuid));
    std::fs::write(&file_path, png_bytes).map_err(|e| e.to_string())?;
    Ok(file_path.to_string_lossy().to_string())
}

pub fn read_full_image_file(file_path: &str) -> Result<Vec<u8>, String> {
    std::fs::read(file_path).map_err(|e| e.to_string())
}

pub fn remove_full_image_file(file_path: &str) {
    if let Err(e) = std::fs::remove_file(file_path) {
        if e.kind() != std::io::ErrorKind::NotFound {
            log::warn!("Failed to delete image file {}: {}", file_path, e);
        }
    }
}

pub fn create_image_thumbnail(content: &[u8], max_edge: u32) -> Result<Vec<u8>, String> {
    use image::codecs::png::PngEncoder;
    use image::imageops::FilterType;
    use image::{ColorType, ImageEncoder};

    let img = image::load_from_memory(content).map_err(|e| e.to_string())?;
    let (width, height) = (img.width(), img.height());
    if width == 0 || height == 0 {
        return Err("Invalid image dimensions".to_string());
    }

    let scale = if width > height {
        max_edge as f32 / width as f32
    } else {
        max_edge as f32 / height as f32
    };

    let (target_w, target_h) = if scale >= 1.0 {
        (width, height)
    } else {
        (
            ((width as f32 * scale).round() as u32).max(1),
            ((height as f32 * scale).round() as u32).max(1),
        )
    };

    let resized = if target_w == width && target_h == height {
        img.to_rgba8()
    } else {
        img.resize_exact(target_w, target_h, FilterType::Triangle)
            .to_rgba8()
    };

    let mut out = Vec::new();
    PngEncoder::new(&mut out)
        .write_image(
            resized.as_raw(),
            resized.width(),
            resized.height(),
            ColorType::Rgba8,
        )
        .map_err(|e| e.to_string())?;

    Ok(out)
}

#[cfg(target_os = "windows")]
fn get_clipboard_owner_app_info() -> (
    Option<String>,
    Option<String>,
    Option<String>,
    Option<String>,
    bool,
) {
    unsafe {
        let (hwnd, is_explicit) = match GetClipboardOwner() {
            Ok(h) if !h.0.is_null() => (h, true),
            Err(e) => {
                log::info!(
                    "CLIPBOARD: GetClipboardOwner failed: {:?}, falling back to foreground window",
                    e
                );
                (GetForegroundWindow(), false)
            }
            Ok(_) => {
                log::info!(
                    "CLIPBOARD: GetClipboardOwner returned null, falling back to foreground window"
                );
                (GetForegroundWindow(), false)
            }
        };

        if hwnd.0.is_null() {
            return (None, None, None, None, false);
        }

        let mut process_id = 0;
        GetWindowThreadProcessId(hwnd, Some(&mut process_id));

        if process_id == 0 {
            return (None, None, None, None, false);
        }

        let process_handle = match OpenProcess(
            PROCESS_QUERY_INFORMATION | PROCESS_VM_READ,
            false,
            process_id,
        ) {
            Ok(h) => h,
            Err(_) => return (None, None, None, None, false),
        };

        let mut name_buffer = [0u16; MAX_PATH as usize];
        let name_size = GetModuleBaseNameW(process_handle, None, &mut name_buffer);
        let exe_name = if name_size > 0 {
            String::from_utf16_lossy(&name_buffer[..name_size as usize])
        } else {
            String::new()
        };

        let mut path_buffer = [0u16; MAX_PATH as usize];
        let path_size = GetModuleFileNameExW(Some(process_handle), None, &mut path_buffer);
        let (app_name, app_icon, full_path) = if path_size > 0 {
            let full_path_str = String::from_utf16_lossy(&path_buffer[..path_size as usize]);

            let desc = get_app_description(&full_path_str);
            let final_name = if let Some(d) = desc {
                Some(d)
            } else {
                if !exe_name.is_empty() {
                    Some(exe_name.clone())
                } else {
                    None
                }
            };

            let icon = extract_icon(&full_path_str);
            (final_name, icon, Some(full_path_str))
        } else {
            (
                if !exe_name.is_empty() {
                    Some(exe_name.clone())
                } else {
                    None
                },
                None,
                None,
            )
        };

        let exe_val = if !exe_name.is_empty() {
            Some(exe_name)
        } else {
            None
        };
        (app_name, app_icon, exe_val, full_path, is_explicit)
    }
}

#[cfg(target_os = "windows")]
unsafe fn get_app_description(path: &str) -> Option<String> {
    use std::ffi::c_void;

    let wide_path: Vec<u16> = OsStr::new(path)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();

    let size = GetFileVersionInfoSizeW(windows::core::PCWSTR(wide_path.as_ptr()), None);
    if size == 0 {
        return None;
    }

    let mut data = vec![0u8; size as usize];
    if GetFileVersionInfoW(
        windows::core::PCWSTR(wide_path.as_ptr()),
        Some(0),
        size,
        data.as_mut_ptr() as *mut _,
    )
    .is_err()
    {
        return None;
    }

    let mut lang_ptr: *mut c_void = std::ptr::null_mut();
    let mut lang_len: u32 = 0;

    let translation_query = OsStr::new("\\VarFileInfo\\Translation")
        .encode_wide()
        .chain(std::iter::once(0))
        .collect::<Vec<u16>>();

    if !VerQueryValueW(
        data.as_ptr() as *const _,
        windows::core::PCWSTR(translation_query.as_ptr()),
        &mut lang_ptr,
        &mut lang_len,
    )
    .as_bool()
    {
        return None;
    }

    if lang_len < 4 {
        return None;
    }

    let pairs = std::slice::from_raw_parts(lang_ptr as *const u16, (lang_len / 2) as usize);
    let num_pairs = (lang_len / 4) as usize;

    let mut lang_code = pairs[0];
    let mut charset_code = pairs[1];

    for i in 0..num_pairs {
        let code = pairs[i * 2];
        let charset = pairs[i * 2 + 1];

        if code == 0x0804 {
            lang_code = code;
            charset_code = charset;
        }
    }

    let keys = ["FileDescription", "ProductName"];

    for key in keys {
        let query_str = format!(
            "\\StringFileInfo\\{:04x}{:04x}\\{}",
            lang_code, charset_code, key
        );
        let query = OsStr::new(&query_str)
            .encode_wide()
            .chain(std::iter::once(0))
            .collect::<Vec<u16>>();

        let mut desc_ptr: *mut c_void = std::ptr::null_mut();
        let mut desc_len: u32 = 0;

        if VerQueryValueW(
            data.as_ptr() as *const _,
            windows::core::PCWSTR(query.as_ptr()),
            &mut desc_ptr,
            &mut desc_len,
        )
        .as_bool()
        {
            let desc = std::slice::from_raw_parts(desc_ptr as *const u16, desc_len as usize);
            let len = if desc.last() == Some(&0) {
                desc.len() - 1
            } else {
                desc.len()
            };
            if len > 0 {
                return Some(String::from_utf16_lossy(&desc[..len]));
            }
        }
    }

    None
}

#[cfg(target_os = "windows")]
unsafe fn extract_icon(path: &str) -> Option<String> {
    use image::ImageEncoder;

    let wide_path: Vec<u16> = OsStr::new(path)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();
    let mut shfi = SHFILEINFOW::default();

    SHGetFileInfoW(
        windows::core::PCWSTR(wide_path.as_ptr()),
        windows::Win32::Storage::FileSystem::FILE_ATTRIBUTE_NORMAL,
        Some(&mut shfi as *mut _),
        std::mem::size_of::<SHFILEINFOW>() as u32,
        SHGFI_ICON | SHGFI_LARGEICON | SHGFI_USEFILEATTRIBUTES,
    );

    if shfi.hIcon.is_invalid() {
        return None;
    }

    let icon = shfi.hIcon;
    struct IconGuard(windows::Win32::UI::WindowsAndMessaging::HICON);
    impl Drop for IconGuard {
        fn drop(&mut self) {
            unsafe {
                let _ = DestroyIcon(self.0);
            }
        }
    }
    let _guard = IconGuard(icon);

    let mut icon_info = ICONINFO::default();
    if GetIconInfo(icon, &mut icon_info).is_err() {
        return None;
    }

    struct BitmapGuard(HBITMAP);
    impl Drop for BitmapGuard {
        fn drop(&mut self) {
            unsafe {
                if !self.0.is_invalid() {
                    let _ = DeleteObject(self.0.into());
                }
            }
        }
    }
    let _bm_mask = BitmapGuard(icon_info.hbmMask);
    let _bm_color = BitmapGuard(icon_info.hbmColor);

    let mut bm = BITMAP::default();
    if GetObjectW(
        icon_info.hbmMask.into(),
        std::mem::size_of::<BITMAP>() as i32,
        Some(&mut bm as *mut _ as *mut _),
    ) == 0
    {
        return None;
    }

    let width = bm.bmWidth;
    let height = if !icon_info.hbmColor.is_invalid() {
        bm.bmHeight
    } else {
        bm.bmHeight / 2
    };

    let screen_dc = GetDC(None);
    let mem_dc = CreateCompatibleDC(Some(screen_dc));
    let mem_bm = CreateCompatibleBitmap(screen_dc, width, height);

    let old_obj = SelectObject(mem_dc, mem_bm.into());

    let _ = DrawIconEx(mem_dc, 0, 0, icon, width, height, 0, None, DI_NORMAL);

    let bi = BITMAPINFOHEADER {
        biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
        biWidth: width,
        biHeight: -height,
        biPlanes: 1,
        biBitCount: 32,
        biCompression: BI_RGB.0,
        ..Default::default()
    };

    let mut pixels = vec![0u8; (width * height * 4) as usize];

    GetDIBits(
        mem_dc,
        mem_bm,
        0,
        height as u32,
        Some(pixels.as_mut_ptr() as *mut _),
        &mut BITMAPINFO {
            bmiHeader: bi,
            ..Default::default()
        },
        DIB_RGB_COLORS,
    );

    SelectObject(mem_dc, old_obj);
    let _ = DeleteDC(mem_dc);
    let _ = DeleteObject(mem_bm.into());
    let _ = ReleaseDC(None, screen_dc);

    for chunk in pixels.chunks_exact_mut(4) {
        let b = chunk[0];
        let r = chunk[2];
        chunk[0] = r;
        chunk[2] = b;
    }

    let mut png_data = Vec::new();
    let encoder = image::codecs::png::PngEncoder::new(&mut png_data);
    encoder
        .write_image(
            &pixels,
            width as u32,
            height as u32,
            image::ColorType::Rgba8,
        )
        .ok()?;

    Some(BASE64.encode(&png_data))
}

#[cfg(target_os = "windows")]
pub fn send_paste_input() {
    unsafe {
        let inputs = vec![
            INPUT {
                r#type: INPUT_KEYBOARD,
                Anonymous: windows::Win32::UI::Input::KeyboardAndMouse::INPUT_0 {
                    ki: KEYBDINPUT {
                        wVk: VK_SHIFT,
                        ..Default::default()
                    },
                },
            },
            INPUT {
                r#type: INPUT_KEYBOARD,
                Anonymous: windows::Win32::UI::Input::KeyboardAndMouse::INPUT_0 {
                    ki: KEYBDINPUT {
                        wVk: VK_INSERT,
                        ..Default::default()
                    },
                },
            },
            INPUT {
                r#type: INPUT_KEYBOARD,
                Anonymous: windows::Win32::UI::Input::KeyboardAndMouse::INPUT_0 {
                    ki: KEYBDINPUT {
                        wVk: VK_INSERT,
                        dwFlags: KEYEVENTF_KEYUP,
                        ..Default::default()
                    },
                },
            },
            INPUT {
                r#type: INPUT_KEYBOARD,
                Anonymous: windows::Win32::UI::Input::KeyboardAndMouse::INPUT_0 {
                    ki: KEYBDINPUT {
                        wVk: VK_SHIFT,
                        dwFlags: KEYEVENTF_KEYUP,
                        ..Default::default()
                    },
                },
            },
        ];

        SendInput(&inputs, std::mem::size_of::<INPUT>() as i32);
    }
}

#[cfg(target_os = "macos")]
fn get_clipboard_owner_app_info() -> (
    Option<String>,
    Option<String>,
    Option<String>,
    Option<String>,
    bool,
) {
    crate::source_app_macos::get_frontmost_app_info()
}

#[cfg(target_os = "macos")]
pub fn write_png_to_pasteboard(png_bytes: &[u8]) -> Result<(), String> {
    use cocoa::base::{id, nil, BOOL};
    use cocoa::foundation::NSString;
    use objc::{class, msg_send, sel, sel_impl};

    // Write PNG to a temp file so target apps can read via file URL (fast SSD path)
    let tmp_dir = std::env::temp_dir().join("pastepaw_images");
    let _ = std::fs::create_dir_all(&tmp_dir);
    let tmp_path = tmp_dir.join(format!("paste_{}.png", std::process::id()));
    std::fs::write(&tmp_path, png_bytes).map_err(|e| format!("Failed to write temp PNG: {}", e))?;
    let file_url_str = format!("file://{}", tmp_path.to_string_lossy());

    unsafe {
        let pasteboard: id = msg_send![class!(NSPasteboard), generalPasteboard];
        let _: i64 = msg_send![pasteboard, clearContents];

        let png_data: id =
            msg_send![class!(NSData), dataWithBytes:png_bytes.as_ptr() length:png_bytes.len()];
        let url_nsstring = NSString::alloc(nil).init_str(&file_url_str);

        let png_type = NSString::alloc(nil).init_str("public.png");
        let file_url_type = NSString::alloc(nil).init_str("public.file-url");

        let _: BOOL = msg_send![pasteboard, setData:png_data forType:png_type];

        // Set file URL as UTF-8 data — target apps read the image from disk directly
        let url_data: id = msg_send![url_nsstring, dataUsingEncoding:4u64]; // NSUTF8StringEncoding = 4
        let _: BOOL = msg_send![pasteboard, setData:url_data forType:file_url_type];
    }
    Ok(())
}

#[cfg(target_os = "macos")]
pub fn send_paste_input() {
    use core_graphics::event::{CGEvent, CGEventFlags, CGEventTapLocation, CGKeyCode};
    use core_graphics::event_source::{CGEventSource, CGEventSourceStateID};

    #[cfg(feature = "app-store")]
    {
        if !crate::source_app_macos::is_accessibility_enabled() {
            log::warn!(
                "CLIPBOARD: Auto-paste failed because Accessibility permissions are not granted."
            );
            return;
        }
    }

    // Give a brief delay for focus to switch
    std::thread::sleep(std::time::Duration::from_millis(20));

    // kVK_ANSI_V = 0x09
    let v_key: CGKeyCode = 0x09;
    // kVK_Command = 0x37 (55)
    let cmd_key: CGKeyCode = 0x37;

    let source = match CGEventSource::new(CGEventSourceStateID::HIDSystemState) {
        Ok(src) => src,
        Err(e) => {
            log::error!("CLIPBOARD: Failed to create CGEventSource: {:?}", e);
            return;
        }
    };

    // 1. Cmd Down
    if let Ok(event) = CGEvent::new_keyboard_event(source.clone(), cmd_key, true) {
        event.set_flags(CGEventFlags::CGEventFlagCommand);
        event.post(CGEventTapLocation::HID);
    } else {
        log::error!("CLIPBOARD: Failed to create Cmd Down event");
    }

    // 2. V Down
    if let Ok(event) = CGEvent::new_keyboard_event(source.clone(), v_key, true) {
        event.set_flags(CGEventFlags::CGEventFlagCommand);
        event.post(CGEventTapLocation::HID);
    } else {
        log::error!("CLIPBOARD: Failed to create V Down event");
    }

    // 3. V Up
    if let Ok(event) = CGEvent::new_keyboard_event(source.clone(), v_key, false) {
        event.set_flags(CGEventFlags::CGEventFlagCommand);
        event.post(CGEventTapLocation::HID);
    }

    // 4. Cmd Up
    if let Ok(event) = CGEvent::new_keyboard_event(source, cmd_key, false) {
        event.post(CGEventTapLocation::HID);
    }

    log::info!("CLIPBOARD: Sent Cmd+V via CoreGraphics");
}
