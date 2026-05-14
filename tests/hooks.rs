//! Integration tests for hook definitions.
//!
//! Covers `define_hook`, `before_hook`, `after_hook` builder semantics
//! plus `HookMetadata` round-trips and validation paths.

use chrono::Utc;
use rustpress_sdk::{
    after_hook, before_hook, define_hook, generate_id, ContentStatus, HookMetadata,
    NotificationChannel, NotificationType, RustPressContext, SdkError, TriggerArgs, TriggerTiming,
};

fn make_args() -> TriggerArgs<serde_json::Value, serde_json::Value> {
    TriggerArgs {
        original_args: serde_json::json!({"id": 1}),
        result: None,
        trigger: "@@rustpress.eco.Order.create@@".to_string(),
        timing: TriggerTiming::Before,
        timestamp: Utc::now(),
        trigger_id: generate_id(None),
    }
}

#[test]
fn metadata_new_sets_defaults() {
    let m = HookMetadata::new("my_hook", "@@rustpress.eco.Order.create@@");
    assert_eq!(m.name, "my_hook");
    assert_eq!(m.trigger, "@@rustpress.eco.Order.create@@");
    assert_eq!(m.timing, TriggerTiming::Before);
    assert_eq!(m.priority, 100);
    assert!(m.enabled);
    assert!(m.tags.is_empty());
    assert!(m.display_name.is_none());
    assert!(m.description.is_none());
}

#[test]
fn metadata_builder_chain() {
    let m = HookMetadata::new("h", "@@rustpress.a.B.c@@")
        .with_timing(TriggerTiming::After)
        .with_display_name("Display")
        .with_description("Desc")
        .with_priority(5)
        .with_enabled(false)
        .with_tags(vec!["a".into(), "b".into()]);

    assert_eq!(m.timing, TriggerTiming::After);
    assert_eq!(m.display_name.as_deref(), Some("Display"));
    assert_eq!(m.description.as_deref(), Some("Desc"));
    assert_eq!(m.priority, 5);
    assert!(!m.enabled);
    assert_eq!(m.tags, vec!["a".to_string(), "b".to_string()]);
}

#[test]
fn metadata_serde_roundtrip() {
    let m = HookMetadata::new("h", "@@rustpress.a.B.c@@")
        .with_timing(TriggerTiming::After)
        .with_description("d")
        .with_priority(50);
    let s = serde_json::to_string(&m).unwrap();
    let back: HookMetadata = serde_json::from_str(&s).unwrap();
    assert_eq!(back.name, m.name);
    assert_eq!(back.trigger, m.trigger);
    assert_eq!(back.timing, m.timing);
    assert_eq!(back.priority, 50);
    assert_eq!(back.description.as_deref(), Some("d"));
}

#[test]
fn metadata_deserialize_with_defaults() {
    // priority, enabled, tags should fall back to defaults when missing.
    let json = r#"{
        "name": "h",
        "trigger": "@@rustpress.a.B.c@@",
        "timing": "before"
    }"#;
    let m: HookMetadata = serde_json::from_str(json).unwrap();
    assert_eq!(m.priority, 100);
    assert!(m.enabled);
    assert!(m.tags.is_empty());
}

#[test]
fn define_hook_succeeds_with_valid_metadata() {
    let metadata = HookMetadata::new("valid_hook", "@@rustpress.eco.Order.create@@");
    let hook = define_hook(metadata, |_args: TriggerArgs, _ctx| async { Ok(()) });
    assert!(hook.is_ok());
    let h = hook.unwrap();
    assert_eq!(h.metadata.name, "valid_hook");
    assert_eq!(h.metadata.timing, TriggerTiming::Before);
}

#[test]
fn define_hook_rejects_empty_name() {
    let metadata = HookMetadata::new("", "@@rustpress.eco.Order.create@@");
    let result = define_hook(metadata, |_args: TriggerArgs, _ctx| async { Ok(()) });
    assert!(matches!(result, Err(SdkError::HookValidation(_))));
}

#[test]
fn define_hook_rejects_empty_trigger() {
    let metadata = HookMetadata::new("h", "");
    let result = define_hook(metadata, |_args: TriggerArgs, _ctx| async { Ok(()) });
    assert!(matches!(result, Err(SdkError::HookValidation(_))));
}

#[test]
fn define_hook_rejects_invalid_trigger_pattern() {
    let metadata = HookMetadata::new("h", "not-a-trigger");
    let result = define_hook(metadata, |_args: TriggerArgs, _ctx| async { Ok(()) });
    assert!(matches!(result, Err(SdkError::InvalidTrigger(_))));
}

#[test]
fn define_hook_preserves_metadata_timing_after() {
    let metadata = HookMetadata::new("h", "@@rustpress.a.B.c@@").with_timing(TriggerTiming::After);
    let hook =
        define_hook(metadata, |_args: TriggerArgs, _ctx| async { Ok(()) }).expect("ok");
    assert_eq!(hook.metadata.timing, TriggerTiming::After);
}

#[test]
fn before_hook_uses_before_timing() {
    let hook = before_hook(
        "@@rustpress.eco.Order.create@@",
        |_args: TriggerArgs, _ctx| async { Ok(()) },
        None,
    )
    .expect("valid");
    assert_eq!(hook.metadata.timing, TriggerTiming::Before);
}

