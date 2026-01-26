# rustpress-sdk - Rust SDK

Official Rust SDK for RustPress - Build hooks, plugins, themes, and apps.

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
rustpress-sdk = "1.0"
```

Or with cargo:

```bash
cargo add rustpress-sdk
```

## Quick Start

### Creating a Hook Function

```rust
use rustpress_sdk::{
    define_hook, HookMetadata, TriggerTiming, TriggerArgs, RustPressContext, Result,
};

#[tokio::main]
async fn main() -> Result<()> {
    let hook = define_hook(
        HookMetadata::new("validate_order", "@@rustpress.ecommerce.Order.create@@")
            .with_timing(TriggerTiming::Before)
            .with_display_name("Order Validation")
            .with_description("Validates orders before creation"),
        |args: TriggerArgs, context: &RustPressContext| async move {
            let order_data = &args.original_args;

            if let Some(items) = order_data.get("items").and_then(|v| v.as_array()) {
                if items.is_empty() {
                    return Err(rustpress_sdk::SdkError::Other(
                        "Order must have at least one item".into()
                    ));
                }
            }

            if let Some(total) = order_data.get("total").and_then(|v| v.as_f64()) {
                if total < 0.0 {
                    return Err(rustpress_sdk::SdkError::Other(
                        "Order total cannot be negative".into()
                    ));
                }
            }

            context.logger.info("Order validation passed");
            Ok(())
        },
    )?;

    Ok(())
}
```

### Using Shorthand Hook Functions

```rust
use rustpress_sdk::{before_hook, after_hook, TriggerArgs, RustPressContext, Result};

#[tokio::main]
async fn main() -> Result<()> {
    // Before hook
    let validate_order = before_hook(
        "@@rustpress.ecommerce.Order.create@@",
        |args: TriggerArgs, context: &RustPressContext| async move {
            // Validation logic
            Ok(())
        },
        Some("validate_order"),
    )?;

    // After hook
    let log_order = after_hook(
        "@@rustpress.ecommerce.Order.create@@",
        |args: TriggerArgs, context: &RustPressContext| async move {
            println!("Order created: {:?}", args.result);
            Ok(())
        },
        Some("log_order"),
    )?;

    Ok(())
}
```

### Creating a Plugin

```rust
use async_trait::async_trait;
use rustpress_sdk::{Plugin, PluginContext, PluginMetadata, Result};

struct MyPlugin {
    id: String,
    metadata: PluginMetadata,
}

impl MyPlugin {
    fn new() -> Self {
        Self {
            id: "my-plugin".to_string(),
            metadata: PluginMetadata::new("My Plugin", "1.0.0")
                .with_description("A sample plugin")
                .with_author("Your Name"),
        }
    }
}

#[async_trait]
impl Plugin for MyPlugin {
    fn id(&self) -> &str {
        &self.id
    }

    fn metadata(&self) -> &PluginMetadata {
        &self.metadata
    }

    async fn on_activate(&mut self, context: &PluginContext) -> Result<()> {
        context.base.logger.info("Plugin activated");
        Ok(())
    }

    async fn on_deactivate(&mut self) -> Result<()> {
        Ok(())
    }
}
```

## API Reference

### Trigger Utilities

```rust
use rustpress_sdk::{parse_trigger, build_trigger, is_valid_trigger};

// Parse a trigger pattern
let parts = parse_trigger("@@rustpress.ecommerce.Order.create@@");
if let Some(p) = parts {
    println!("Plugin: {}, Class: {}, Method: {}", p.plugin, p.class_name, p.method);
}

// Build a trigger pattern
let trigger = build_trigger("ecommerce", "Order", "create");
// "@@rustpress.ecommerce.Order.create@@"

// Validate a trigger pattern
assert!(is_valid_trigger("@@rustpress.ecommerce.Order.create@@"));
assert!(!is_valid_trigger("invalid"));
```

### Utility Functions

```rust
use rustpress_sdk::{sleep_ms, retry, generate_id, RetryOptions};

// Sleep
sleep_ms(1000).await; // Wait 1 second

// Retry with exponential backoff
let result = retry(
    || async { fetch_data().await },
    RetryOptions {
        attempts: 3,
        delay_ms: 1000,
        max_delay_ms: 30000,
    },
).await?;

// Generate unique ID
let id = generate_id(Some("order")); // "order_18a3b4c5d6e7..."
let id2 = generate_id(None); // "18a3b4c5d6e7..."
```

### HTTP Client

The HTTP client is available with the `http-client` feature (enabled by default):

```rust
use rustpress_sdk::SimpleHttpClient;
use std::collections::HashMap;

let mut headers = HashMap::new();
headers.insert("Authorization".to_string(), "Bearer token".to_string());

let client = SimpleHttpClient::new("https://api.example.com")
    .with_headers(headers);

let users = client.get("/users").await?;
let new_user = client.post("/users", serde_json::json!({ "name": "John" })).await?;
```

### Event Emitter

```rust
use rustpress_sdk::SimpleEventEmitter;

let emitter = SimpleEventEmitter::new();

emitter.on("user:created", |user| async move {
    println!("User created: {:?}", user);
}).await;

emitter.emit("user:created", serde_json::json!({ "id": 1, "name": "John" })).await;
```

## Type Reference

### TriggerArgs

```rust
pub struct TriggerArgs<TInput = serde_json::Value, TResult = serde_json::Value> {
    pub original_args: TInput,      // Arguments passed to the plugin function
    pub result: Option<TResult>,    // Result (only in AFTER hooks)
    pub trigger: String,            // The trigger pattern
    pub timing: TriggerTiming,      // Before or After
    pub timestamp: DateTime<Utc>,
    pub trigger_id: String,
}
```

### RustPressContext

```rust
pub struct RustPressContext {
    pub execution_id: String,
    pub user: Option<User>,
    pub logger: Arc<dyn Logger>,
}
```

## Enums

```rust
use rustpress_sdk::{
    TriggerTiming,       // Before, After
    NotificationType,    // Info, Success, Warning, Error
    NotificationChannel, // InApp, Email, Sms, Push, Slack, Webhook
    ContentStatus,       // Draft, Pending, Published, Scheduled, Archived
};
```

## Features

- `http-client` (default) - Include the `SimpleHttpClient` powered by reqwest

To disable default features:

```toml
[dependencies]
rustpress-sdk = { version = "1.0", default-features = false }
```

## License

MIT
