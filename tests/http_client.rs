//! Integration tests for `SimpleHttpClient`.
//!
//! Gated behind the `http-client` feature (enabled by default in the crate).
//! Uses `mockito` to spin up a local HTTP server so no network access is needed.

#![cfg(feature = "http-client")]

use rustpress_sdk::SimpleHttpClient;
use serde_json::json;
use std::collections::HashMap;

#[tokio::test]
async fn get_returns_parsed_json() {
    let mut server = mockito::Server::new_async().await;
    let mock = server
        .mock("GET", "/users/1")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"id":1,"name":"Alice"}"#)
        .create_async()
        .await;

    let client = SimpleHttpClient::new(server.url());
    let body = client.get("/users/1").await.expect("ok");
    assert_eq!(body, json!({"id": 1, "name": "Alice"}));
    mock.assert_async().await;
}

#[tokio::test]
async fn post_sends_json_body_and_returns_response() {
    let mut server = mockito::Server::new_async().await;
    let mock = server
        .mock("POST", "/users")
        .match_header("content-type", "application/json")
        .match_body(r#"{"name":"Bob"}"#)
        .with_status(201)
        .with_header("content-type", "application/json")
        .with_body(r#"{"id":2,"name":"Bob"}"#)
        .create_async()
        .await;

    let client = SimpleHttpClient::new(server.url());
    let response = client
        .post("/users", json!({"name": "Bob"}))
        .await
        .expect("ok");
    assert_eq!(response["id"], json!(2));
    assert_eq!(response["name"], json!("Bob"));
    mock.assert_async().await;
}

#[tokio::test]
async fn put_sends_json_body() {
    let mut server = mockito::Server::new_async().await;
    let mock = server
        .mock("PUT", "/users/1")
        .match_body(r#"{"name":"Carol"}"#)
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"id":1,"name":"Carol"}"#)
        .create_async()
        .await;

    let client = SimpleHttpClient::new(server.url());
    let response = client
        .put("/users/1", json!({"name": "Carol"}))
        .await
        .expect("ok");
    assert_eq!(response["name"], json!("Carol"));
    mock.assert_async().await;
}

#[tokio::test]
async fn delete_returns_response_body() {
    let mut server = mockito::Server::new_async().await;
    let mock = server
        .mock("DELETE", "/users/1")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"deleted":true}"#)
        .create_async()
        .await;

    let client = SimpleHttpClient::new(server.url());
    let response = client.delete("/users/1").await.expect("ok");
    assert_eq!(response, json!({"deleted": true}));
    mock.assert_async().await;
}

#[tokio::test]
async fn with_headers_sends_custom_headers() {
    let mut server = mockito::Server::new_async().await;
    let mock = server
        .mock("GET", "/me")
        .match_header("authorization", "Bearer test-token")
        .match_header("x-trace-id", "trace-123")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"ok":true}"#)
        .create_async()
        .await;

    let mut headers = HashMap::new();
    headers.insert("Authorization".to_string(), "Bearer test-token".to_string());
    headers.insert("X-Trace-Id".to_string(), "trace-123".to_string());

    let client = SimpleHttpClient::new(server.url()).with_headers(headers);
    let response = client.get("/me").await.expect("ok");
    assert_eq!(response, json!({"ok": true}));
    mock.assert_async().await;
}

#[tokio::test]
async fn get_returns_err_for_invalid_json_body() {
    let mut server = mockito::Server::new_async().await;
    let _mock = server
        .mock("GET", "/broken")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body("not valid json at all")
        .create_async()
        .await;

    let client = SimpleHttpClient::new(server.url());
    let result = client.get("/broken").await;
    assert!(result.is_err(), "expected JSON parse error");
}

#[tokio::test]
async fn get_returns_err_when_unable_to_connect() {
    // Address chosen so the connection is refused — no server listening on this port.
    let client = SimpleHttpClient::new("http://127.0.0.1:1/");
    let result = client.get("/anything").await;
    assert!(result.is_err());
}

#[tokio::test]
async fn base_url_is_prepended_to_path() {
    let mut server = mockito::Server::new_async().await;
    let mock = server
        .mock("GET", "/api/v1/ping")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"pong":true}"#)
        .create_async()
        .await;

    // base_url ends without trailing slash; path begins with /api/v1/ping
    let client = SimpleHttpClient::new(server.url());
    let response = client.get("/api/v1/ping").await.expect("ok");
    assert_eq!(response, json!({"pong": true}));
    mock.assert_async().await;
}

#[tokio::test]
async fn post_with_empty_body_object() {
    let mut server = mockito::Server::new_async().await;
    let mock = server
        .mock("POST", "/empty")
        .match_body("{}")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"received":true}"#)
        .create_async()
        .await;

    let client = SimpleHttpClient::new(server.url());
    let response = client.post("/empty", json!({})).await.expect("ok");
    assert_eq!(response, json!({"received": true}));
    mock.assert_async().await;
}
