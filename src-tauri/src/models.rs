use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use std::sync::OnceLock;

use std::collections::HashSet;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AppSettings {
    pub theme: String,
    pub mica_effect: String,
    pub language: String,
    pub max_items: i64,
    pub auto_delete_days: i64,
    pub hotkey: String,
    pub auto_paste: bool,
    pub ignore_ghost_clips: bool,
    pub startup_with_windows: bool,
    
    // AI
    pub ai_provider: String,
    pub ai_api_key: String,
    pub ai_model: String,
    pub ai_base_url: String,
    pub ai_prompt_summarize: String,
    pub ai_prompt_translate: String,
    pub ai_prompt_explain_code: String,
    pub ai_prompt_fix_grammar: String,
    pub ai_title_summarize: String,
    pub ai_title_translate: String,
    pub ai_title_explain_code: String,
    pub ai_title_fix_grammar: String,

    // Privacy
    pub ignored_apps: HashSet<String>,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            theme: "system".to_string(),
            mica_effect: "clear".to_string(),
            language: "en".to_string(),
            max_items: 1000,
            auto_delete_days: 30,
            hotkey: if cfg!(target_os = "macos") { "Cmd+Shift+V" } else { "Ctrl+Shift+V" }.to_string(),
            auto_paste: false,
            ignore_ghost_clips: false,
            startup_with_windows: false,
            
            ai_provider: "openai".to_string(),
            ai_api_key: "".to_string(),
            ai_model: "gpt-3.5-turbo".to_string(),
            ai_base_url: "https://api.openai.com/v1".to_string(),
            
            ai_prompt_summarize: "Summarize this content concisely.".to_string(),
            ai_prompt_translate: "Translate this to English (or user language).".to_string(),
            ai_prompt_explain_code: "Explain this code snippet.".to_string(),
            ai_prompt_fix_grammar: "Fix grammar and spelling.".to_string(),
            
            ai_title_summarize: "Summarize".to_string(),
            ai_title_translate: "Translate".to_string(),
            ai_title_explain_code: "Explain Code".to_string(),
            ai_title_fix_grammar: "Fix Grammar".to_string(),

            ignored_apps: HashSet::new(),
        }
    }
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
    pub is_thumbnail: bool,
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
