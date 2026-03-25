use crate::models::{ActionResponse, AppStatus, ClipboardItem, ClipboardItemType, ClipboardRow, HistoryResponse, ImageSize};
use anyhow::{Context, Result};
use base64::{engine::general_purpose::STANDARD as Base64Engine, Engine as _};
use clipboard_rs::common::{RustImage, RustImageData};
use clipboard_rs::{Clipboard, ClipboardContext};
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions, SqliteSynchronous};
use sqlx::{query, query_as, SqlitePool};
use std::path::{Path, PathBuf};

const HISTORY_LIMIT: i64 = 500;
const HOTKEY_LABEL: &str = "Ctrl+Shift+V";

#[derive(Clone)]
pub struct AppStore {
    pool: SqlitePool,
    images_dir: PathBuf,
}

impl AppStore {
    pub async fn new(data_dir: &Path) -> Result<Self> {
        std::fs::create_dir_all(data_dir).context("failed to create app data directory")?;
        let images_dir = data_dir.join("images");
        std::fs::create_dir_all(&images_dir).context("failed to create image cache directory")?;

        let db_path = data_dir.join("paste.sqlite");
        let connect_options = SqliteConnectOptions::new()
            .filename(&db_path)
            .create_if_missing(true)
            .journal_mode(SqliteJournalMode::Wal)
            .synchronous(SqliteSynchronous::Normal);

        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect_with(connect_options)
            .await
            .context("failed to connect sqlite")?;

        let store = Self { pool, images_dir };
        store.migrate().await?;
        Ok(store)
    }

