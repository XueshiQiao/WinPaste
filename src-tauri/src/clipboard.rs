
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
use windows::Win32::Storage::FileSystem::{GetFileVersionInfoSizeW, GetFileVersionInfoW, VerQueryValueW};
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
use std::sync::Mutex;
use once_cell::sync::Lazy;

// GLOBAL STATE: Store the hash of the clip we just pasted ourselves.
// If the next clipboard change matches this hash, we ignore it (don't update timestamp).
static IGNORE_HASH: Lazy<Mutex<Option<String>>> = Lazy::new(|| Mutex::new(None));

pub fn set_ignore_hash(hash: String) {
    if let Ok(mut lock) = IGNORE_HASH.lock() {
        *lock = Some(hash);
    }
}

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

    // Check if we should ignore this update (it's our own paste via double clickiing the card)
    if let Ok(mut lock) = IGNORE_HASH.lock() {
        if let Some(ignore_hash) = lock.take() {
            if ignore_hash == clip_hash {
                eprintln!("CLIPBOARD: Ignoring update for hash {} (detect self-paste)", ignore_hash);
                return;
            }
        }
    }

    // capture source app info only if we are proceeding
    let (source_app, source_icon) = get_clipboard_owner_app_info();

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
        let hwnd = match GetClipboardOwner() {
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
        let exe_name = if name_size > 0 {
            String::from_utf16_lossy(&name_buffer[..name_size as usize])
        } else {
            String::new()
        };

        let mut path_buffer = [0u16; MAX_PATH as usize];
        let path_size = GetModuleFileNameExW(Some(process_handle), None, &mut path_buffer);
        let (app_name, app_icon) = if path_size > 0 {
            let full_path = String::from_utf16_lossy(&path_buffer[..path_size as usize]);

            let desc = get_app_description(&full_path);
            let final_name = if let Some(d) = desc {
                eprintln!("CLIPBOARD: Found description '{}' for {}", d, full_path);
                Some(d)
            } else {
                eprintln!("CLIPBOARD: No description for {}, using exe name '{}'", full_path, exe_name);
                if !exe_name.is_empty() { Some(exe_name.clone()) } else { None }
            };

            let icon = extract_icon(&full_path);
            (final_name, icon)
        } else {
            (if !exe_name.is_empty() { Some(exe_name) } else { None }, None)
        };

        (app_name, app_icon)
    }
}

unsafe fn get_app_description(path: &str) -> Option<String> {
    use std::ffi::c_void;

    let wide_path: Vec<u16> = OsStr::new(path).encode_wide().chain(std::iter::once(0)).collect();

    let size = GetFileVersionInfoSizeW(windows::core::PCWSTR(wide_path.as_ptr()), None);
    if size == 0 { return None; }

    let mut data = vec![0u8; size as usize];
    if GetFileVersionInfoW(windows::core::PCWSTR(wide_path.as_ptr()), Some(0), size, data.as_mut_ptr() as *mut _).is_err() {
        return None;
    }

    let mut lang_ptr: *mut c_void = std::ptr::null_mut();
    let mut lang_len: u32 = 0;

    let translation_query = OsStr::new("\\VarFileInfo\\Translation").encode_wide().chain(std::iter::once(0)).collect::<Vec<u16>>();

    if !VerQueryValueW(data.as_ptr() as *const _, windows::core::PCWSTR(translation_query.as_ptr()), &mut lang_ptr, &mut lang_len).as_bool() {
        return None;
    }

    if lang_len < 4 { return None; }

    let pairs = std::slice::from_raw_parts(lang_ptr as *const u16, (lang_len / 2) as usize);
    let num_pairs = (lang_len / 4) as usize;

    let mut lang_code = pairs[0];
    let mut charset_code = pairs[1];

    // Log available translations
    for i in 0..num_pairs {
        let code = pairs[i * 2];
        let charset = pairs[i * 2 + 1];
        eprintln!("CLIPBOARD: Found translation: lang={:04x}, charset={:04x}", code, charset);

        // Prioritize Chinese Simplified (0x0804)
        if code == 0x0804 {
            lang_code = code;
            charset_code = charset;
        }
    }

    eprintln!("CLIPBOARD: Using translation: lang={:04x}, charset={:04x}", lang_code, charset_code);

    let keys = ["FileDescription", "ProductName"];

    for key in keys {
        let query_str = format!("\\StringFileInfo\\{:04x}{:04x}\\{}", lang_code, charset_code, key);
        let query = OsStr::new(&query_str).encode_wide().chain(std::iter::once(0)).collect::<Vec<u16>>();

        let mut desc_ptr: *mut c_void = std::ptr::null_mut();
        let mut desc_len: u32 = 0;

        if VerQueryValueW(data.as_ptr() as *const _, windows::core::PCWSTR(query.as_ptr()), &mut desc_ptr, &mut desc_len).as_bool() {
             let desc = std::slice::from_raw_parts(desc_ptr as *const u16, desc_len as usize);
             let len = if desc.last() == Some(&0) { desc.len() - 1 } else { desc.len() };
             if len > 0 {
                 let val = String::from_utf16_lossy(&desc[..len]);
                 eprintln!("CLIPBOARD: Found {} = {}", key, val);
                 return Some(val);
             }
        } else {
             eprintln!("CLIPBOARD: Key {} not found", key);
        }
    }

    None
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
