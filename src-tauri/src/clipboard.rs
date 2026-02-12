
use tauri::{AppHandle, Listener, Emitter};
// Import functions directly from the crate root
use tauri_plugin_clipboard_x::{read_image, read_text, start_listening};
use std::sync::Arc;
use crate::database::Database;
use uuid::Uuid;
use sha2::{Digest, Sha256};
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
#[cfg(target_os = "windows")]
use windows::Win32::Foundation::MAX_PATH;
#[cfg(target_os = "windows")]
use windows::Win32::System::Threading::{OpenProcess, PROCESS_QUERY_INFORMATION, PROCESS_VM_READ};
#[cfg(target_os = "windows")]
use windows::Win32::System::ProcessStatus::{GetModuleBaseNameW, GetModuleFileNameExW};
#[cfg(target_os = "windows")]
use windows::Win32::Storage::FileSystem::{GetFileVersionInfoSizeW, GetFileVersionInfoW, VerQueryValueW};
#[cfg(target_os = "windows")]
use windows::Win32::System::DataExchange::GetClipboardOwner;
#[cfg(target_os = "windows")]
use windows::Win32::UI::WindowsAndMessaging::{GetForegroundWindow, GetWindowThreadProcessId, DestroyIcon, DrawIconEx, DI_NORMAL, GetIconInfo, ICONINFO};
#[cfg(target_os = "windows")]
use windows::Win32::UI::Input::KeyboardAndMouse::{SendInput, INPUT, INPUT_KEYBOARD, KEYBDINPUT, KEYEVENTF_KEYUP, VK_SHIFT, VK_INSERT};
#[cfg(target_os = "windows")]
use windows::Win32::UI::Shell::{SHGetFileInfoW, SHGFI_ICON, SHGFI_LARGEICON, SHFILEINFOW, SHGFI_USEFILEATTRIBUTES};
#[cfg(target_os = "windows")]
use windows::Win32::Graphics::Gdi::{
    GetObjectW, GetDC, ReleaseDC, CreateCompatibleDC, SelectObject, DeleteDC,
    GetDIBits, BITMAPINFO, BITMAPINFOHEADER, BI_RGB, DIB_RGB_COLORS,
    BITMAP, HBITMAP, CreateCompatibleBitmap, DeleteObject
};
#[cfg(target_os = "windows")]
use std::ffi::OsStr;
#[cfg(target_os = "windows")]
use std::os::windows::ffi::OsStrExt;
use once_cell::sync::Lazy;

// GLOBAL STATE: Store the hash of the clip we just pasted ourselves.
// If the next clipboard change matches this hash, we ignore it (don't update timestamp).
static IGNORE_HASH: Lazy<parking_lot::Mutex<Option<String>>> = Lazy::new(|| parking_lot::Mutex::new(None));
static LAST_STABLE_HASH: Lazy<parking_lot::Mutex<Option<String>>> = Lazy::new(|| parking_lot::Mutex::new(None));
pub static CLIPBOARD_SYNC: Lazy<Arc<tokio::sync::Mutex<()>>> = Lazy::new(|| Arc::new(tokio::sync::Mutex::new(())));

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

        // DEBOUNCE LOGIC:
        let current_count = DEBOUNCE_COUNTER.fetch_add(1, Ordering::SeqCst) + 1;

        tauri::async_runtime::spawn(async move {
            tokio::time::sleep(std::time::Duration::from_millis(150)).await;

            if DEBOUNCE_COUNTER.load(Ordering::SeqCst) != current_count {
                log::debug!("CLIPBOARD: Debounce: Aborting older event, current_count:{}", current_count);
                return;
            }

            process_clipboard_change(app, db).await;
        });
    });
}

