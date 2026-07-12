use std::sync::Arc;

use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use serde::Serialize;

use crate::sync_state::SyncState;

// ---------------------------------------------------------------------------
// Response types
// ---------------------------------------------------------------------------

#[derive(Serialize)]
pub struct HealthResponse {
    /// `"ok"` when the sync worker is current; `"degraded"` when it has not
    /// produced a successful poll within the staleness threshold.
    status: &'static str,
    sync: SyncStatus,
}

#[derive(Serialize)]
pub struct SyncStatus {
    last_processed_ledger: u64,
    /// Seconds elapsed since the last successful poll cycle.
    /// `null` in JSON if the worker has never completed a cycle since startup.
    seconds_since_last_poll: Option<u64>,
}

// ---------------------------------------------------------------------------
// Handler
// ---------------------------------------------------------------------------

/// GET /health
///
/// Returns:
/// - `200 OK`   with `{"status":"ok",     "sync":{...}}` when sync is healthy.
/// - `503`      with `{"status":"degraded","sync":{...}}` when no successful
///              poll has happened within the staleness threshold (60 s), or
///              when the worker has never completed a cycle since startup.
///
/// The `sync` object is additive — existing consumers that only check
/// `status == "ok"` continue to work without changes.
pub async fn handler(State(sync_state): State<Arc<SyncState>>) -> impl IntoResponse {
    let snap = sync_state.snapshot();

    let seconds_since_last_poll = if snap.seconds_since_last_poll == u64::MAX {
        // Never polled — surface as null rather than a nonsensical huge number.
        None
    } else {
        Some(snap.seconds_since_last_poll)
    };

    let sync = SyncStatus {
        last_processed_ledger: snap.last_processed_ledger,
        seconds_since_last_poll,
    };

    if snap.is_stale() {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(HealthResponse {
                status: "degraded",
                sync,
            }),
        )
    } else {
        (
            StatusCode::OK,
            Json(HealthResponse {
                status: "ok",
                sync,
            }),
        )
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use std::sync::atomic::Ordering;
    use std::sync::Arc;
    use std::time::{SystemTime, UNIX_EPOCH};

    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    use crate::router_for_test;
    use crate::sync_state::{SyncState, STALE_THRESHOLD_SECS};

    fn now_unix_secs() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    }

    async fn get_health(sync_state: Arc<SyncState>) -> (StatusCode, serde_json::Value) {
        let app = router_for_test(sync_state);
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        let status = response.status();
        let body = response.into_body().collect().await.unwrap().to_bytes();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        (status, json)
    }

    // -----------------------------------------------------------------------
    // Degraded: worker never polled
    // -----------------------------------------------------------------------

    #[tokio::test]
    async fn health_degraded_when_never_polled() {
        let state = SyncState::new_shared();
        let (status, json) = get_health(state).await;

        assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
        assert_eq!(json["status"], "degraded");
        assert_eq!(json["sync"]["last_processed_ledger"], 0);
        assert!(
            json["sync"]["seconds_since_last_poll"].is_null(),
            "expected null for never-polled, got: {}",
            json["sync"]["seconds_since_last_poll"]
        );
    }

    // -----------------------------------------------------------------------
    // Healthy: recent poll
    // -----------------------------------------------------------------------

    #[tokio::test]
    async fn health_ok_after_recent_poll() {
        let state = SyncState::new_shared();
        state.record_poll(500);

        let (status, json) = get_health(state).await;

        assert_eq!(status, StatusCode::OK);
        assert_eq!(json["status"], "ok");
        assert_eq!(json["sync"]["last_processed_ledger"], 500);

        let secs = json["sync"]["seconds_since_last_poll"]
            .as_u64()
            .expect("should be a number");
        assert!(secs <= 1, "should be fresh, got {secs}s");
    }

    // -----------------------------------------------------------------------
    // Degraded: stale worker
    // -----------------------------------------------------------------------

    #[tokio::test]
    async fn health_degraded_when_poll_is_stale() {
        let state = SyncState::new_shared();
        // Backdate the last poll to beyond the threshold.
        let stale_ts = now_unix_secs() - STALE_THRESHOLD_SECS - 5;
        state
            .last_poll_unix_secs
            .store(stale_ts, Ordering::Relaxed);
        state
            .last_processed_ledger
            .store(999, Ordering::Relaxed);

        let (status, json) = get_health(state).await;

        assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
        assert_eq!(json["status"], "degraded");
        assert_eq!(json["sync"]["last_processed_ledger"], 999);

        let secs = json["sync"]["seconds_since_last_poll"]
            .as_u64()
            .expect("should be a number");
        assert!(
            secs > STALE_THRESHOLD_SECS,
            "expected > {STALE_THRESHOLD_SECS}s, got {secs}s"
        );
    }

    // -----------------------------------------------------------------------
    // Content-type
    // -----------------------------------------------------------------------

    #[tokio::test]
    async fn health_content_type_is_json() {
        let state = SyncState::new_shared();
        state.record_poll(1);

        let app = router_for_test(state);
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        let content_type = response
            .headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");

        assert!(
            content_type.contains("application/json"),
            "expected application/json, got {content_type}"
        );
    }

    // -----------------------------------------------------------------------
    // Response shape is additive (sync field present in both ok and degraded)
    // -----------------------------------------------------------------------

    #[tokio::test]
    async fn health_response_always_contains_sync_object() {
        for polled in [false, true] {
            let state = SyncState::new_shared();
            if polled {
                state.record_poll(1);
            }
            let (_, json) = get_health(state).await;
            assert!(
                json.get("sync").is_some(),
                "sync key missing from response (polled={polled}): {json}"
            );
            assert!(
                json["sync"].get("last_processed_ledger").is_some(),
                "last_processed_ledger missing (polled={polled})"
            );
            assert!(
                json["sync"].get("seconds_since_last_poll").is_some(),
                "seconds_since_last_poll key missing (polled={polled})"
            );
        }
    }
}