    async fn migrate(&self) -> Result<()> {
        query(
            r#"
            CREATE TABLE IF NOT EXISTS clipboard_items (
              id INTEGER PRIMARY KEY AUTOINCREMENT,
              item_type TEXT NOT NULL,
              content_text TEXT,
              image_path TEXT,
              content_hash TEXT NOT NULL UNIQUE,
              image_width INTEGER,
              image_height INTEGER,
              is_pinned INTEGER NOT NULL DEFAULT 0,
              created_at INTEGER NOT NULL,
              updated_at INTEGER NOT NULL
            );
            "#,
        )
        .execute(&self.pool)
        .await?;

        query(
            r#"
            CREATE INDEX IF NOT EXISTS idx_clipboard_items_updated_at
            ON clipboard_items(is_pinned DESC, updated_at DESC);
            "#,
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn upsert_text(&self, text: String) -> Result<Option<ClipboardItem>> {
        let normalized = normalize_text(&text);
        if normalized.is_empty() {
            return Ok(None);
        }

        let now = unix_ms();
        let content_hash = hash_string(&format!("text:{normalized}"));

        let existing = query_as::<_, ClipboardRow>(
            r#"
            SELECT id, item_type, content_text, image_path, content_hash, image_width, image_height, is_pinned, created_at, updated_at
            FROM clipboard_items
            WHERE content_hash = ?
            "#,
        )
        .bind(&content_hash)
        .fetch_optional(&self.pool)
        .await?;

        let row = if let Some(existing_row) = existing {
            query(
                r#"
                UPDATE clipboard_items
                SET content_text = ?, updated_at = ?
                WHERE id = ?
                "#,
            )
            .bind(&normalized)
            .bind(now)
            .bind(existing_row.id)
            .execute(&self.pool)
            .await?;

            self.fetch_item(existing_row.id).await?
        } else {
            let result = query(
                r#"
                INSERT INTO clipboard_items (
                  item_type, content_text, image_path, content_hash, image_width, image_height, is_pinned, created_at, updated_at
                ) VALUES (?, ?, NULL, ?, NULL, NULL, 0, ?, ?)
                "#,
            )
            .bind("text")
            .bind(&normalized)
            .bind(&content_hash)
            .bind(now)
            .bind(now)
            .execute(&self.pool)
            .await?;

            let item_id = result.last_insert_rowid();
            self.fetch_item(item_id).await?
        };

        self.prune_unpinned().await?;
        Ok(Some(self.row_to_item(row)))
    }

    pub async fn upsert_image(&self, image: RustImageData) -> Result<Option<ClipboardItem>> {
        let png = image
            .to_png()
            .map_err(|error| anyhow::anyhow!(error.to_string()))?;
        let bytes = png.get_bytes();
        if bytes.is_empty() {
            return Ok(None);
        }

        let now = unix_ms();
        let content_hash = hash_bytes(bytes);
        let image_path = self.images_dir.join(format!("{content_hash}.png"));
        if !image_path.exists() {
            png.save_to_path(&image_path.to_string_lossy())
                .map_err(|error| anyhow::anyhow!(error.to_string()))?;
        }
        let (width, height) = image.get_size();

        let existing = query_as::<_, ClipboardRow>(
            r#"
            SELECT id, item_type, content_text, image_path, content_hash, image_width, image_height, is_pinned, created_at, updated_at
            FROM clipboard_items
            WHERE content_hash = ?
            "#,
        )
        .bind(&content_hash)
        .fetch_optional(&self.pool)
        .await?;

        let row = if let Some(existing_row) = existing {
            query(
                r#"
                UPDATE clipboard_items
                SET image_path = ?, image_width = ?, image_height = ?, updated_at = ?
                WHERE id = ?
                "#,
            )
            .bind(image_path.to_string_lossy().to_string())
            .bind(i64::from(width))
            .bind(i64::from(height))
            .bind(now)
            .bind(existing_row.id)
            .execute(&self.pool)
            .await?;

            self.fetch_item(existing_row.id).await?
        } else {
            let result = query(
                r#"
                INSERT INTO clipboard_items (
                  item_type, content_text, image_path, content_hash, image_width, image_height, is_pinned, created_at, updated_at
                ) VALUES (?, NULL, ?, ?, ?, ?, 0, ?, ?)
                "#,
            )
            .bind("image")
            .bind(image_path.to_string_lossy().to_string())
            .bind(&content_hash)
            .bind(i64::from(width))
            .bind(i64::from(height))
            .bind(now)
            .bind(now)
            .execute(&self.pool)
            .await?;

            let item_id = result.last_insert_rowid();
            self.fetch_item(item_id).await?
        };

        self.prune_unpinned().await?;
        Ok(Some(self.row_to_item(row)))
    }

    pub async fn list_items(&self, query_text: Option<String>) -> Result<HistoryResponse> {
        let items = if let Some(raw_query) = query_text {
            let trimmed = raw_query.trim().to_lowercase();
            if trimmed.is_empty() {
                self.fetch_all_items().await?
            } else {
                query_as::<_, ClipboardRow>(
                    r#"
                    SELECT id, item_type, content_text, image_path, content_hash, image_width, image_height, is_pinned, created_at, updated_at
                    FROM clipboard_items
                    WHERE item_type = 'text' AND lower(content_text) LIKE ?
                    ORDER BY is_pinned DESC, updated_at DESC
                    "#,
                )
                .bind(format!("%{trimmed}%"))
                .fetch_all(&self.pool)
                .await?
            }
        } else {
            self.fetch_all_items().await?
        };

        Ok(HistoryResponse {
            items: items.into_iter().map(|row| self.row_to_item(row)).collect(),
        })
    }

    pub async fn app_status(&self, monitoring_paused: bool) -> Result<AppStatus> {
        Ok(AppStatus {
            monitoring_paused,
            history_count: self.history_count().await?,
            hotkey: HOTKEY_LABEL.to_string(),
        })
    }

    pub async fn toggle_pin(&self, id: i64) -> Result<ActionResponse> {
        query(
            r#"
            UPDATE clipboard_items
            SET is_pinned = CASE WHEN is_pinned = 1 THEN 0 ELSE 1 END
            WHERE id = ?
            "#,
        )
        .bind(id)
        .execute(&self.pool)
        .await?;

        let item = self.fetch_item(id).await.map(|row| self.row_to_item(row)).ok();
        Ok(ActionResponse {
            ok: item.is_some(),
            item,
            history_count: self.history_count().await?,
        })
    }

    pub async fn delete_item(&self, id: i64) -> Result<ActionResponse> {
        if let Ok(row) = self.fetch_item(id).await {
            self.remove_image_file(&row)?;
            query("DELETE FROM clipboard_items WHERE id = ?")
                .bind(id)
                .execute(&self.pool)
                .await?;
        }

        Ok(ActionResponse {
            ok: true,
            item: None,
            history_count: self.history_count().await?,
        })
    }

    pub async fn clear_history(&self) -> Result<ActionResponse> {
        let rows = self.fetch_all_items().await?;
        for row in rows {
            self.remove_image_file(&row)?;
        }

        query("DELETE FROM clipboard_items").execute(&self.pool).await?;

        Ok(ActionResponse {
            ok: true,
            item: None,
            history_count: 0,
        })
    }

    pub async fn copy_item_to_clipboard(&self, id: i64) -> Result<ActionResponse> {
        let row = self.fetch_item(id).await?;
        let clipboard = ClipboardContext::new()
            .map_err(|error| anyhow::anyhow!("failed to create clipboard context: {error}"))?;

        match row.item_type() {
            ClipboardItemType::Text => {
                clipboard
                    .set_text(row.content_text.clone().unwrap_or_default())
                    .map_err(|error| anyhow::anyhow!(error.to_string()))?;
            }
            ClipboardItemType::Image => {
                let image_path = row.image_path.clone().context("image path missing")?;
                let image = RustImageData::from_path(&image_path)
                    .map_err(|error| anyhow::anyhow!(error.to_string()))?;
                clipboard
                    .set_image(image)
                    .map_err(|error| anyhow::anyhow!(error.to_string()))?;
            }
        }

        query("UPDATE clipboard_items SET updated_at = ? WHERE id = ?")
            .bind(unix_ms())
            .bind(id)
            .execute(&self.pool)
            .await?;

        let item = self.fetch_item(id).await.map(|updated| self.row_to_item(updated)).ok();

        Ok(ActionResponse {
            ok: true,
            item,
            history_count: self.history_count().await?,
        })
    }

    async fn fetch_all_items(&self) -> Result<Vec<ClipboardRow>> {
        let items = query_as::<_, ClipboardRow>(
            r#"
            SELECT id, item_type, content_text, image_path, content_hash, image_width, image_height, is_pinned, created_at, updated_at
            FROM clipboard_items
            ORDER BY is_pinned DESC, updated_at DESC
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(items)
    }

    async fn fetch_item(&self, id: i64) -> Result<ClipboardRow> {
        let item = query_as::<_, ClipboardRow>(
            r#"
            SELECT id, item_type, content_text, image_path, content_hash, image_width, image_height, is_pinned, created_at, updated_at
            FROM clipboard_items
            WHERE id = ?
            "#,
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await?;

        Ok(item)
    }

    async fn history_count(&self) -> Result<i64> {
        let (count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM clipboard_items")
            .fetch_one(&self.pool)
            .await?;
        Ok(count)
    }

    async fn prune_unpinned(&self) -> Result<()> {
        let rows = query_as::<_, ClipboardRow>(
            r#"
            SELECT id, item_type, content_text, image_path, content_hash, image_width, image_height, is_pinned, created_at, updated_at
            FROM clipboard_items
            WHERE is_pinned = 0
            ORDER BY updated_at DESC
            LIMIT -1 OFFSET ?
            "#,
        )
        .bind(HISTORY_LIMIT)
        .fetch_all(&self.pool)
        .await?;

        for row in rows {
            self.remove_image_file(&row)?;
            query("DELETE FROM clipboard_items WHERE id = ?")
                .bind(row.id)
                .execute(&self.pool)
                .await?;
        }

        Ok(())
    }

    fn row_to_item(&self, row: ClipboardRow) -> ClipboardItem {
        let image_size = match (row.image_width, row.image_height) {
            (Some(width), Some(height)) => Some(ImageSize {
                width: width.max(0) as u32,
                height: height.max(0) as u32,
            }),
            _ => None,
        };

        let image_url = row.image_path.as_ref().and_then(|path| self.thumbnail_data_url(path).ok());
        let preview = match row.item_type() {
            ClipboardItemType::Text => text_preview(row.content_text.as_deref().unwrap_or_default()),
            ClipboardItemType::Image => match &image_size {
                Some(size) => format!("{} x {} 图片", size.width, size.height),
                None => "图片".to_string(),
            },
        };

        ClipboardItem {
            id: row.id,
            item_type: row.item_type(),
            preview,
            is_pinned: row.is_pinned(),
            created_at: row.created_at,
            updated_at: row.updated_at,
            image_url,
            image_size,
        }
    }

    fn thumbnail_data_url(&self, image_path: &str) -> Result<String> {
        let image = RustImageData::from_path(image_path)
            .map_err(|error| anyhow::anyhow!(error.to_string()))?;
        let thumbnail = image
            .thumbnail(320, 180)
            .map_err(|error| anyhow::anyhow!(error.to_string()))?;
        let png = thumbnail
            .to_png()
            .map_err(|error| anyhow::anyhow!(error.to_string()))?;
        Ok(format!(
            "data:image/png;base64,{}",
            Base64Engine.encode(png.get_bytes())
        ))
    }

    fn remove_image_file(&self, row: &ClipboardRow) -> Result<()> {
        if let Some(image_path) = &row.image_path {
            let path = PathBuf::from(image_path);
            if path.exists() {
                std::fs::remove_file(path).ok();
            }
        }
        Ok(())
    }
}

fn normalize_text(text: &str) -> String {
    text.replace("\r\n", "\n").trim().to_string()
}

fn text_preview(text: &str) -> String {
    let single_line = text.split_whitespace().collect::<Vec<_>>().join(" ");
    if single_line.chars().count() > 140 {
        let preview = single_line.chars().take(140).collect::<String>();
        format!("{preview}...")
    } else if single_line.is_empty() {
        "空文本".to_string()
    } else {
        single_line
    }
}

fn unix_ms() -> i64 {
    let duration = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    duration.as_millis() as i64
}

fn hash_string(value: &str) -> String {
    hash_bytes(value.as_bytes())
}

fn hash_bytes(bytes: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}
