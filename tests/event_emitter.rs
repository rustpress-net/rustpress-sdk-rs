//! Integration tests for `SimpleEventEmitter`.
//!
//! Covers subscribe + emit, multiple handlers for the same event,
//! independence of unrelated events, and emit-without-subscribers no-op.

use rustpress_sdk::SimpleEventEmitter;
use serde_json::json;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use tokio::sync::Mutex;

#[tokio::test]
async fn emitter_new_and_default_are_equivalent() {
    let _e1 = SimpleEventEmitter::new();
    let _e2 = SimpleEventEmitter::default();
}

#[tokio::test]
async fn emit_invokes_registered_handler() {
    let emitter = SimpleEventEmitter::new();
    let count = Arc::new(AtomicU32::new(0));
    let c = count.clone();

    emitter
        .on("test", move |_data| {
            let c = c.clone();
            async move {
                c.fetch_add(1, Ordering::SeqCst);
            }
        })
        .await;

    emitter.emit("test", json!({"x": 1})).await;
    assert_eq!(count.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn emit_passes_data_to_handler() {
    let emitter = SimpleEventEmitter::new();
    let captured: Arc<Mutex<Option<serde_json::Value>>> = Arc::new(Mutex::new(None));
    let c = captured.clone();

    emitter
        .on("data", move |data| {
            let c = c.clone();
            async move {
                *c.lock().await = Some(data);
            }
        })
        .await;

    emitter.emit("data", json!({"name": "rustpress"})).await;
    let value = captured.lock().await;
    assert_eq!(value.as_ref().unwrap(), &json!({"name": "rustpress"}));
}

#[tokio::test]
async fn multiple_handlers_for_same_event_all_fire() {
    let emitter = SimpleEventEmitter::new();
    let count = Arc::new(AtomicU32::new(0));

    for _ in 0..3 {
        let c = count.clone();
        emitter
            .on("multi", move |_data| {
                let c = c.clone();
                async move {
                    c.fetch_add(1, Ordering::SeqCst);
                }
            })
            .await;
    }

    emitter.emit("multi", json!(null)).await;
    assert_eq!(count.load(Ordering::SeqCst), 3);
}

#[tokio::test]
async fn handlers_for_different_events_are_independent() {
    let emitter = SimpleEventEmitter::new();
    let a_count = Arc::new(AtomicU32::new(0));
    let b_count = Arc::new(AtomicU32::new(0));

    let ac = a_count.clone();
    emitter
        .on("a", move |_data| {
            let ac = ac.clone();
            async move {
                ac.fetch_add(1, Ordering::SeqCst);
            }
        })
        .await;

    let bc = b_count.clone();
    emitter
        .on("b", move |_data| {
            let bc = bc.clone();
            async move {
                bc.fetch_add(1, Ordering::SeqCst);
            }
        })
        .await;

    emitter.emit("a", json!(1)).await;
    emitter.emit("a", json!(2)).await;
    emitter.emit("b", json!(3)).await;

    assert_eq!(a_count.load(Ordering::SeqCst), 2);
    assert_eq!(b_count.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn emit_with_no_subscribers_is_noop() {
    let emitter = SimpleEventEmitter::new();
    // No handlers registered; emit must not panic.
    emitter.emit("nobody-listening", json!({"x": 1})).await;
}

#[tokio::test]
async fn emit_multiple_times_invokes_handler_each_time() {
    let emitter = SimpleEventEmitter::new();
    let count = Arc::new(AtomicU32::new(0));
    let c = count.clone();

    emitter
        .on("tick", move |_data| {
            let c = c.clone();
            async move {
                c.fetch_add(1, Ordering::SeqCst);
            }
        })
        .await;

    for _ in 0..5 {
        emitter.emit("tick", json!(null)).await;
    }
    assert_eq!(count.load(Ordering::SeqCst), 5);
}

#[tokio::test]
async fn handler_can_use_captured_state() {
    let emitter = SimpleEventEmitter::new();
    let log: Arc<Mutex<Vec<i64>>> = Arc::new(Mutex::new(Vec::new()));
    let l = log.clone();

    emitter
        .on("push", move |data| {
            let l = l.clone();
            async move {
                if let Some(n) = data.as_i64() {
                    l.lock().await.push(n);
                }
            }
        })
        .await;

    emitter.emit("push", json!(1)).await;
    emitter.emit("push", json!(2)).await;
    emitter.emit("push", json!(3)).await;

    let log = log.lock().await;
    assert_eq!(*log, vec![1, 2, 3]);
}

#[tokio::test]
async fn emitter_shared_across_tasks() {
    let emitter = Arc::new(SimpleEventEmitter::new());
    let count = Arc::new(AtomicU32::new(0));
    let c = count.clone();

    emitter
        .on("shared", move |_data| {
            let c = c.clone();
            async move {
                c.fetch_add(1, Ordering::SeqCst);
            }
        })
        .await;

    let mut handles = Vec::new();
    for i in 0..10 {
        let e = emitter.clone();
        handles.push(tokio::spawn(async move {
            e.emit("shared", json!(i)).await;
        }));
    }
    for h in handles {
        h.await.unwrap();
    }
    assert_eq!(count.load(Ordering::SeqCst), 10);
}
