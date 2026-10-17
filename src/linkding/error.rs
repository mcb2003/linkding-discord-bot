use reqwest::Response;

#[derive(Clone, Debug, serde::Deserialize)]
pub struct ErrorResponse {
    pub(self) detail: String,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Invalid URL: {0}")]
    InvalidUrl(#[from] url::ParseError),
    #[error("Request failed: {0}")]
    RequestFailed(#[from] reqwest::Error),
    #[error("Invalid response format")]
    InvalidResponseFormat,
    #[error("{}", .response.detail)]
    Response {
        status: reqwest::StatusCode,
        response: ErrorResponse,
    },
}

impl Error {
    /// Converts a `Response` into an `Error` if the response status is not successful.
    ///
    /// If the response status is successful, it returns the original response.
    pub(super) async fn for_api_error(response: Response) -> Result<Response> {
        let status = response.status();
        if status.is_success() {
            Ok(response)
        } else {
            let body = response.bytes().await?;
            Err(serde_json::from_slice(&body)
                .map(|response| Error::Response { status, response })
                .unwrap_or_else(|_| Error::Response {
                    status,
                    response: ErrorResponse {
                        detail: status
                            .canonical_reason()
                            .unwrap_or("Unknown HTTP error")
                            .to_owned(),
                    },
                }))
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;
