use serde::Serialize;
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ClipboardItem {
    pub id: i64,
    pub item_type: ClipboardItemType,
    pub preview: String,
    pub is_pinned: bool,
    pub created_at: i64,
    pub updated_at: i64,
    pub image_url: Option<String>,
    pub image_size: Option<ImageSize>,
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ClipboardItemType {
    Text,
    Image,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImageSize {
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HistoryResponse {
    pub items: Vec<ClipboardItem>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppStatus {
    pub monitoring_paused: bool,
    pub history_count: i64,
    pub manage_hotkey: String,
    pub picker_hotkey: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ActionResponse {
    pub ok: bool,
    pub item: Option<ClipboardItem>,
    pub history_count: i64,
}

#[derive(Debug, Clone, FromRow)]
pub struct ClipboardRow {
    pub id: i64,
    pub item_type: String,
    pub content_text: Option<String>,
    pub image_path: Option<String>,
    #[allow(dead_code)]
    pub content_hash: String,
    pub image_width: Option<i64>,
    pub image_height: Option<i64>,
    pub is_pinned: i64,
    pub created_at: i64,
    pub updated_at: i64,
}

impl ClipboardRow {
    pub fn is_pinned(&self) -> bool {
        self.is_pinned != 0
    }

    pub fn item_type(&self) -> ClipboardItemType {
        match self.item_type.as_str() {
            "image" => ClipboardItemType::Image,
            _ => ClipboardItemType::Text,
        }
    }
}
