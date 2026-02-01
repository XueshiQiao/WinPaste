use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use std::sync::OnceLock;

pub enum ClipType {
    Text,
    Image,
    Html,
    Rtf,
    File,
    Url,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Clip {
    pub id: i64,
    pub uuid: String,
    pub clip_type: String,
    pub content: Vec<u8>,
    pub text_preview: String,
    pub content_hash: String,
    pub folder_id: Option<i64>,
    pub is_deleted: bool,
    pub source_app: Option<String>,
    pub source_icon: Option<String>,
    pub metadata: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub last_accessed: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Folder {
    pub id: i64,
    pub name: String,
    pub icon: Option<String>,
    pub color: Option<String>,
    pub is_system: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub max_items: i64,
    pub auto_delete_days: i64,
    pub startup_with_windows: bool,
    pub show_in_taskbar: bool,
    pub hotkey: String,
    pub theme: String,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            max_items: 1000,
            auto_delete_days: 30,
            startup_with_windows: true,
            show_in_taskbar: false,
            hotkey: "Ctrl+Shift+V".to_string(),
            theme: "dark".to_string(),
        }
    }
}

static DB_PATH: OnceLock<String> = OnceLock::new();

pub fn set_db_path(path: String) {
    DB_PATH.set(path).ok();
}

pub fn get_db_path() -> &'static str {
    DB_PATH.get().map(|s| s.as_str()).unwrap_or("")
}

static RUNTIME: OnceLock<tokio::runtime::Runtime> = OnceLock::new();

pub fn get_runtime() -> Result<&'static tokio::runtime::Runtime, String> {
    if let Some(rt) = RUNTIME.get() {
        return Ok(rt);
    }

    let rt = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .enable_all()
        .build()
        .map_err(|e| e.to_string())?;

    RUNTIME.set(rt).ok();
    Ok(RUNTIME.get().unwrap())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClipboardItem {
    pub id: String,
    pub clip_type: String,
    pub content: String,
    pub preview: String,
    pub folder_id: Option<String>,
    pub created_at: String,
    pub source_app: Option<String>,
    pub source_icon: Option<String>,
    pub metadata: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FolderItem {
    pub id: String,
    pub name: String,
    pub icon: Option<String>,
    pub color: Option<String>,
    pub is_system: bool,
    pub item_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub clips: Vec<ClipboardItem>,
    pub total_count: i64,
}
