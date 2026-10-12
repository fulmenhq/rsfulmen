//! Integration coverage for `POST /admin/signal`.
//!
//! The HTTP adapter needs `serde_json` (`json` feature). `foundry-core` alone
//! compiles the request/response types and `handle_admin_signal_request`.

#![cfg(all(feature = "json", feature = "foundry-core"))]

use rsfulmen::signals::{SignalEndpointMetadata, SignalManager};

#[test]
fn admin_signal_http_accepts_hup() {
    let manager = SignalManager::new();
    let (status, body) = manager.handle_admin_signal_http(
        "POST",
        "/admin/signal",
        br#"{"signal":"HUP","reason":"reload"}"#,
        SignalEndpointMetadata::default(),
    );
    assert_eq!(status, 202, "body={body}");
    assert!(body.contains("\"status\":\"accepted\""));
    assert!(body.contains("\"signal\":\"HUP\""));
}

#[test]
fn admin_signal_http_rejects_unknown_signal() {
    let manager = SignalManager::new();
    let (status, body) = manager.handle_admin_signal_http(
        "POST",
        "/admin/signal",
        br#"{"signal":"KILL"}"#,
        SignalEndpointMetadata::default(),
    );
    assert_eq!(status, 400, "body={body}");
    assert!(body.contains("invalid_signal"));
}

#[test]
fn admin_signal_http_rejects_wrong_method_and_path() {
    let manager = SignalManager::new();
    let (status, _) = manager.handle_admin_signal_http(
        "GET",
        "/admin/signal",
        b"{}",
        SignalEndpointMetadata::default(),
    );
    assert_eq!(status, 405);
    let (status, _) = manager.handle_admin_signal_http(
        "POST",
        "/admin/other",
        b"{}",
        SignalEndpointMetadata::default(),
    );
    assert_eq!(status, 404);
}

#[test]
fn admin_signal_http_rejects_invalid_json() {
    let manager = SignalManager::new();
    let (status, body) = manager.handle_admin_signal_http(
        "POST",
        "/admin/signal",
        b"not-json",
        SignalEndpointMetadata::default(),
    );
    assert_eq!(status, 400);
    assert!(body.contains("invalid_json"));
}
