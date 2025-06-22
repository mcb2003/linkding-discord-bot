use url::Url;

use super::Bookmark;

/// A response from the /bookmarks/check/ endpoint.
#[derive(Debug, serde::Deserialize)]
pub struct CheckResponse {
    bookmark: Option<Bookmark>,
    metadata: PageMetadata,
    auto_tags: Vec<String>,
}

impl CheckResponse {
    #[inline]
    pub fn bookmark(&self) -> Option<&Bookmark> {
        self.bookmark.as_ref()
    }

    #[inline]
    pub fn scraped_url(&self) -> &Url {
        // Not sure if this is always the same as the checked URL, or if it modifies it for redirects etc.
        &self.metadata.url
    }

    #[inline]
    pub fn scraped_title(&self) -> Option<&str> {
        self.metadata.title.as_deref()
    }

    #[inline]
    pub fn scraped_description(&self) -> Option<&str> {
        self.metadata.description.as_deref()
    }

    #[inline]
    pub fn auto_tags(&self) -> &[String] {
        &self.auto_tags
    }
}

/// Scraped metadata from the page at a URL.
#[derive(Debug, serde::Deserialize)]
struct PageMetadata {
    url: Url,
    title: Option<String>,
    description: Option<String>,
}
