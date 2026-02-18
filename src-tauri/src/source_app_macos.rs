use cocoa::base::{id, nil};
use cocoa::foundation::NSString;
use objc::{class, msg_send, sel, sel_impl};
use objc::declare::ClassDecl;
use objc::runtime::{Object, Sel};
use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;
use std::sync::Mutex;
use once_cell::sync::Lazy;

const OWN_BUNDLE_ID: &str = "me.xueshi.pastepaw";
const OWN_APP_NAME: &str = "PastePaw";

fn is_own_app(name: &Option<String>, bundle_id: &Option<String>) -> bool {
    if let Some(ref bid) = bundle_id {
        if bid.eq_ignore_ascii_case(OWN_BUNDLE_ID) {
            return true;
        }
    }
    if let Some(ref n) = name {
        if n == OWN_APP_NAME {
            return true;
        }
    }
    false
}

/// Cached info about the last non-PastePaw foreground app.
struct CachedAppInfo {
    name: Option<String>,
    icon_base64: Option<String>,
    bundle_id: Option<String>,
}

static LAST_FOREGROUND_APP: Lazy<Mutex<Option<CachedAppInfo>>> = Lazy::new(|| Mutex::new(None));

/// Start observing NSWorkspace.didActivateApplicationNotification so we always know
/// the last non-PastePaw app that was in the foreground. Must be called once at startup.
pub fn start_frontmost_app_observer() {
    unsafe {
        let superclass = class!(NSObject);
        let mut decl = ClassDecl::new("PastePawAppObserver", superclass)
            .expect("Failed to create PastePawAppObserver class");

        extern "C" fn handle_app_activated(_self: &Object, _cmd: Sel, notification: id) {
            unsafe {
                let user_info: id = msg_send![notification, userInfo];
                if user_info == nil { return; }

                let key = cocoa::foundation::NSString::alloc(nil)
                    .init_str("NSWorkspaceApplicationKey");
                let app: id = msg_send![user_info, objectForKey: key];
                if app == nil { return; }

                let ns_name: id = msg_send![app, localizedName];
                let name = nsstring_to_string(ns_name);

                let ns_bundle_id: id = msg_send![app, bundleIdentifier];
                let bundle_id = nsstring_to_string(ns_bundle_id);

                // Skip PastePaw — we only want to remember other apps
                if is_own_app(&name, &bundle_id) {
                    return;
                }

                let ns_icon: id = msg_send![app, icon];
                let icon_base64 = if ns_icon != nil {
                    nsimage_to_base64_png(ns_icon, 64.0)
                } else {
                    None
                };

                log::debug!("CLIPBOARD: App activated (tracked): {:?} ({:?})", name, bundle_id);

                if let Ok(mut lock) = LAST_FOREGROUND_APP.lock() {
                    *lock = Some(CachedAppInfo {
                        name,
                        icon_base64,
                        bundle_id,
                    });
                }
            }
        }

        decl.add_method(
            sel!(appDidActivate:),
            handle_app_activated as extern "C" fn(&Object, Sel, id),
        );

        let observer_class = decl.register();
        let observer: id = msg_send![observer_class, new];
        // Prevent the observer from being released — it must live for the entire app lifetime.
        // Retain it so ARC won't deallocate it.
        let _: () = msg_send![observer, retain];

        let workspace: id = msg_send![class!(NSWorkspace), sharedWorkspace];
        let nc: id = msg_send![workspace, notificationCenter];
        let notif_name = cocoa::foundation::NSString::alloc(nil)
            .init_str("NSWorkspaceDidActivateApplicationNotification");

        let _: () = msg_send![nc,
            addObserver: observer
            selector: sel!(appDidActivate:)
            name: notif_name
            object: nil
        ];

        log::info!("CLIPBOARD: Started frontmost app observer");
    }
}

/// Returns (app_name, icon_base64, bundle_id, bundle_id, true) for the source app.
/// Uses native Cocoa APIs (NSWorkspace) instead of subprocess calls for App Store compliance.
///
/// If the current frontmost app is PastePaw, falls back to the last tracked non-PastePaw app
/// from the activation observer, eliminating the race condition where the user switches to
/// PastePaw before the clipboard event is processed.
pub fn get_frontmost_app_info() -> (Option<String>, Option<String>, Option<String>, Option<String>, bool) {
    unsafe {
        let workspace: id = msg_send![class!(NSWorkspace), sharedWorkspace];
        let front_app: id = msg_send![workspace, frontmostApplication];

        if front_app == nil {
            log::warn!("CLIPBOARD: Failed to get frontmost application via NSWorkspace");
            return (None, None, None, None, false);
        }

        let ns_name: id = msg_send![front_app, localizedName];
        let app_name = nsstring_to_string(ns_name);

        let ns_bundle_id: id = msg_send![front_app, bundleIdentifier];
        let bundle_id = nsstring_to_string(ns_bundle_id);

        // If frontmost app is NOT PastePaw, use it directly
        if !is_own_app(&app_name, &bundle_id) {

            let ns_icon: id = msg_send![front_app, icon];
            let icon_base64 = if ns_icon != nil {
                nsimage_to_base64_png(ns_icon, 64.0)
            } else {
                None
            };

            log::info!("CLIPBOARD: Source app (frontmost): {:?} ({:?})", app_name, bundle_id);
            return (app_name, icon_base64, bundle_id.clone(), bundle_id, true);
        }

        // Frontmost is PastePaw — fall back to last tracked foreground app
        log::debug!("CLIPBOARD: Frontmost is PastePaw, using last tracked foreground app");
        if let Ok(lock) = LAST_FOREGROUND_APP.lock() {
            if let Some(ref cached) = *lock {
                log::info!("CLIPBOARD: Source app (cached): {:?} ({:?})", cached.name, cached.bundle_id);
                return (
                    cached.name.clone(),
                    cached.icon_base64.clone(),
                    cached.bundle_id.clone(),
                    cached.bundle_id.clone(),
                    true,
                );
            }
        }

        log::warn!("CLIPBOARD: No cached foreground app available");
        (None, None, None, None, false)
    }
}

