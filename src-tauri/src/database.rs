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
}
