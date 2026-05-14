//! Integration tests for the `Plugin` trait contract.
//!
//! Exercises trait method dispatch, metadata accessors,
//! activation/deactivation lifecycle, and serde round-trips
//! for `PluginMetadata`, `User`, and `Session`.

use async_trait::async_trait;
use chrono::Utc;
use rustpress_sdk::{
    Plugin, PluginContext, PluginMetadata, Result, RustPressContext, Session, User,
};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;

struct FakePlugin {
    id: String,
    meta: PluginMetadata,
    activations: Arc<AtomicU32>,
    deactivations: Arc<AtomicU32>,
    activate_should_fail: bool,
}

impl FakePlugin {
    fn new(id: &str) -> Self {
        Self {
            id: id.into(),
            meta: PluginMetadata::new("Fake Plugin", "1.0.0")
                .with_description("Test plugin")
                .with_author("test"),
            activations: Arc::new(AtomicU32::new(0)),
            deactivations: Arc::new(AtomicU32::new(0)),
            activate_should_fail: false,
        }
    }
}

#[async_trait]
impl Plugin for FakePlugin {
    fn id(&self) -> &str {
        &self.id
    }

    fn metadata(&self) -> &PluginMetadata {
        &self.meta
    }

    async fn on_activate(&mut self, _context: &PluginContext) -> Result<()> {
        self.activations.fetch_add(1, Ordering::SeqCst);
        if self.activate_should_fail {
            Err(rustpress_sdk::SdkError::Plugin("activate failed".into()))
        } else {
            Ok(())
        }
    }

    async fn on_deactivate(&mut self) -> Result<()> {
        self.deactivations.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }
}

#[test]
fn plugin_metadata_new_defaults() {
    let m = PluginMetadata::new("My Plugin", "0.1.0");
    assert_eq!(m.name, "My Plugin");
    assert_eq!(m.version, "0.1.0");
    assert!(m.description.is_none());
    assert!(m.author.is_none());
    assert!(m.keywords.is_empty());
    assert!(m.dependencies.is_empty());
    assert!(m.permissions.is_empty());
}

#[test]
fn plugin_metadata_builder() {
    let m = PluginMetadata::new("p", "1.0.0")
        .with_description("desc")
        .with_author("alice");
    assert_eq!(m.description.as_deref(), Some("desc"));
    assert_eq!(m.author.as_deref(), Some("alice"));
}

#[test]
fn plugin_metadata_serde_roundtrip() {
    let mut deps = HashMap::new();
    deps.insert("other".to_string(), "^1.0".to_string());

    let mut m = PluginMetadata::new("p", "2.0.0").with_description("d");
    m.dependencies = deps;
    m.keywords = vec!["a".into(), "b".into()];
    m.permissions = vec!["read".into()];

    let s = serde_json::to_string(&m).unwrap();
    let back: PluginMetadata = serde_json::from_str(&s).unwrap();
    assert_eq!(back.name, m.name);
    assert_eq!(back.version, m.version);
    assert_eq!(back.dependencies.get("other").unwrap(), "^1.0");
    assert_eq!(back.keywords, vec!["a".to_string(), "b".to_string()]);
    assert_eq!(back.permissions, vec!["read".to_string()]);
}

#[test]
fn plugin_id_accessor() {
    let p = FakePlugin::new("fake");
    assert_eq!(p.id(), "fake");
}

#[test]
fn plugin_metadata_accessor() {
    let p = FakePlugin::new("fake");
    assert_eq!(p.metadata().name, "Fake Plugin");
    assert_eq!(p.metadata().version, "1.0.0");
    assert_eq!(p.metadata().description.as_deref(), Some("Test plugin"));
}