/// Convert an NSString to a Rust String, returning None if nil.
unsafe fn nsstring_to_string(nsstr: id) -> Option<String> {
    if nsstr == nil {
        return None;
    }
    let utf8: *const std::os::raw::c_char = msg_send![nsstr, UTF8String];
    if utf8.is_null() {
        return None;
    }
    Some(std::ffi::CStr::from_ptr(utf8).to_string_lossy().into_owned())
}

/// Convert an NSImage to a base64-encoded PNG string at the given size.
fn nsimage_to_base64_png(source_image: id, size: f64) -> Option<String> {
    unsafe {
        let target_size = cocoa::foundation::NSSize::new(size, size);

        let resized: id = msg_send![class!(NSImage), alloc];
        let resized: id = msg_send![resized, initWithSize: target_size];

        resized.lockFocus();
        let source_size: cocoa::foundation::NSSize = msg_send![source_image, size];
        let src_rect = cocoa::foundation::NSRect::new(
            cocoa::foundation::NSPoint::new(0.0, 0.0),
            source_size,
        );
        let dst_rect = cocoa::foundation::NSRect::new(
            cocoa::foundation::NSPoint::new(0.0, 0.0),
            target_size,
        );
        // NSCompositingOperationCopy = 1
        let _: () = msg_send![source_image,
            drawInRect: dst_rect
            fromRect: src_rect
            operation: 1i64
            fraction: 1.0f64
        ];
        resized.unlockFocus();

        let tiff_data: id = msg_send![resized, TIFFRepresentation];
        if tiff_data == nil {
            let _: () = msg_send![resized, release];
            return None;
        }

        let bitmap_rep: id = msg_send![class!(NSBitmapImageRep), imageRepWithData: tiff_data];
        if bitmap_rep == nil {
            let _: () = msg_send![resized, release];
            return None;
        }

        // NSBitmapImageFileTypePNG = 4
        let empty_dict: id = msg_send![class!(NSDictionary), dictionary];
        let png_data: id = msg_send![bitmap_rep,
            representationUsingType: 4u64
            properties: empty_dict
        ];

        if png_data == nil {
            let _: () = msg_send![resized, release];
            return None;
        }

        let length: usize = msg_send![png_data, length];
        let bytes: *const u8 = msg_send![png_data, bytes];

        if bytes.is_null() || length == 0 {
            let _: () = msg_send![resized, release];
            return None;
        }

        let slice = std::slice::from_raw_parts(bytes, length);
        let encoded = BASE64.encode(slice);

        let _: () = msg_send![resized, release];

        Some(encoded)
    }
}

/// Checks if the current process is a "trusted" accessibility client.
pub fn is_accessibility_enabled() -> bool {
    // AXIsProcessTrustedWithOptions is available on macOS 10.9+
    // Passing nil/null as options means we don't want to show the system prompt immediately
    // (we'll handle the prompting via our own UI or by opening settings).
    objc_is_process_trusted()
}

/// Opens the System Settings to the Accessibility > Input Monitoring or Accessibility page.
pub fn open_accessibility_settings() {
    unsafe {
        let workspace: id = msg_send![class!(NSWorkspace), sharedWorkspace];
        let url_string = cocoa::foundation::NSString::alloc(nil)
            .init_str("x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility");
        let url: id = msg_send![class!(NSURL), URLWithString: url_string];
        let _: () = msg_send![workspace, openURL: url];
    }
}

extern "C" {
    fn AXIsProcessTrusted() -> bool;
}

fn objc_is_process_trusted() -> bool {
    unsafe { AXIsProcessTrusted() }
}

trait NSImageFocusExt {
    unsafe fn lockFocus(self);
    unsafe fn unlockFocus(self);
}

impl NSImageFocusExt for id {
    unsafe fn lockFocus(self) {
        let _: () = msg_send![self, lockFocus];
    }
    unsafe fn unlockFocus(self) {
        let _: () = msg_send![self, unlockFocus];
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_frontmost_app_info() {
        let (name, _icon, bundle_id, _full_path, _explicit) = get_frontmost_app_info();
        println!("App name: {:?}", name);
        println!("Bundle ID: {:?}", bundle_id);
    }

    #[test]
    fn test_is_accessibility_enabled() {
        let enabled = is_accessibility_enabled();
        println!("Accessibility enabled: {}", enabled);
    }

    #[test]
    #[cfg(feature = "app-store")]
    fn test_smappservice_status() {
        use smappservice_rs::{AppService, ServiceType};
        let app_service = AppService::new(ServiceType::MainApp);
        let status = app_service.status();
        println!("SMAppService status: {:?}", status);
        // We don't assert Registered because it won't be in a test environment
    }
}
