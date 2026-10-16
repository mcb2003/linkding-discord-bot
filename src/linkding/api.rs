use anyhow::Context;
use url::Url;

use crate::linkding::CreateBookmarkRequest;

use super::{Bookmark, CheckResponse, Error};

#[derive(Debug)]
pub struct LinkdingApi {
    client: reqwest::Client,
    base_url: reqwest::Url,
}

/// Basic info and construction.
impl LinkdingApi {
    /// The user agent send with each request to Linkding
    pub const USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));

    /// Create a new Linkding client from a base URL and API token.
    pub fn new(base_url: Url, token: String) -> Self {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            reqwest::header::AUTHORIZATION,
            format!("Token {}", token)
                .parse()
                .expect("Invalid token format"),
        );
        let client = reqwest::Client::builder()
            .user_agent(Self::USER_AGENT)
            .default_headers(headers)
            .tls_built_in_webpki_certs(false)
            .tls_built_in_native_certs(true)
            .build()
            .expect("Failed to build HTTP client");

        LinkdingApi { client, base_url }
    }

    /// Create a new Linkding client from environment variables.
    pub fn from_env() -> anyhow::Result<Self> {
        let base_url = std::env::var("LINKDING_BASE_URL")
            .context("Could not retrieve Linkding base URL from environment")?;
        let base_url: Url = base_url
            .parse()
            .context("Invalid Linkding base URL format")?;
        let token = std::env::var("LINKDING_API_TOKEN")
            .context("Could not retrieve Linkding API token from environment")?;

        Ok(LinkdingApi::new(base_url, token))
    }
}

/// General operations.
impl LinkdingApi {
    /// Get the base URL of the Linkding instance.
    pub fn base_url(&self) -> &Url {
        &self.base_url
    }

    /// Determine if the Linkding instance is reachable.
    pub async fn is_reachable(&self) -> super::Result<()> {
        let url = self.base_url.join("api/")?;
        let response = self.client.get(url).send().await?;
        let response = Error::for_api_error(response).await?;
        let body = response
            .json::<serde_json::Value>()
            .await
            .map_err(|_| Error::InvalidResponseFormat)?;

        match body {
            serde_json::Value::Object(map) if map.is_empty() => Ok(()),
            _ => Err(Error::InvalidResponseFormat),
        }
    }
}

/// Bookmark operations.
impl LinkdingApi {
    /// Get bookmark by ID.
    pub async fn bookmark(&self, id: u64) -> super::Result<Bookmark> {
        let url = self.base_url.join(&format!("api/bookmarks/{}", id))?;
        let response = self.client.get(url).send().await?;
        let response = Error::for_api_error(response).await?;
        response
            .json::<Bookmark>()
            .await
            .map_err(|_| Error::InvalidResponseFormat)
    }

    /// check if a URL is already bookmarked.
    ///
    /// If the URL is already bookmarked, the bookmark property in the [response][CheckResponse] holds the bookmark data, otherwise it is [`None`]`].
    pub async fn check_bookmark(&self, url: Url) -> super::Result<CheckResponse> {
        let mut req_url = self.base_url.join("api/bookmarks/check/")?;
        req_url
            .query_pairs_mut()
            .append_pair("url", url.as_str())
            .finish();

        let response = self.client.get(req_url).send().await?;
        let response = Error::for_api_error(response).await?;
        response
            .json::<CheckResponse>()
            .await
            .map_err(|_| Error::InvalidResponseFormat)
    }

    /// Create a new bookmark.
    pub async fn create_bookmark(&self, req: CreateBookmarkRequest) -> super::Result<Bookmark> {
        let url = self.base_url.join("api/bookmarks/")?;

        let response = self.client.post(url).json(&req).send().await?;
        let response = Error::for_api_error(response).await?;
        response
            .json::<Bookmark>()
            .await
            .map_err(|_| Error::InvalidResponseFormat)
    }
}
