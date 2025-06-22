use super::DateTime;

#[derive(Debug, Clone, serde::Deserialize)]
// Fixme: Remove warning when splitting into linkding_api crate
#[allow(dead_code)]
pub struct Bookmark {
    pub id: u64,
    pub url: String,
    pub title: String,
    pub description: String,
    pub notes: String,
    pub web_archive_snapshot_url: Option<String>,
    pub favicon_url: Option<String>,
    pub preview_image_url: Option<String>,
    pub is_archived: bool,
    pub unread: bool,
    pub shared: bool,
    pub tag_names: Vec<String>,
    pub date_added: DateTime,
    pub date_modified: DateTime,
    pub website_title: Option<String>,
    pub website_description: Option<String>,
}

#[derive(Debug, Default, Clone, serde::Serialize)]
pub struct CreateBookmarkRequest {
    pub url: String,
    pub title: Option<String>,
    pub description: Option<String>,
    pub notes: String,
    pub is_archived: bool,
    pub unread: bool,
    pub shared: bool,
    pub tag_names: Vec<String>,
}
