
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
use windows::Win32::System::ProcessStatus::{GetModuleBaseNameW, GetModuleFileNameExW};
use windows::Win32::System::DataExchange::GetClipboardOwner;
use windows::Win32::UI::WindowsAndMessaging::{GetForegroundWindow, GetWindowThreadProcessId, DestroyIcon, DrawIconEx, DI_NORMAL, GetIconInfo, ICONINFO};
use windows::Win32::UI::Shell::{SHGetFileInfoW, SHGFI_ICON, SHGFI_LARGEICON, SHFILEINFOW, SHGFI_USEFILEATTRIBUTES};
use windows::Win32::Graphics::Gdi::{
    GetObjectW, DeleteObject, GetDC, ReleaseDC, CreateCompatibleDC, SelectObject, DeleteDC,
    GetDIBits, BITMAPINFO, BITMAPINFOHEADER, BI_RGB, DIB_RGB_COLORS,
    BITMAP, HBITMAP, HDC, CreateCompatibleBitmap
};
use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;

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
    // Ignore updates from self
    unsafe {
        if let Ok(hwnd) = GetClipboardOwner() {
            if !hwnd.0.is_null() {
                let mut process_id = 0;
                GetWindowThreadProcessId(hwnd, Some(&mut process_id));
                if process_id == std::process::id() {
                    return;
                }
            }
        }
    }

    // Initialize Clipboard struct
    let clipboard = app.state::<ClipboardPlugin>();

    // capture source app info immediately
    let (source_app, source_icon) = get_clipboard_owner_app_info();

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
            "source_app": source_app,
            "source_icon": source_icon,
            "created_at": chrono::Utc::now().to_rfc3339()
        }));
    } else {
        let clip_uuid = Uuid::new_v4().to_string();

        let _ = sqlx::query(r#"
            INSERT INTO clips (uuid, clip_type, content, text_preview, content_hash, folder_id, is_pinned, is_deleted, source_app, source_icon, metadata, created_at, last_accessed)
            VALUES (?, ?, ?, ?, ?, NULL, 0, 0, ?, ?, ?, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
        "#)
        .bind(&clip_uuid)
        .bind(clip_type)
        .bind(&clip_content)
        .bind(&clip_preview)
        .bind(&clip_hash)
        .bind(&source_app)
        .bind(&source_icon)
        .bind(if clip_type == "image" { Some(metadata) } else { None })
        .execute(pool)
        .await;

        let _ = app.emit("clipboard-change", &serde_json::json!({
            "id": clip_uuid,
            "content": clip_preview,
            "clip_type": clip_type,
            "is_pinned": false,
            "source_app": source_app,
            "source_icon": source_icon,
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

fn get_clipboard_owner_app_info() -> (Option<String>, Option<String>) {
    unsafe {
        let mut hwnd = match GetClipboardOwner() {
            Ok(h) if !h.0.is_null() => h,
            Err(e) => {
                eprintln!("CLIPBOARD: GetClipboardOwner failed: {:?}, falling back to foreground window", e);
                GetForegroundWindow()
            },
            Ok(_) => {
                eprintln!("CLIPBOARD: GetClipboardOwner returned null, falling back to foreground window");
                GetForegroundWindow()
            }
        };

        if hwnd.0.is_null() {
            return (None, None);
        }

        let mut process_id = 0;
        GetWindowThreadProcessId(hwnd, Some(&mut process_id));

        if process_id == 0 {
            return (None, None);
        }

        let process_handle = match OpenProcess(PROCESS_QUERY_INFORMATION | PROCESS_VM_READ, false, process_id) {
            Ok(h) => h,
            Err(_) => return (None, None),
        };

        let mut name_buffer = [0u16; MAX_PATH as usize];
        let name_size = GetModuleBaseNameW(process_handle, None, &mut name_buffer);
        let app_name = if name_size > 0 {
            Some(String::from_utf16_lossy(&name_buffer[..name_size as usize]))
        } else {
            None
        };

        let mut path_buffer = [0u16; MAX_PATH as usize];
        let path_size = GetModuleFileNameExW(Some(process_handle), None, &mut path_buffer);
        let app_icon = if path_size > 0 {
            let path = String::from_utf16_lossy(&path_buffer[..path_size as usize]);
            extract_icon(&path)
        } else {
            None
        };

        (app_name, app_icon)
    }
}

unsafe fn extract_icon(path: &str) -> Option<String> {
    use image::ImageEncoder;

    let wide_path: Vec<u16> = OsStr::new(path).encode_wide().chain(std::iter::once(0)).collect();
    let mut shfi = SHFILEINFOW::default();

    SHGetFileInfoW(
        windows::core::PCWSTR(wide_path.as_ptr()),
        windows::Win32::Storage::FileSystem::FILE_ATTRIBUTE_NORMAL,
        Some(&mut shfi as *mut _),
        std::mem::size_of::<SHFILEINFOW>() as u32,
        SHGFI_ICON | SHGFI_LARGEICON | SHGFI_USEFILEATTRIBUTES
    );

    if shfi.hIcon.is_invalid() {
        return None;
    }

    let icon = shfi.hIcon;
    struct IconGuard(windows::Win32::UI::WindowsAndMessaging::HICON);
    impl Drop for IconGuard { fn drop(&mut self) { unsafe { DestroyIcon(self.0); } } }
    let _guard = IconGuard(icon);

    let mut icon_info = ICONINFO::default();
    if GetIconInfo(icon, &mut icon_info).is_err() { return None; }

    struct BitmapGuard(HBITMAP);
    impl Drop for BitmapGuard { fn drop(&mut self) { unsafe { if !self.0.is_invalid() { DeleteObject(self.0.into()); } } } }
    let _bm_mask = BitmapGuard(icon_info.hbmMask);
    let _bm_color = BitmapGuard(icon_info.hbmColor);

    let mut bm = BITMAP::default();
    if GetObjectW(icon_info.hbmMask.into(), std::mem::size_of::<BITMAP>() as i32, Some(&mut bm as *mut _ as *mut _)) == 0 { return None; }

    let width = bm.bmWidth;
    let height = if !icon_info.hbmColor.is_invalid() { bm.bmHeight } else { bm.bmHeight / 2 };

    let screen_dc = GetDC(None);
    let mem_dc = CreateCompatibleDC(Some(screen_dc));
    let mem_bm = CreateCompatibleBitmap(screen_dc, width, height);

    let old_obj = SelectObject(mem_dc, mem_bm.into());

    DrawIconEx(mem_dc, 0, 0, icon, width, height, 0, None, DI_NORMAL);

    let mut bi = BITMAPINFOHEADER {
        biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
        biWidth: width,
        biHeight: -height,
        biPlanes: 1,
        biBitCount: 32,
        biCompression: BI_RGB.0,
        ..Default::default()
    };

    let mut pixels = vec![0u8; (width * height * 4) as usize];

    GetDIBits(mem_dc, mem_bm, 0, height as u32, Some(pixels.as_mut_ptr() as *mut _), &mut BITMAPINFO { bmiHeader: bi, ..Default::default() }, DIB_RGB_COLORS);

    SelectObject(mem_dc, old_obj);
    DeleteDC(mem_dc);
    DeleteObject(mem_bm.into());
    ReleaseDC(None, screen_dc);

    // Convert BGRA to RGBA
    for chunk in pixels.chunks_exact_mut(4) {
        let b = chunk[0];
        let r = chunk[2];
        chunk[0] = r;
        chunk[2] = b;
    }

    let mut png_data = Vec::new();
    let encoder = image::codecs::png::PngEncoder::new(&mut png_data);
    encoder.write_image(&pixels, width as u32, height as u32, image::ColorType::Rgba8).ok()?;

    Some(BASE64.encode(&png_data))
}
