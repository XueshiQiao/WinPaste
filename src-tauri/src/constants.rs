// keep sync with frontend (constants.ts)
#[cfg(target_os = "macos")]
pub const WINDOW_HEIGHT: f64 = 302.0;  // less wrapper padding on macOS (4px vs 32px)

#[cfg(not(target_os = "macos"))]
pub const WINDOW_HEIGHT: f64 = 330.0;

pub const WINDOW_MARGIN: f64 = 0.0;
