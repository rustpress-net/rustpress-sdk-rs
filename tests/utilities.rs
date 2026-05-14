//! Integration tests for SDK utility functions.
//!
//! Covers `generate_id` (uniqueness, prefix), `retry` (success/failure paths
//! including success-after-N), and `sleep_ms` smoke test.

use rustpress_sdk::{generate_id, retry, sleep_ms, RetryOptions};
use std::collections::HashSet;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use std::time::Instant;

#[test]
fn generate_id_no_prefix_is_nonempty() {
    let id = generate_id(None);
    assert!(!id.is_empty());
    assert!(!id.contains('_'), "no prefix means no underscore separator");
}

#[test]
fn generate_id_with_prefix_starts_with_prefix() {
    let id = generate_id(Some("order"));
    assert!(id.starts_with("order_"));
    assert!(id.len() > "order_".len());
}

#[test]
fn generate_id_unique_across_many_calls() {
    let mut set = HashSet::new();
    for _ in 0..1000 {
        let id = generate_id(None);
        assert!(set.insert(id), "ID collision in 1000 calls");
    }
}

#[test]
fn generate_id_unique_with_same_prefix() {
    let mut set = HashSet::new();
    for _ in 0..500 {
        let id = generate_id(Some("test"));
        assert!(id.starts_with("test_"));
        assert!(set.insert(id), "Collision with same prefix");
    }
}

#[test]
fn generate_id_format_contains_hex() {
    let id = generate_id(None);
    // Should consist of timestamp-hex + uuid-fragment, all lowercase hex with possible dashes.
    assert!(id.chars().all(|c| c.is_ascii_hexdigit() || c == '-'));
}

#[test]
fn retry_options_default_values() {
    let opts = RetryOptions::default();
    assert_eq!(opts.attempts, 3);
    assert_eq!(opts.delay_ms, 1000);
    assert_eq!(opts.max_delay_ms, 30000);
}

#[tokio::test]
async fn retry_returns_ok_immediately_on_first_success() {
    let calls = Arc::new(AtomicU32::new(0));
    let calls_c = calls.clone();
    let opts = RetryOptions {
        attempts: 3,
        delay_ms: 1,
        max_delay_ms: 5,
    };
    let result: Result<u32, &str> = retry(
        || {
            let calls_c = calls_c.clone();
            async move {
                calls_c.fetch_add(1, Ordering::SeqCst);
                Ok(42u32)
            }
        },
        opts,
    )
    .await;
    assert_eq!(result.unwrap(), 42);
    assert_eq!(calls.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn retry_succeeds_after_n_failures() {
    let calls = Arc::new(AtomicU32::new(0));
    let calls_c = calls.clone();
    let opts = RetryOptions {
        attempts: 5,
        delay_ms: 1,
        max_delay_ms: 5,
    };
    let result: Result<&'static str, &'static str> = retry(
        || {
            let calls_c = calls_c.clone();
            async move {
                let n = calls_c.fetch_add(1, Ordering::SeqCst) + 1;
                if n < 3 {
                    Err("not yet")
                } else {
                    Ok("ok")
                }
            }
        },
        opts,
    )
    .await;
    assert_eq!(result.unwrap(), "ok");
    assert_eq!(calls.load(Ordering::SeqCst), 3);
}

#[tokio::test]
async fn retry_returns_last_error_when_all_attempts_fail() {
    let calls = Arc::new(AtomicU32::new(0));
    let calls_c = calls.clone();
    let opts = RetryOptions {
        attempts: 4,
        delay_ms: 1,
        max_delay_ms: 5,
    };
    let result: Result<(), String> = retry(
        || {
            let calls_c = calls_c.clone();
            async move {
                let n = calls_c.fetch_add(1, Ordering::SeqCst) + 1;
                Err::<(), String>(format!("err {}", n))
            }
        },
        opts,
    )
    .await;
    assert_eq!(result.unwrap_err(), "err 4");
    assert_eq!(calls.load(Ordering::SeqCst), 4);
}

#[tokio::test]
async fn retry_with_single_attempt_does_not_sleep() {
    let calls = Arc::new(AtomicU32::new(0));
    let calls_c = calls.clone();
    let opts = RetryOptions {
        attempts: 1,
        delay_ms: 10_000, // would dominate if it slept
        max_delay_ms: 30_000,
    };
    let start = Instant::now();
    let result: Result<(), &str> = retry(
        || {
            let calls_c = calls_c.clone();
            async move {
                calls_c.fetch_add(1, Ordering::SeqCst);
                Err::<(), &str>("nope")
            }
        },
        opts,
    )
    .await;
    let elapsed = start.elapsed();
    assert!(result.is_err());
    assert_eq!(calls.load(Ordering::SeqCst), 1);
    // No sleep when there are no further attempts to wait for.
    assert!(
        elapsed.as_millis() < 500,
        "single-attempt retry should not delay (got {}ms)",
        elapsed.as_millis()
    );
}

#[tokio::test]
async fn sleep_ms_actually_sleeps() {
    let start = Instant::now();
    sleep_ms(20).await;
    let elapsed = start.elapsed();
    assert!(
        elapsed.as_millis() >= 15,
        "expected ~20ms sleep, got {}ms",
        elapsed.as_millis()
    );
}

#[tokio::test]
async fn sleep_ms_zero_returns_quickly() {
    let start = Instant::now();
    sleep_ms(0).await;
    assert!(start.elapsed().as_millis() < 100);
}

#[test]
fn retry_options_can_be_cloned() {
    let opts = RetryOptions {
        attempts: 7,
        delay_ms: 100,
        max_delay_ms: 1000,
    };
    let cloned = opts.clone();
    assert_eq!(cloned.attempts, 7);
    assert_eq!(cloned.delay_ms, 100);
    assert_eq!(cloned.max_delay_ms, 1000);
}
