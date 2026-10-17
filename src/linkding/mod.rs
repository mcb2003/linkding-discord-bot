mod api;
pub use api::LinkdingApi;
mod bookmark;
pub use bookmark::{Bookmark, CreateBookmarkRequest};
mod check_response;
pub use check_response::CheckResponse;
mod error;
pub use error::{Error, Result};

pub type DateTime = chrono::DateTime<chrono::Utc>;

#[cfg(test)]
mod tests;
