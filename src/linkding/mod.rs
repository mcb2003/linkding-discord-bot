mod api;
pub use api::LinkdingApi;
mod bookmark;

pub type DateTime = chrono::DateTime<chrono::Utc>;
