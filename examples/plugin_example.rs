//! Minimal example of implementing the `Plugin` trait.
//!
//! Run with: `cargo run --example plugin_example`

use async_trait::async_trait;
use rustpress_sdk::{Plugin, PluginContext, PluginMetadata, Result, RustPressContext};

/// A trivial plugin that logs on activation and deactivation.
pub struct HelloPlugin {
    id: String,
    metadata: PluginMetadata,
}

impl HelloPlugin {
    pub fn new() -> Self {
        Self {
            id: "hello-plugin".to_string(),
            metadata: PluginMetadata::new("Hello Plugin", "0.1.0")
                .with_description("A minimal example plugin")
                .with_author("RustPress Team"),
        }
    }
}

impl Default for HelloPlugin {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Plugin for HelloPlugin {
    fn id(&self) -> &str {
        &self.id
    }

    fn metadata(&self) -> &PluginMetadata {
        &self.metadata
    }

    async fn on_activate(&mut self, context: &PluginContext) -> Result<()> {
        context
            .base
            .logger
            .info(&format!("[{}] activated", self.id));
        Ok(())
    }

    async fn on_deactivate(&mut self) -> Result<()> {
        println!("[{}] deactivated", self.id);
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let mut plugin = HelloPlugin::new();
    let context = PluginContext {
        base: RustPressContext::new(),
    };

    plugin.on_activate(&context).await?;
    println!(
        "Plugin '{}' v{} ready",
        plugin.metadata().name,
        plugin.metadata().version
    );
    plugin.on_deactivate().await?;
    Ok(())
}
