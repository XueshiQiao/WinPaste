use crate::models::AppSettings;
use crate::database::Database;
use std::sync::RwLock;
use std::path::PathBuf;
use std::fs;
use tauri::AppHandle;
use tauri::Manager;

pub struct SettingsManager {
    file_path: PathBuf,
    settings: RwLock<AppSettings>,
}

impl SettingsManager {
    pub async fn new(app: &AppHandle, db: &Database) -> Self {
        let path = app.path().app_data_dir().unwrap().join("settings.json");
        let settings = if path.exists() {
            // Load from file
            match fs::read_to_string(&path) {
                Ok(content) => serde_json::from_str(&content).unwrap_or_default(),
                Err(_) => AppSettings::default(),
            }
        } else {
            // Migrate from SQLite or use default
            Self::migrate_from_sqlite(db).await
        };

        // Ensure we save it once immediately if migrating, so file exists
        let manager = Self {
            file_path: path,
            settings: RwLock::new(settings.clone()),
        };
        if !manager.file_path.exists() {
            let _ = manager.save(settings);
        }
        manager
    }

    async fn migrate_from_sqlite(db: &Database) -> AppSettings {
        let mut settings = AppSettings::default();
        let pool = &db.pool;

        async fn get_val(pool: &sqlx::SqlitePool, key: &str) -> Option<String> {
            sqlx::query_scalar::<_, String>("SELECT value FROM settings WHERE key = ?")
                .bind(key)
                .fetch_optional(pool)
                .await
                .unwrap_or(None)
        }

        if let Some(v) = get_val(pool, "theme").await { settings.theme = v; }
        if let Some(v) = get_val(pool, "mica_effect").await { settings.mica_effect = v; }
        if let Some(v) = get_val(pool, "language").await { settings.language = v; }

        if let Some(v) = get_val(pool, "max_items").await { if let Ok(i) = v.parse() { settings.max_items = i; } }
        if let Some(v) = get_val(pool, "auto_delete_days").await { if let Ok(i) = v.parse() { settings.auto_delete_days = i; } }
        if let Some(v) = get_val(pool, "hotkey").await { settings.hotkey = v; }

        if let Some(v) = get_val(pool, "auto_paste").await { if let Ok(b) = v.parse() { settings.auto_paste = b; } }
        if let Some(v) = get_val(pool, "ignore_ghost_clips").await { if let Ok(b) = v.parse() { settings.ignore_ghost_clips = b; } }

        // AI
        if let Some(v) = get_val(pool, "ai_provider").await { settings.ai_provider = v; }
        if let Some(v) = get_val(pool, "ai_api_key").await { settings.ai_api_key = v; }
        if let Some(v) = get_val(pool, "ai_model").await { settings.ai_model = v; }
        if let Some(v) = get_val(pool, "ai_base_url").await { settings.ai_base_url = v; }

        if let Some(v) = get_val(pool, "ai_prompt_summarize").await { settings.ai_prompt_summarize = v; }
        if let Some(v) = get_val(pool, "ai_prompt_translate").await { settings.ai_prompt_translate = v; }
        if let Some(v) = get_val(pool, "ai_prompt_explain_code").await { settings.ai_prompt_explain_code = v; }
        if let Some(v) = get_val(pool, "ai_prompt_fix_grammar").await { settings.ai_prompt_fix_grammar = v; }

        if let Some(v) = get_val(pool, "ai_title_summarize").await { settings.ai_title_summarize = v; }
        if let Some(v) = get_val(pool, "ai_title_translate").await { settings.ai_title_translate = v; }
        if let Some(v) = get_val(pool, "ai_title_explain_code").await { settings.ai_title_explain_code = v; }
        if let Some(v) = get_val(pool, "ai_title_fix_grammar").await { settings.ai_title_fix_grammar = v; }

        // Ignored Apps
        if let Ok(apps) = sqlx::query_scalar::<_, String>("SELECT app_name FROM ignored_apps").fetch_all(pool).await {
            settings.ignored_apps = apps.into_iter().collect();
        }

        settings
    }

    pub fn get(&self) -> AppSettings {
        self.settings.read().unwrap().clone()
    }

    pub fn save(&self, new_settings: AppSettings) -> Result<(), String> {
        {
            let mut lock = self.settings.write().unwrap();
            *lock = new_settings.clone();
        }
        // TODO - what happens if multiple threads call save at the same time?
        let json = serde_json::to_string_pretty(&new_settings).map_err(|e| e.to_string())?;
        fs::write(&self.file_path, json).map_err(|e| e.to_string())?;

        Ok(())
    }
}