#[test]
fn before_hook_auto_names_when_none() {
    let hook = before_hook(
        "@@rustpress.eco.Order.create@@",
        |_args: TriggerArgs, _ctx| async { Ok(()) },
        None,
    )
    .expect("valid");
    // Non-alphanumeric chars (including `@` and `.`) get replaced with underscores.
    assert!(hook.metadata.name.starts_with("before_"));
    assert!(hook.metadata.name.contains("rustpress"));
    assert!(hook.metadata.name.contains("Order"));
}

#[test]
fn before_hook_respects_provided_name() {
    let hook = before_hook(
        "@@rustpress.eco.Order.create@@",
        |_args: TriggerArgs, _ctx| async { Ok(()) },
        Some("custom_before"),
    )
    .expect("valid");
    assert_eq!(hook.metadata.name, "custom_before");
}

#[test]
fn after_hook_uses_after_timing() {
    let hook = after_hook(
        "@@rustpress.eco.Order.create@@",
        |_args: TriggerArgs, _ctx| async { Ok(()) },
        None,
    )
    .expect("valid");
    assert_eq!(hook.metadata.timing, TriggerTiming::After);
}

#[test]
fn after_hook_auto_names_starts_with_after() {
    let hook = after_hook(
        "@@rustpress.eco.Order.create@@",
        |_args: TriggerArgs, _ctx| async { Ok(()) },
        None,
    )
    .expect("valid");
    assert!(hook.metadata.name.starts_with("after_"));
}

#[test]
fn after_hook_respects_provided_name() {
    let hook = after_hook(
        "@@rustpress.eco.Order.create@@",
        |_args: TriggerArgs, _ctx| async { Ok(()) },
        Some("audit_log"),
    )
    .expect("valid");
    assert_eq!(hook.metadata.name, "audit_log");
}

#[test]
fn before_hook_propagates_invalid_trigger() {
    let result = before_hook(
        "garbage",
        |_args: TriggerArgs, _ctx| async { Ok(()) },
        Some("name"),
    );
    assert!(matches!(result, Err(SdkError::InvalidTrigger(_))));
}

#[test]
fn after_hook_propagates_invalid_trigger() {
    let result = after_hook(
        "garbage",
        |_args: TriggerArgs, _ctx| async { Ok(()) },
        Some("name"),
    );
    assert!(matches!(result, Err(SdkError::InvalidTrigger(_))));
}

#[tokio::test]
async fn hook_handler_is_invoked_with_args_and_context() {
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;

    let called = Arc::new(AtomicBool::new(false));
    let called_clone = called.clone();

    let hook = define_hook(
        HookMetadata::new("invoke", "@@rustpress.eco.Order.create@@"),
        move |_args: TriggerArgs, _ctx| {
            let called = called_clone.clone();
            async move {
                called.store(true, Ordering::SeqCst);
                Ok(())
            }
        },
    )
    .expect("valid");

    let ctx = RustPressContext::new();
    let args = make_args();
    (hook.handler)(args, &ctx).await.expect("handler ok");
    assert!(called.load(Ordering::SeqCst));
}

#[tokio::test]
async fn hook_handler_can_return_error() {
    let hook = define_hook(
        HookMetadata::new("errs", "@@rustpress.eco.Order.create@@"),
        |_args: TriggerArgs, _ctx| async { Err(SdkError::Other("boom".into())) },
    )
    .expect("valid");

    let ctx = RustPressContext::new();
    let result = (hook.handler)(make_args(), &ctx).await;
    assert!(matches!(result, Err(SdkError::Other(ref m)) if m == "boom"));
}

#[test]
fn enum_defaults() {
    assert_eq!(TriggerTiming::default(), TriggerTiming::Before);
    assert_eq!(NotificationType::default(), NotificationType::Info);
    assert_eq!(NotificationChannel::default(), NotificationChannel::InApp);
    assert_eq!(ContentStatus::default(), ContentStatus::Draft);
}

#[test]
fn enum_serde_lowercase() {
    let s = serde_json::to_string(&TriggerTiming::After).unwrap();
    assert_eq!(s, "\"after\"");
    let s = serde_json::to_string(&NotificationType::Warning).unwrap();
    assert_eq!(s, "\"warning\"");
    let s = serde_json::to_string(&ContentStatus::Published).unwrap();
    assert_eq!(s, "\"published\"");
}

#[test]
fn notification_channel_kebab_case_serde() {
    let s = serde_json::to_string(&NotificationChannel::InApp).unwrap();
    assert_eq!(s, "\"in-app\"");
    let back: NotificationChannel = serde_json::from_str("\"in-app\"").unwrap();
    assert_eq!(back, NotificationChannel::InApp);
}

#[test]
fn trigger_args_serde_roundtrip() {
    let args = make_args();
    let s = serde_json::to_string(&args).unwrap();
    let back: TriggerArgs = serde_json::from_str(&s).unwrap();
    assert_eq!(back.trigger, args.trigger);
    assert_eq!(back.timing, args.timing);
    assert_eq!(back.trigger_id, args.trigger_id);
}
