use sqlx::SqlitePool;

#[derive(Clone)]
pub struct Database {
    pub pool: SqlitePool,
}

impl Database {
    pub async fn new(db_path: &str) -> Self {
        let options = sqlx::sqlite::SqliteConnectOptions::new()
            .filename(db_path)
            .create_if_missing(true);

        let pool = SqlitePool::connect_with(options).await.unwrap();

        Self { pool }
    }

    pub async fn migrate(&self) -> Result<(), sqlx::Error> {
        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS folders (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL,
                icon TEXT,
                color TEXT,
                is_system INTEGER DEFAULT 0,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP
            )
        "#).execute(&self.pool).await?;

        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS clips (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                uuid TEXT NOT NULL UNIQUE,
                clip_type TEXT NOT NULL,
                content BLOB NOT NULL,
                text_preview TEXT,
                content_hash TEXT NOT NULL,
                folder_id INTEGER REFERENCES folders(id),
                is_deleted INTEGER DEFAULT 0,
                source_app TEXT,
                source_icon TEXT,
                metadata TEXT,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                last_accessed DATETIME DEFAULT CURRENT_TIMESTAMP
            )
        "#).execute(&self.pool).await?;

        sqlx::query(r#"
            CREATE INDEX IF NOT EXISTS idx_clips_hash ON clips(content_hash);
        "#).execute(&self.pool).await?;

        sqlx::query(r#"
            CREATE INDEX IF NOT EXISTS idx_clips_folder ON clips(folder_id);
        "#).execute(&self.pool).await?;

        sqlx::query(r#"
            CREATE INDEX IF NOT EXISTS idx_clips_created ON clips(created_at);
        "#).execute(&self.pool).await?;

        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS settings (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            )
        "#).execute(&self.pool).await?;

        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS ignored_apps (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                app_name TEXT NOT NULL UNIQUE
            )
        "#).execute(&self.pool).await?;

       Ok(())
    }

    pub async fn add_ignored_app(&self, app_name: &str) -> Result<(), sqlx::Error> {
        sqlx::query("INSERT OR IGNORE INTO ignored_apps (app_name) VALUES (?)")
            .bind(app_name)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn remove_ignored_app(&self, app_name: &str) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM ignored_apps WHERE app_name = ?")
            .bind(app_name)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn get_ignored_apps(&self) -> Result<Vec<String>, sqlx::Error> {
        let apps = sqlx::query_scalar::<_, String>("SELECT app_name FROM ignored_apps ORDER BY app_name")
            .fetch_all(&self.pool)
            .await?;
        log::info!("DB: Ignored apps: {:?}", apps);
        Ok(apps)
    }

    pub async fn is_app_ignored(&self, app_name: &str) -> Result<bool, sqlx::Error> {
        // Case-insensitive check might be better for Windows exe names
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM ignored_apps WHERE LOWER(app_name) = LOWER(?)")
            .bind(app_name)
            .fetch_one(&self.pool)
            .await?;
        Ok(count > 0)
    }

    pub async fn get_setting(&self, key: &str) -> Result<Option<String>, sqlx::Error> {
        let value = sqlx::query_scalar(r#"
            SELECT value FROM settings WHERE key = ?
        "#)
        .bind(key)
        .fetch_optional(&self.pool)
        .await?;

        Ok(value)
    }
}
