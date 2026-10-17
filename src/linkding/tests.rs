use reqwest::StatusCode;
use serde_json::{Value, json};
use wiremock::{
    Mock, MockServer, ResponseTemplate,
    matchers::{body_json, header, method, path, query_param},
};

use super::{CreateBookmarkRequest, Error, LinkdingApi};

const TOKEN: &str = "test-token";

fn api(server: &MockServer) -> LinkdingApi {
    LinkdingApi::new(
        server
            .uri()
            .parse()
            .expect("mock server URI should be valid"),
        TOKEN.into(),
    )
}

fn bookmark_json() -> Value {
    json!({
        "id": 42,
        "url": "https://example.org/article",
        "title": "Example Article",
        "description": "A description",
        "notes": "Read this before Friday",
        "web_archive_snapshot_url": "https://web.archive.org/example",
        "favicon_url": "https://example.org/favicon.ico",
        "preview_image_url": "https://example.org/preview.png",
        "is_archived": false,
        "unread": true,
        "shared": true,
        "tag_names": ["example", "article"],
        "date_added": "2026-10-17T12:34:56Z",
        "date_modified": "2026-10-17T13:00:00Z",
        "website_title": "Example",
        "website_description": "Example website",
    })
}

#[tokio::test]
async fn reachability_request_uses_authentication_and_accepts_empty_object() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/"))
        .and(header("authorization", format!("Token {TOKEN}")))
        .and(header("user-agent", LinkdingApi::USER_AGENT))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({})))
        .expect(1)
        .mount(&server)
        .await;

    api(&server)
        .is_reachable()
        .await
        .expect("empty API root response should indicate reachability");
}

#[tokio::test]
async fn bookmark_request_uses_documented_endpoint_and_decodes_response() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/bookmarks/42/"))
        .and(header("authorization", format!("Token {TOKEN}")))
        .respond_with(ResponseTemplate::new(200).set_body_json(bookmark_json()))
        .mount(&server)
        .await;

    let bookmark = api(&server)
        .bookmark(42)
        .await
        .expect("bookmark response should be decoded");

    assert_eq!(bookmark.id, 42);
    assert_eq!(bookmark.url, "https://example.org/article");
    assert_eq!(bookmark.title, "Example Article");
    assert_eq!(bookmark.tag_names, ["example", "article"]);
}

#[tokio::test]
async fn check_request_encodes_target_url_and_decodes_scraped_metadata() {
    let server = MockServer::start().await;
    let target = "https://example.org/article?language=rust&sort=newest";
    Mock::given(method("GET"))
        .and(path("/api/bookmarks/check/"))
        .and(query_param("url", target))
        .and(header("authorization", format!("Token {TOKEN}")))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "bookmark": null,
            "metadata": {
                "url": target,
                "title": "Example Article",
                "description": "A description"
            },
            "auto_tags": ["example", "article"]
        })))
        .expect(1)
        .mount(&server)
        .await;

    let response = api(&server)
        .check_bookmark(target.parse().unwrap())
        .await
        .expect("check response should be decoded");

    assert!(response.bookmark().is_none());
    assert_eq!(response.scraped_url().as_str(), target);
    assert_eq!(response.scraped_title(), Some("Example Article"));
    assert_eq!(response.scraped_description(), Some("A description"));
    assert_eq!(response.auto_tags(), ["example", "article"]);
}

#[tokio::test]
async fn create_request_sends_expected_json_and_decodes_bookmark() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/bookmarks/"))
        .and(header("authorization", format!("Token {TOKEN}")))
        .and(body_json(json!({
            "url": "https://example.org/article",
            "title": "Example Article",
            "description": "A description",
            "notes": "Read this before Friday",
            "is_archived": false,
            "unread": true,
            "shared": true,
            "tag_names": ["example", "article"]
        })))
        .respond_with(ResponseTemplate::new(201).set_body_json(bookmark_json()))
        .expect(1)
        .mount(&server)
        .await;

    let request = CreateBookmarkRequest {
        url: "https://example.org/article".into(),
        title: Some("Example Article".into()),
        description: Some("A description".into()),
        notes: "Read this before Friday".into(),
        is_archived: false,
        unread: true,
        shared: true,
        tag_names: vec!["example".into(), "article".into()],
    };

    let bookmark = api(&server)
        .create_bookmark(request)
        .await
        .expect("created bookmark response should be decoded");

    assert_eq!(bookmark.id, 42);
    assert_eq!(bookmark.title, "Example Article");
}

#[tokio::test]
async fn structured_api_error_preserves_status_and_message() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/"))
        .respond_with(ResponseTemplate::new(403).set_body_json(json!({"detail": "Invalid token."})))
        .expect(1)
        .mount(&server)
        .await;

    let error = api(&server)
        .is_reachable()
        .await
        .expect_err("error response should not be accepted");

    assert_eq!(error.to_string(), "Invalid token.");
    match error {
        Error::Response { status, .. } => assert_eq!(status, StatusCode::FORBIDDEN),
        other => panic!("expected a structured API error, got {other:?}"),
    }
}

#[tokio::test]
async fn malformed_successful_response_is_reported_as_invalid_format() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/"))
        .respond_with(ResponseTemplate::new(200).set_body_string("not JSON"))
        .expect(1)
        .mount(&server)
        .await;

    let error = api(&server)
        .is_reachable()
        .await
        .expect_err("malformed response should not be accepted");

    assert!(matches!(error, Error::InvalidResponseFormat));
}

#[tokio::test]
async fn non_json_api_error_preserves_http_status() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/"))
        .respond_with(
            ResponseTemplate::new(StatusCode::BAD_GATEWAY.as_u16())
                .set_body_string("<html>Bad gateway</html>"),
        )
        .expect(1)
        .mount(&server)
        .await;

    let error = api(&server)
        .is_reachable()
        .await
        .expect_err("server error should not be accepted");

    match error {
        Error::Response { status, .. } => assert_eq!(status, StatusCode::BAD_GATEWAY),
        other => panic!("expected an API error retaining HTTP status, got {other:?}"),
    }
}