async fn process_clipboard_change(app: AppHandle, db: Arc<Database>) {
    let _guard = CLIPBOARD_SYNC.lock().await;

    let mut clip_type = "text";
    let mut clip_content = Vec::new();
    let mut clip_preview = String::new();
    let mut clip_hash = String::new();
    let mut metadata = String::new();
    let mut found_content = false;

    // Try Image
    // read_image saves to disk and returns info
    // We pass None for save_path to use default
    if let Ok(read_image_result) = read_image(app.clone(), None).await {
         // read_image_result is ReadImage { path: PathBuf, ... }
         // We need to read the file content
         if let Ok(bytes) = std::fs::read(&read_image_result.path) {
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
                     "format": "png", // Usually it saves as png? Code said format string "{hash}.png"
                     "size_bytes": size_bytes
                 }).to_string();
                 found_content = true;

                 // Clean up the temp file
                 let _ = std::fs::remove_file(read_image_result.path);
             }
         }
    }

    if !found_content {
        // Try Text
        if let Ok(text) = read_text().await {
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
                log::info!("CLIPBOARD: Detected self-paste for hash {}, proceeding to update timestamp", ignore_hash);
            }
        }
    }

    // capture source app info
    let (source_app, source_icon, exe_name, full_path, is_explicit_owner) = get_clipboard_owner_app_info();
    log::info!("CLIPBOARD: Source app: {:?}, exe_name: {:?}, full_path: {:?}, explicit: {}", source_app, exe_name, full_path, is_explicit_owner);

    // Check ignore_ghost_clips setting
    let pool = &db.pool;
    let ignore_ghost_clips = sqlx::query_scalar::<_, String>(r#"SELECT value FROM settings WHERE key = 'ignore_ghost_clips'"#)
        .fetch_optional(pool)
        .await
        .unwrap_or(None)
        .and_then(|v| v.parse::<bool>().ok())
        .unwrap_or(false);

    if ignore_ghost_clips && !is_explicit_owner {
        log::info!("CLIPBOARD: Ignoring ghost clip (unknown owner)");
        return;
    }

    // Check if the app is in the ignore list
    if let Some(ref path) = full_path {
        if let Ok(true) = db.is_app_ignored(path).await {
             log::info!("CLIPBOARD: Ignoring content from ignored app (path match): {}", path);
             return;
        }
    }

    if let Some(ref exe) = exe_name {
        if let Ok(true) = db.is_app_ignored(exe).await {
             log::info!("CLIPBOARD: Ignoring content from ignored app (exe match): {}", exe);
             return;
        }
    }

    // DB Logic
    let pool = &db.pool;

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
            "source_app": source_app,
            "source_icon": source_icon,
            "created_at": chrono::Utc::now().to_rfc3339()
        }));
    } else {
        let clip_uuid = Uuid::new_v4().to_string();

        let _ = sqlx::query(r#"
            INSERT INTO clips (uuid, clip_type, content, text_preview, content_hash, folder_id, is_deleted, source_app, source_icon, metadata, created_at, last_accessed)
            VALUES (?, ?, ?, ?, ?, NULL, 0, ?, ?, ?, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
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

#[cfg(target_os = "windows")]
fn get_clipboard_owner_app_info() -> (Option<String>, Option<String>, Option<String>, Option<String>, bool) {
    unsafe {
        let (hwnd, is_explicit) = match GetClipboardOwner() {
            Ok(h) if !h.0.is_null() => (h, true),
            Err(e) => {
                log::info!("CLIPBOARD: GetClipboardOwner failed: {:?}, falling back to foreground window", e);
                (GetForegroundWindow(), false)
            },
            Ok(_) => {
                log::info!("CLIPBOARD: GetClipboardOwner returned null, falling back to foreground window");
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

        let process_handle = match OpenProcess(PROCESS_QUERY_INFORMATION | PROCESS_VM_READ, false, process_id) {
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
                if !exe_name.is_empty() { Some(exe_name.clone()) } else { None }
            };

            let icon = extract_icon(&full_path_str);
            (final_name, icon, Some(full_path_str))
        } else {
            (if !exe_name.is_empty() { Some(exe_name.clone()) } else { None }, None, None)
        };

        let exe_val = if !exe_name.is_empty() { Some(exe_name) } else { None };
        (app_name, app_icon, exe_val, full_path, is_explicit)
    }
}

#[cfg(target_os = "windows")]
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
        let query_str = format!("\\StringFileInfo\\{:04x}{:04x}\\{}", lang_code, charset_code, key);
        let query = OsStr::new(&query_str).encode_wide().chain(std::iter::once(0)).collect::<Vec<u16>>();

        let mut desc_ptr: *mut c_void = std::ptr::null_mut();
        let mut desc_len: u32 = 0;

        if VerQueryValueW(data.as_ptr() as *const _, windows::core::PCWSTR(query.as_ptr()), &mut desc_ptr, &mut desc_len).as_bool() {
             let desc = std::slice::from_raw_parts(desc_ptr as *const u16, desc_len as usize);
             let len = if desc.last() == Some(&0) { desc.len() - 1 } else { desc.len() };
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
    impl Drop for IconGuard { fn drop(&mut self) { unsafe { let _ = DestroyIcon(self.0); } } }
    let _guard = IconGuard(icon);

    let mut icon_info = ICONINFO::default();
    if GetIconInfo(icon, &mut icon_info).is_err() { return None; }

    struct BitmapGuard(HBITMAP);
    impl Drop for BitmapGuard { fn drop(&mut self) { unsafe { if !self.0.is_invalid() { let _ = DeleteObject(self.0.into()); } } } }
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

    GetDIBits(mem_dc, mem_bm, 0, height as u32, Some(pixels.as_mut_ptr() as *mut _), &mut BITMAPINFO { bmiHeader: bi, ..Default::default() }, DIB_RGB_COLORS);

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
    encoder.write_image(&pixels, width as u32, height as u32, image::ColorType::Rgba8).ok()?;

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
fn get_clipboard_owner_app_info() -> (Option<String>, Option<String>, Option<String>, Option<String>, bool) {
    use std::process::Command;

    let output = Command::new("osascript")
        .args(["-e", "tell application \"System Events\" to get {name, bundle identifier} of first application process whose frontmost is true"])
        .output();

    match output {
        Ok(out) if out.status.success() => {
            let result = String::from_utf8_lossy(&out.stdout).trim().to_string();
            // Result format: "AppName, com.example.app"
            let parts: Vec<&str> = result.splitn(2, ", ").collect();
            let app_name = parts.first().map(|s| s.to_string());
            let bundle_id = parts.get(1).map(|s| s.to_string());

            let icon = bundle_id.as_ref().and_then(|bid| extract_macos_app_icon(bid));

            (app_name, icon, bundle_id.clone(), bundle_id, true)
        }
        _ => {
            log::warn!("CLIPBOARD: Failed to get frontmost app info via osascript");
            (None, None, None, None, false)
        }
    }
}

#[cfg(target_os = "macos")]
fn extract_macos_app_icon(bundle_id: &str) -> Option<String> {
    use std::process::Command;

    // Find the .app bundle path from bundle ID
    let mdfind_output = Command::new("mdfind")
        .args([&format!("kMDItemCFBundleIdentifier == '{}'", bundle_id)])
        .output()
        .ok()?;

    let app_path = String::from_utf8_lossy(&mdfind_output.stdout)
        .lines()
        .find(|l| l.ends_with(".app"))
        .map(|s| s.to_string())?;

    // Read CFBundleIconFile from Info.plist
    let plist_path = format!("{}/Contents/Info.plist", app_path);
    let icon_name_output = Command::new("/usr/libexec/PlistBuddy")
        .args(["-c", "Print :CFBundleIconFile", &plist_path])
        .output()
        .ok()?;

    if !icon_name_output.status.success() {
        // Try CFBundleIconName as fallback
        let icon_name_output2 = Command::new("/usr/libexec/PlistBuddy")
            .args(["-c", "Print :CFBundleIconName", &plist_path])
            .output()
            .ok()?;
        if !icon_name_output2.status.success() {
            return None;
        }
    }

    let icon_name_raw = String::from_utf8_lossy(&icon_name_output.stdout).trim().to_string();
    // Append .icns if not present
    let icon_file = if icon_name_raw.ends_with(".icns") {
        icon_name_raw
    } else {
        format!("{}.icns", icon_name_raw)
    };

    let icns_path = format!("{}/Contents/Resources/{}", app_path, icon_file);

    // Check if the icns file exists
    if !std::path::Path::new(&icns_path).exists() {
        return None;
    }

    // Convert .icns to PNG via sips (built-in macOS tool)
    let tmp_png = format!("/tmp/pastepaw_icon_{}.png", bundle_id.replace('.', "_"));
    let sips_result = Command::new("sips")
        .args(["-s", "format", "png", "--resampleWidth", "64", &icns_path, "--out", &tmp_png])
        .output()
        .ok()?;

    if !sips_result.status.success() {
        return None;
    }

    // Read and encode as base64
    let png_data = std::fs::read(&tmp_png).ok()?;
    let _ = std::fs::remove_file(&tmp_png);

    Some(BASE64.encode(&png_data))
}

#[cfg(target_os = "macos")]
pub fn write_png_to_pasteboard(png_bytes: &[u8]) -> Result<(), String> {
    use cocoa::base::{nil, id, BOOL};
    use cocoa::foundation::NSString;
    use objc::{msg_send, sel, sel_impl, class};

    // Write PNG to a temp file so target apps can read via file URL (fast SSD path)
    let tmp_dir = std::env::temp_dir().join("pastepaw_images");
    let _ = std::fs::create_dir_all(&tmp_dir);
    let tmp_path = tmp_dir.join(format!("paste_{}.png", std::process::id()));
    std::fs::write(&tmp_path, png_bytes)
        .map_err(|e| format!("Failed to write temp PNG: {}", e))?;
    let file_url_str = format!("file://{}", tmp_path.to_string_lossy());

    unsafe {
        let pasteboard: id = msg_send![class!(NSPasteboard), generalPasteboard];
        let _: i64 = msg_send![pasteboard, clearContents];

        let png_data: id = msg_send![class!(NSData), dataWithBytes:png_bytes.as_ptr() length:png_bytes.len()];
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
    use core_graphics::event::{CGEvent, CGEventTapLocation, CGEventFlags, CGKeyCode};
    use core_graphics::event_source::{CGEventSource, CGEventSourceStateID};

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
