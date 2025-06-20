use anyhow::Context;
use reqwest::Url;

#[derive(Debug)]
pub struct LinkdingApi {
    client: reqwest::Client,
    base_url: reqwest::Url,
    token: String,
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
            .build()
            .expect("Failed to build HTTP client");

        LinkdingApi {
            client,
            base_url,
            token,
        }
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