#[tokio::test]
async fn plugin_activation_lifecycle() {
    let mut p = FakePlugin::new("fake");
    let activations = p.activations.clone();
    let deactivations = p.deactivations.clone();

    let ctx = PluginContext {
        base: RustPressContext::new(),
    };

    p.on_activate(&ctx).await.expect("activate");
    assert_eq!(activations.load(Ordering::SeqCst), 1);
    assert_eq!(deactivations.load(Ordering::SeqCst), 0);

    p.on_deactivate().await.expect("deactivate");
    assert_eq!(deactivations.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn plugin_can_be_used_as_trait_object() {
    let mut boxed: Box<dyn Plugin> = Box::new(FakePlugin::new("dyn"));
    let ctx = PluginContext {
        base: RustPressContext::new(),
    };
    boxed.on_activate(&ctx).await.expect("activate via dyn");
    boxed.on_deactivate().await.expect("deactivate via dyn");
    assert_eq!(boxed.id(), "dyn");
    assert_eq!(boxed.metadata().version, "1.0.0");
}

#[tokio::test]
async fn plugin_activate_can_fail() {
    let mut p = FakePlugin::new("err");
    p.activate_should_fail = true;
    let ctx = PluginContext {
        base: RustPressContext::new(),
    };
    let result = p.on_activate(&ctx).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn plugin_default_on_deactivate_is_ok() {
    // A plugin that does not override on_deactivate should still get Ok(()).
    struct Minimal {
        meta: PluginMetadata,
    }

    #[async_trait]
    impl Plugin for Minimal {
        fn id(&self) -> &str {
            "minimal"
        }
        fn metadata(&self) -> &PluginMetadata {
            &self.meta
        }
        async fn on_activate(&mut self, _ctx: &PluginContext) -> Result<()> {
            Ok(())
        }
    }

    let mut m = Minimal {
        meta: PluginMetadata::new("m", "0.0.1"),
    };
    assert!(m.on_deactivate().await.is_ok());
}

#[test]
fn user_serde_with_minimal_fields() {
    let json = r#"{"id":"u1","email":"a@b.com","name":"Alice"}"#;
    let u: User = serde_json::from_str(json).unwrap();
    assert_eq!(u.id, "u1");
    assert_eq!(u.email, "a@b.com");
    assert_eq!(u.name, "Alice");
    assert!(u.roles.is_empty());
    assert!(u.permissions.is_empty());
    assert!(u.metadata.is_empty());
}

#[test]
fn user_serde_with_full_fields() {
    let u = User {
        id: "u1".into(),
        email: "a@b.com".into(),
        name: "Alice".into(),
        roles: vec!["admin".into(), "editor".into()],
        permissions: vec!["read".into(), "write".into()],
        metadata: {
            let mut m = HashMap::new();
            m.insert("age".to_string(), serde_json::json!(30));
            m
        },
    };
    let s = serde_json::to_string(&u).unwrap();
    let back: User = serde_json::from_str(&s).unwrap();
    assert_eq!(back.roles, u.roles);
    assert_eq!(back.permissions, u.permissions);
    assert_eq!(back.metadata.get("age").unwrap(), &serde_json::json!(30));
}

#[test]
fn session_serde_roundtrip() {
    let now = Utc::now();
    let s = Session {
        id: "sess".into(),
        user_id: "u1".into(),
        expires_at: now,
        created_at: now,
        ip_address: Some("127.0.0.1".into()),
        user_agent: Some("test-agent".into()),
    };
    let json = serde_json::to_string(&s).unwrap();
    let back: Session = serde_json::from_str(&json).unwrap();
    assert_eq!(back.id, s.id);
    assert_eq!(back.user_id, s.user_id);
    assert_eq!(back.ip_address.as_deref(), Some("127.0.0.1"));
    assert_eq!(back.user_agent.as_deref(), Some("test-agent"));
}

#[test]
fn session_serde_optional_fields_missing() {
    // ip_address and user_agent are Option<String> and may be omitted as null.
    let now = Utc::now();
    let s = Session {
        id: "sess".into(),
        user_id: "u1".into(),
        expires_at: now,
        created_at: now,
        ip_address: None,
        user_agent: None,
    };
    let json = serde_json::to_string(&s).unwrap();
    let back: Session = serde_json::from_str(&json).unwrap();
    assert!(back.ip_address.is_none());
    assert!(back.user_agent.is_none());
}

#[test]
fn rustpress_context_default_has_execution_id_and_logger() {
    let ctx = RustPressContext::default();
    assert!(!ctx.execution_id.is_empty());
    // logger is non-null by construction; exercise it.
    ctx.logger.info("hello from test");
}

#[test]
fn rustpress_context_with_user() {
    let u = User {
        id: "u".into(),
        email: "a@b".into(),
        name: "A".into(),
        roles: vec![],
        permissions: vec![],
        metadata: HashMap::new(),
    };
    let ctx = RustPressContext::new().with_user(u);
    assert!(ctx.user.is_some());
    assert_eq!(ctx.user.unwrap().id, "u");
}
