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

#[test]
fn admin_signal_http_rejects_missing_signal_field() {
    let manager = SignalManager::new();
    let (status, body) = manager.handle_admin_signal_http(
        "POST",
        "/admin/signal",
        b"{}",
        SignalEndpointMetadata::default(),
    );
    assert_eq!(status, 400, "body={body}");
    assert!(body.contains("invalid_json"));
}

#[test]
fn admin_signal_http_accepts_sig_prefix_and_query_path() {
    let manager = SignalManager::new();
    let (status, body) = manager.handle_admin_signal_http(
        "post",
        "/admin/signal?source=ops",
        br#"{"signal":"SIGHUP"}"#,
        SignalEndpointMetadata::default(),
    );
    assert_eq!(status, 202, "body={body}");
    assert!(body.contains("\"signal\":\"HUP\""));
}

#[test]
fn admin_signal_http_put_is_method_not_allowed() {
    let manager = SignalManager::new();
    let (status, body) = manager.handle_admin_signal_http(
        "PUT",
        "/admin/signal",
        br#"{"signal":"HUP"}"#,
        SignalEndpointMetadata::default(),
    );
    assert_eq!(status, 405);
    assert!(body.contains("method_not_allowed"));
}

#[test]
fn admin_signal_http_accepts_without_auth_identity() {
    let manager = SignalManager::new();
    let (status, _) = manager.handle_admin_signal_http(
        "POST",
        "/admin/signal",
        br#"{"signal":"USR1"}"#,
        SignalEndpointMetadata::default(),
    );
    assert_eq!(
        status, 202,
        "adapter has no auth; missing identity is not 401"
    );
}

#[test]
fn admin_signal_http_concurrent_posts() {
    use std::sync::Arc;
    use std::thread;

    let manager = Arc::new(SignalManager::new());
    let mut joins = Vec::new();
    for _ in 0..8 {
        let manager = Arc::clone(&manager);
        joins.push(thread::spawn(move || {
            manager.handle_admin_signal_http(
                "POST",
                "/admin/signal",
                br#"{"signal":"USR2"}"#,
                SignalEndpointMetadata::default(),
            )
        }));
    }

    let mut accepted = 0;
    for join in joins {
        let (status, body) = join.join().expect("worker");
        assert_eq!(status, 202, "body={body}");
        accepted += 1;
    }
    assert_eq!(accepted, 8);
}

#[test]
fn admin_signal_http_rejects_blank_signal() {
    let manager = SignalManager::new();
    let (status, body) = manager.handle_admin_signal_http(
        "POST",
        "/admin/signal",
        br#"{"signal":"   "}"#,
        SignalEndpointMetadata::default(),
    );
    assert_eq!(status, 400, "body={body}");
    assert!(body.contains("invalid_signal"), "body={body}");
}

#[test]
fn admin_signal_http_rejects_trailing_slash_path() {
    let manager = SignalManager::new();
    let (status, body) = manager.handle_admin_signal_http(
        "POST",
        "/admin/signal/",
        br#"{"signal":"HUP"}"#,
        SignalEndpointMetadata::default(),
    );
    assert_eq!(status, 404, "body={body}");
}

#[test]
fn admin_signal_http_hup_is_injected_and_reloads() {
    use rsfulmen::signals::testing::SignalInjector;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;
    use std::thread;
    use std::time::{Duration, Instant};

    let manager = SignalManager::new();
    let hits = Arc::new(AtomicUsize::new(0));
    let hits_c = Arc::clone(&hits);
    manager.on_reload(move || {
        hits_c.fetch_add(1, Ordering::SeqCst);
        Ok(())
    });

    let injector = SignalInjector::new(&manager);
    let manager_listen = manager.clone();
    let listener = thread::spawn(move || manager_listen.listen());

    injector
        .wait_for_listen(Duration::from_secs(1))
        .expect("listener should start");

    let (status, body) = manager.handle_admin_signal_http(
        "POST",
        "/admin/signal",
        br#"{"signal":"HUP"}"#,
        SignalEndpointMetadata::default(),
    );
    assert_eq!(status, 202, "body={body}");

    let deadline = Instant::now() + Duration::from_secs(1);
    while hits.load(Ordering::SeqCst) == 0 && Instant::now() < deadline {
        thread::sleep(Duration::from_millis(10));
    }

    manager.stop();
    listener
        .join()
        .expect("listener thread should join")
        .expect("listener should exit cleanly");

    assert_eq!(
        hits.load(Ordering::SeqCst),
        1,
        "HTTP adapter must inject into the listen loop"
    );
}
