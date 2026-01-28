use sqlx::SqlitePool;
use crate::models::{Clip, Folder};

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

        // Add source_icon column if it doesn't exist (migrations for existing dbs)
        sqlx::query("ALTER TABLE clips ADD COLUMN source_icon TEXT").execute(&self.pool).await.ok();

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
            INSERT OR IGNORE INTO folders (id, name, is_system) VALUES (1, 'All', 1);
        "#).execute(&self.pool).await?;

        // Cleanup Pinned folder if exists
        sqlx::query("DELETE FROM folders WHERE id = 2").execute(&self.pool).await.ok();

        sqlx::query(r#"
            INSERT OR IGNORE INTO folders (id, name, is_system) VALUES (3, 'Recent', 1);
        "#).execute(&self.pool).await?;

        Ok(())
    }

    pub async fn add_clip(&self, clip: &Clip) -> Result<i64, sqlx::Error> {
        let id = sqlx::query(r#"
            INSERT INTO clips (uuid, clip_type, content, text_preview, content_hash, folder_id, source_app, source_icon, metadata)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#)
        .bind(&clip.uuid)
        .bind(&clip.clip_type)
        .bind(&clip.content)
        .bind(&clip.text_preview)
        .bind(&clip.content_hash)
        .bind(clip.folder_id)
        .bind(clip.folder_id)
        .bind(&clip.source_app)
        .bind(&clip.source_icon)
        .bind(&clip.metadata)
        .execute(&self.pool)
        .await?
        .last_insert_rowid();

        Ok(id)
    }

    pub async fn get_clips(&self, folder_id: Option<i64>, limit: i64, offset: i64) -> Result<Vec<Clip>, sqlx::Error> {
        let clips = match folder_id {
            Some(id) => {
                sqlx::query_as(r#"
                    SELECT * FROM clips WHERE is_deleted = 0 AND folder_id = ?
                    ORDER BY created_at DESC LIMIT ? OFFSET ?
                "#)
                .bind(id)
                .bind(limit)
                .bind(offset)
                .fetch_all(&self.pool)
                .await?
            }
            None => {
                sqlx::query_as(r#"
                    SELECT * FROM clips WHERE is_deleted = 0
                    ORDER BY created_at DESC LIMIT ? OFFSET ?
                "#)
                .bind(limit)
                .bind(offset)
                .fetch_all(&self.pool)
                .await?
            }
        };

        Ok(clips)
    }

    pub async fn get_clip_by_id(&self, id: i64) -> Result<Option<Clip>, sqlx::Error> {
        let clip = sqlx::query_as(r#"
            SELECT * FROM clips WHERE id = ? AND is_deleted = 0
        "#)
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(clip)
    }

    pub async fn get_clip_by_uuid(&self, uuid: &str) -> Result<Option<Clip>, sqlx::Error> {
        let clip = sqlx::query_as(r#"
            SELECT * FROM clips WHERE uuid = ? AND is_deleted = 0
        "#)
        .bind(uuid)
        .fetch_optional(&self.pool)
        .await?;

        Ok(clip)
    }

    pub async fn get_clip_by_hash(&self, hash: &str) -> Result<Option<Clip>, sqlx::Error> {
        let clip = sqlx::query_as(r#"
            SELECT * FROM clips WHERE content_hash = ? AND is_deleted = 0
            ORDER BY created_at DESC LIMIT 1
        "#)
        .bind(hash)
        .fetch_optional(&self.pool)
        .await?;

        Ok(clip)
    }

    pub async fn update_clip_folder(&self, clip_id: i64, folder_id: Option<i64>) -> Result<(), sqlx::Error> {
        sqlx::query(r#"
            UPDATE clips SET folder_id = ? WHERE id = ?
        "#)
        .bind(folder_id)
        .bind(clip_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }



    pub async fn delete_clip(&self, clip_id: i64) -> Result<(), sqlx::Error> {
        sqlx::query(r#"
            UPDATE clips SET is_deleted = 1 WHERE id = ?
        "#)
        .bind(clip_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn hard_delete_clip(&self, clip_id: i64) -> Result<(), sqlx::Error> {
        sqlx::query(r#"
            DELETE FROM clips WHERE id = ?
        "#)
        .bind(clip_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn get_folders(&self) -> Result<Vec<Folder>, sqlx::Error> {
        let folders = sqlx::query_as(r#"
            SELECT * FROM folders ORDER BY is_system DESC, created_at DESC
        "#)
        .fetch_all(&self.pool)
        .await?;

        Ok(folders)
    }

    pub async fn create_folder(&self, name: &str, icon: Option<&str>, color: Option<&str>) -> Result<i64, sqlx::Error> {
        let id = sqlx::query(r#"
            INSERT INTO folders (name, icon, color, is_system)
            VALUES (?, ?, ?, 0)
        "#)
        .bind(name)
        .bind(icon)
        .bind(color)
        .execute(&self.pool)
        .await?
        .last_insert_rowid();

        Ok(id)
    }

    pub async fn delete_folder(&self, folder_id: i64) -> Result<(), sqlx::Error> {
        sqlx::query(r#"
            DELETE FROM folders WHERE id = ? AND is_system = 0
        "#)
        .bind(folder_id)
        .execute(&self.pool)
        .await?;

        sqlx::query(r#"
            UPDATE clips SET folder_id = 1 WHERE folder_id = ?
        "#)
        .bind(folder_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn search_clips(&self, query: &str, limit: i64) -> Result<Vec<Clip>, sqlx::Error> {
        let search_pattern = format!("%{}%", query);
        let clips = sqlx::query_as(r#"
            SELECT * FROM clips
            WHERE is_deleted = 0 AND text_preview LIKE ?
            ORDER BY created_at DESC
            LIMIT ?
        "#)
        .bind(&search_pattern)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(clips)
    }

    pub async fn get_clipboard_history_size(&self) -> Result<i64, sqlx::Error> {
        let count = sqlx::query_scalar(r#"
            SELECT COUNT(*) FROM clips WHERE is_deleted = 0
        "#)
        .fetch_one(&self.pool)
        .await?;

        Ok(count)
    }

    pub async fn clear_history(&self) -> Result<(), sqlx::Error> {
        sqlx::query(r#"
            UPDATE clips SET is_deleted = 1
        "#)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn delete_old_clips(&self, days: i64) -> Result<(), sqlx::Error> {
        sqlx::query(r#"
            UPDATE clips SET is_deleted = 1
            WHERE is_deleted = 0
            AND created_at < datetime('now', ?)
        "#)
        .bind(format!("-{} days", days))
        .execute(&self.pool)
        .await?;

        Ok(())
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

    pub async fn set_setting(&self, key: &str, value: &str) -> Result<(), sqlx::Error> {
        sqlx::query(r#"
            INSERT OR REPLACE INTO settings (key, value) VALUES (?, ?)
        "#)
        .bind(key)
        .bind(value)
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}
