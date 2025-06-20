mod api;
pub use api::LinkdingApi;
mod bookmark;
use bookmark::Bookmark;
mod error;
pub use error::{Error, Result};

pub type DateTime = chrono::DateTime<chrono::Utc>;
