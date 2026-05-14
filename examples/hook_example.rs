//! Minimal example of defining a hook with the RustPress SDK.
//!
//! Run with: `cargo run --example hook_example`

use rustpress_sdk::{
    define_hook, generate_id, HookMetadata, RustPressContext, TriggerArgs, TriggerTiming,
};

#[tokio::main]
async fn main() -> rustpress_sdk::Result<()> {
    // Build hook metadata using the fluent builder.
    let metadata = HookMetadata::new("validate_order", "@@rustpress.ecommerce.Order.create@@")
        .with_timing(TriggerTiming::Before)
        .with_description("Validates orders before creation")
        .with_priority(50);

    // Define a hook that runs before the Order.create operation.
    let hook = define_hook(metadata, |args: TriggerArgs, ctx| async move {
        ctx.logger
            .info(&format!("[hook] firing for trigger {}", args.trigger));
        ctx.logger
            .info(&format!("[hook] original args = {}", args.original_args));
        Ok(())
    })?;

    // Simulate the runtime invoking the hook with a synthetic TriggerArgs.
    let context = RustPressContext::new();
    let args = TriggerArgs {
        original_args: serde_json::json!({"order_id": 42}),
        result: None,
        trigger: hook.metadata.trigger.clone(),
        timing: hook.metadata.timing,
        timestamp: chrono::Utc::now(),
        trigger_id: generate_id(Some("trig")),
    };

    (hook.handler)(args, &context).await?;

    println!("Hook '{}' executed successfully", hook.metadata.name);
    Ok(())
}
