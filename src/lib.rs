//! # RustPress SDK
//!
//! Official Rust SDK for building hooks, plugins, themes, and apps for RustPress.
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use rustpress_sdk::{
//!     define_hook, HookMetadata, TriggerTiming, TriggerArgs, RustPressContext,
//! };
//!
//! #[tokio::main]
//! async fn main() {
//!     let hook = define_hook(
//!         HookMetadata::new("validate_order", "@@rustpress.ecommerce.Order.create@@")
//!             .with_timing(TriggerTiming::Before)
//!             .with_description("Validates orders before creation"),
//!         |args, context| Box::pin(async move {
//!             // Validation logic
//!             context.logger.info("Order validation passed");
//!             Ok(())
//!         }),
//!     );
//! }
//! ```

#![warn(missing_docs)]
#![cfg_attr(docsrs, feature(doc_cfg))]

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;
use thiserror::Error;
use tokio::sync::RwLock;
use tokio::time::sleep;
use uuid::Uuid;

// =============================================================================
// Error Types
// =============================================================================

/// SDK Error types
#[derive(Error, Debug)]
pub enum SdkError {
    /// Hook validation error
    #[error("Hook validation error: {0}")]
    HookValidation(String),

    /// Invalid trigger pattern
    #[error("Invalid trigger pattern: {0}")]
    InvalidTrigger(String),

    /// Plugin error
    #[error("Plugin error: {0}")]
    Plugin(String),

    /// HTTP error
    #[error("HTTP error: {0}")]
    Http(String),

    /// Serialization error
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// Other error
    #[error("{0}")]
    Other(String),
}

/// Result type for SDK operations
pub type Result<T> = std::result::Result<T, SdkError>;

// =============================================================================
// Enums
// =============================================================================

/// Hook timing options
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TriggerTiming {
    /// Execute before the main operation
    Before,
    /// Execute after the main operation
    After,
}

impl Default for TriggerTiming {
    fn default() -> Self {
        Self::Before
    }
}

/// Notification type options
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum NotificationType {
    /// Informational notification
    Info,
    /// Success notification
    Success,
    /// Warning notification
    Warning,
    /// Error notification
    Error,
}

impl Default for NotificationType {
    fn default() -> Self {
        Self::Info
    }
}

/// Notification channel options
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum NotificationChannel {
    /// In-app notification
    InApp,
    /// Email notification
    Email,
    /// SMS notification
    Sms,
    /// Push notification
    Push,
    /// Slack notification
    Slack,
    /// Webhook notification
    Webhook,
}

impl Default for NotificationChannel {
    fn default() -> Self {
        Self::InApp
    }
}

/// Content status options
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ContentStatus {
    /// Draft content
    Draft,
    /// Pending review
    Pending,
    /// Published content
    Published,
    /// Scheduled for publication
    Scheduled,
    /// Archived content
    Archived,
}

impl Default for ContentStatus {
    fn default() -> Self {
        Self::Draft
    }
}

// =============================================================================
// Data Structures
// =============================================================================

/// User information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    /// User ID
    pub id: String,
    /// User email
    pub email: String,
    /// User name
    pub name: String,
    /// User roles
    #[serde(default)]
    pub roles: Vec<String>,
    /// User permissions
    #[serde(default)]
    pub permissions: Vec<String>,
    /// Additional metadata
    #[serde(default)]
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Session information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    /// Session ID
    pub id: String,
    /// User ID
    pub user_id: String,
    /// Session expiration time
    pub expires_at: DateTime<Utc>,
    /// Session creation time
    pub created_at: DateTime<Utc>,
    /// Client IP address
    pub ip_address: Option<String>,
    /// Client user agent
    pub user_agent: Option<String>,
}

/// Arguments passed to hook functions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TriggerArgs<TInput = serde_json::Value, TResult = serde_json::Value> {
    /// Original arguments passed to the plugin function
    pub original_args: TInput,
    /// Result from the plugin function (only in AFTER hooks)
    pub result: Option<TResult>,
    /// The trigger pattern
    pub trigger: String,
    /// Hook timing
    pub timing: TriggerTiming,
    /// Trigger timestamp
    pub timestamp: DateTime<Utc>,
    /// Unique trigger execution ID
    pub trigger_id: String,
}

/// Hook metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookMetadata {
    /// Unique name for the hook
    pub name: String,
    /// Human-readable name
    pub display_name: Option<String>,
    /// Description of what the hook does
    pub description: Option<String>,
    /// Trigger pattern
    pub trigger: String,
    /// When to execute the hook
    pub timing: TriggerTiming,
    /// Version of the hook
    pub version: Option<String>,
    /// Author of the hook
    pub author: Option<String>,
    /// Tags for categorization
    #[serde(default)]
    pub tags: Vec<String>,
    /// Execution priority (lower runs first)
    #[serde(default = "default_priority")]
    pub priority: i32,
    /// Whether the hook is enabled
    #[serde(default = "default_enabled")]
    pub enabled: bool,
}

fn default_priority() -> i32 {
    100
}

fn default_enabled() -> bool {
    true
}

impl HookMetadata {
    /// Create new hook metadata
    pub fn new(name: impl Into<String>, trigger: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            display_name: None,
            description: None,
            trigger: trigger.into(),
            timing: TriggerTiming::Before,
            version: None,
            author: None,
            tags: Vec::new(),
            priority: 100,
            enabled: true,
        }
    }

    /// Set the timing
    pub fn with_timing(mut self, timing: TriggerTiming) -> Self {
        self.timing = timing;
        self
    }

    /// Set the display name
    pub fn with_display_name(mut self, display_name: impl Into<String>) -> Self {
        self.display_name = Some(display_name.into());
        self
    }

    /// Set the description
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Set the priority
    pub fn with_priority(mut self, priority: i32) -> Self {
        self.priority = priority;
        self
    }

    /// Set enabled status
    pub fn with_enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Add tags
    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags = tags;
        self
    }
}

/// Plugin metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginMetadata {
    /// Plugin display name
    pub name: String,
    /// Plugin version
    pub version: String,
    /// Plugin description
    pub description: Option<String>,
    /// Plugin author
    pub author: Option<String>,
    /// Plugin homepage URL
    pub homepage: Option<String>,
    /// Plugin repository URL
    pub repository: Option<String>,
    /// Plugin license
    pub license: Option<String>,
    /// Plugin keywords
    #[serde(default)]
    pub keywords: Vec<String>,
    /// Plugin dependencies
    #[serde(default)]
    pub dependencies: HashMap<String, String>,
    /// Required permissions
    #[serde(default)]
    pub permissions: Vec<String>,
}

impl PluginMetadata {
    /// Create new plugin metadata
    pub fn new(name: impl Into<String>, version: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            version: version.into(),
            description: None,
            author: None,
            homepage: None,
            repository: None,
            license: None,
            keywords: Vec::new(),
            dependencies: HashMap::new(),
            permissions: Vec::new(),
        }
    }

    /// Set the description
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Set the author
    pub fn with_author(mut self, author: impl Into<String>) -> Self {
        self.author = Some(author.into());
        self
    }
}

/// Theme metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeMetadata {
    /// Theme display name
    pub name: String,
    /// Theme version
    pub version: String,
    /// Theme description
    pub description: Option<String>,
    /// Theme author
    pub author: Option<String>,
    /// Theme screenshot URL
    pub screenshot: Option<String>,
    /// Supported features
    #[serde(default)]
    pub supports: Vec<String>,
}

/// App metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppMetadata {
    /// App display name
    pub name: String,
    /// App version
    pub version: String,
    /// App description
    pub description: Option<String>,
    /// App author
    pub author: Option<String>,
    /// App icon URL
    pub icon: Option<String>,
    /// Required permissions
    #[serde(default)]
    pub permissions: Vec<String>,
}

/// Notification data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Notification {
    /// Notification title
    pub title: String,
    /// Notification message
    pub message: String,
    /// Notification type
    #[serde(default)]
    pub notification_type: NotificationType,
    /// Notification channel
    #[serde(default)]
    pub channel: NotificationChannel,
    /// Additional data
    #[serde(default)]
    pub data: HashMap<String, serde_json::Value>,
}

// =============================================================================
// Service Traits
// =============================================================================

/// Logger service trait
#[async_trait]
pub trait Logger: Send + Sync {
    /// Log debug message
    fn debug(&self, message: &str);
    /// Log info message
    fn info(&self, message: &str);
    /// Log warning message
    fn warning(&self, message: &str);
    /// Log error message
    fn error(&self, message: &str, error: Option<&dyn std::error::Error>);
}

/// Simple console logger implementation
#[derive(Debug, Default)]
pub struct ConsoleLogger;

impl Logger for ConsoleLogger {
    fn debug(&self, message: &str) {
        println!("[DEBUG] {}", message);
    }

    fn info(&self, message: &str) {
        println!("[INFO] {}", message);
    }

    fn warning(&self, message: &str) {
        println!("[WARN] {}", message);
    }

    fn error(&self, message: &str, error: Option<&dyn std::error::Error>) {
        if let Some(e) = error {
            println!("[ERROR] {}: {}", message, e);
        } else {
            println!("[ERROR] {}", message);
        }
    }
}

/// Cache service trait
#[async_trait]
pub trait Cache: Send + Sync {
    /// Get a value from cache
    async fn get(&self, key: &str) -> Option<serde_json::Value>;
    /// Set a value in cache
    async fn set(&self, key: &str, value: serde_json::Value, ttl: Option<Duration>) -> Result<()>;
    /// Delete a value from cache
    async fn delete(&self, key: &str) -> Result<bool>;
    /// Check if key exists
    async fn exists(&self, key: &str) -> bool;
}

/// Queue service trait
#[async_trait]
pub trait QueueService: Send + Sync {
    /// Enqueue a message
    async fn enqueue(&self, queue: &str, message: serde_json::Value) -> Result<String>;
}

/// Notification service trait
#[async_trait]
pub trait NotificationService: Send + Sync {
    /// Send a notification
    async fn send(&self, notification: Notification) -> Result<String>;
    /// Send notification to specific user
    async fn send_to_user(&self, user_id: &str, notification: Notification) -> Result<String>;
}

/// Storage service trait
#[async_trait]
pub trait StorageService: Send + Sync {
    /// Store a file
    async fn put(&self, path: &str, content: &[u8]) -> Result<String>;
    /// Get a file
    async fn get(&self, path: &str) -> Result<Option<Vec<u8>>>;
    /// Delete a file
    async fn delete(&self, path: &str) -> Result<bool>;
    /// Check if file exists
    async fn exists(&self, path: &str) -> Result<bool>;
    /// Get public URL for file
    fn url(&self, path: &str) -> String;
}

/// Config service trait
#[async_trait]
pub trait ConfigService: Send + Sync {
    /// Get config value
    fn get(&self, key: &str) -> Option<serde_json::Value>;
    /// Set config value
    async fn set(&self, key: &str, value: serde_json::Value) -> Result<()>;
}

// =============================================================================
// Context
// =============================================================================

/// Context provided to hook functions
pub struct RustPressContext {
    /// Unique execution ID for tracing
    pub execution_id: String,
    /// Current authenticated user
    pub user: Option<User>,
    /// Logger service
    pub logger: Arc<dyn Logger>,
}

impl RustPressContext {
    /// Create a new context with default services
    pub fn new() -> Self {
        Self {
            execution_id: generate_id(None),
            user: None,
            logger: Arc::new(ConsoleLogger),
        }
    }

    /// Set the user
    pub fn with_user(mut self, user: User) -> Self {
        self.user = Some(user);
        self
    }

    /// Set the logger
    pub fn with_logger(mut self, logger: Arc<dyn Logger>) -> Self {
        self.logger = logger;
        self
    }
}

impl Default for RustPressContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Context provided to plugins
pub struct PluginContext {
    /// Base context
    pub base: RustPressContext,
}

// =============================================================================
// Hook Functions
// =============================================================================

/// Hook handler function type
pub type HookHandler<TInput = serde_json::Value, TResult = serde_json::Value> = Box<
    dyn Fn(
            TriggerArgs<TInput, TResult>,
            &RustPressContext,
        ) -> Pin<Box<dyn Future<Output = Result<()>> + Send + '_>>
        + Send
        + Sync,
>;

/// Hook function definition
pub struct HookFunction<TInput = serde_json::Value, TResult = serde_json::Value> {
    /// Hook metadata
    pub metadata: HookMetadata,
    /// Hook handler
    pub handler: HookHandler<TInput, TResult>,
}

/// Define a hook function
pub fn define_hook<TInput, TResult, F, Fut>(
    metadata: HookMetadata,
    handler: F,
) -> Result<HookFunction<TInput, TResult>>
where
    TInput: Send + 'static,
    TResult: Send + 'static,
    F: Fn(TriggerArgs<TInput, TResult>, &RustPressContext) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = Result<()>> + Send + 'static,
{
    if metadata.name.is_empty() {
        return Err(SdkError::HookValidation("Hook name is required".into()));
    }
    if metadata.trigger.is_empty() {
        return Err(SdkError::HookValidation("Hook trigger is required".into()));
    }
    if !is_valid_trigger(&metadata.trigger) {
        return Err(SdkError::InvalidTrigger(metadata.trigger.clone()));
    }

    Ok(HookFunction {
        metadata,
        handler: Box::new(move |args, ctx| Box::pin(handler(args, ctx))),
    })
}

/// Create a before hook (shorthand)
pub fn before_hook<TInput, TResult, F, Fut>(
    trigger: &str,
    handler: F,
    name: Option<&str>,
) -> Result<HookFunction<TInput, TResult>>
where
    TInput: Send + 'static,
    TResult: Send + 'static,
    F: Fn(TriggerArgs<TInput, TResult>, &RustPressContext) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = Result<()>> + Send + 'static,
{
    let hook_name = name
        .map(String::from)
        .unwrap_or_else(|| format!("before_{}", trigger.replace(|c: char| !c.is_alphanumeric(), "_")));

    let metadata = HookMetadata::new(hook_name, trigger).with_timing(TriggerTiming::Before);

    define_hook(metadata, handler)
}

/// Create an after hook (shorthand)
pub fn after_hook<TInput, TResult, F, Fut>(
    trigger: &str,
    handler: F,
    name: Option<&str>,
) -> Result<HookFunction<TInput, TResult>>
where
    TInput: Send + 'static,
    TResult: Send + 'static,
    F: Fn(TriggerArgs<TInput, TResult>, &RustPressContext) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = Result<()>> + Send + 'static,
{
    let hook_name = name
        .map(String::from)
        .unwrap_or_else(|| format!("after_{}", trigger.replace(|c: char| !c.is_alphanumeric(), "_")));

    let metadata = HookMetadata::new(hook_name, trigger).with_timing(TriggerTiming::After);

    define_hook(metadata, handler)
}

// =============================================================================
// Plugin Base Trait
// =============================================================================

/// Plugin trait
#[async_trait]
pub trait Plugin: Send + Sync {
    /// Get plugin ID
    fn id(&self) -> &str;

    /// Get plugin metadata
    fn metadata(&self) -> &PluginMetadata;

    /// Called when the plugin is activated
    async fn on_activate(&mut self, context: &PluginContext) -> Result<()>;

    /// Called when the plugin is deactivated
    async fn on_deactivate(&mut self) -> Result<()> {
        Ok(())
    }
}

/// Theme trait
#[async_trait]
pub trait Theme: Send + Sync {
    /// Get theme ID
    fn id(&self) -> &str;

    /// Get theme metadata
    fn metadata(&self) -> &ThemeMetadata;

    /// Called when the theme is activated
    async fn on_activate(&mut self, context: &PluginContext) -> Result<()>;

    /// Called when the theme is deactivated
    async fn on_deactivate(&mut self) -> Result<()> {
        Ok(())
    }
}

/// App trait
#[async_trait]
pub trait App: Send + Sync {
    /// Get app ID
    fn id(&self) -> &str;

    /// Get app metadata
    fn metadata(&self) -> &AppMetadata;

    /// Called when the app is activated
    async fn on_activate(&mut self, context: &PluginContext) -> Result<()>;

    /// Called when the app is deactivated
    async fn on_deactivate(&mut self) -> Result<()> {
        Ok(())
    }
}

// =============================================================================
// Trigger Utilities
// =============================================================================

lazy_static::lazy_static! {
    static ref TRIGGER_PATTERN: Regex = Regex::new(
        r"^@@rustpress\.([a-zA-Z0-9_-]+)\.([a-zA-Z0-9_]+)\.([a-zA-Z0-9_]+)@@$"
    ).unwrap();
}

/// Parsed trigger components
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TriggerParts {
    /// Plugin name
    pub plugin: String,
    /// Class name
    pub class_name: String,
    /// Method name
    pub method: String,
}

/// Parse a trigger pattern into its components
pub fn parse_trigger(trigger: &str) -> Option<TriggerParts> {
    TRIGGER_PATTERN.captures(trigger).map(|caps| TriggerParts {
        plugin: caps.get(1).unwrap().as_str().to_string(),
        class_name: caps.get(2).unwrap().as_str().to_string(),
        method: caps.get(3).unwrap().as_str().to_string(),
    })
}

/// Build a trigger pattern from components
pub fn build_trigger(plugin: &str, class_name: &str, method: &str) -> String {
    format!("@@rustpress.{}.{}.{}@@", plugin, class_name, method)
}

/// Validate a trigger pattern
pub fn is_valid_trigger(trigger: &str) -> bool {
    TRIGGER_PATTERN.is_match(trigger)
}

// =============================================================================
// Utility Functions
// =============================================================================

/// Sleep for a specified duration
pub async fn sleep_ms(ms: u64) {
    sleep(Duration::from_millis(ms)).await;
}

/// Retry options
#[derive(Debug, Clone)]
pub struct RetryOptions {
    /// Number of attempts
    pub attempts: u32,
    /// Initial delay in milliseconds
    pub delay_ms: u64,
    /// Maximum delay in milliseconds
    pub max_delay_ms: u64,
}

impl Default for RetryOptions {
    fn default() -> Self {
        Self {
            attempts: 3,
            delay_ms: 1000,
            max_delay_ms: 30000,
        }
    }
}

/// Retry a function with exponential backoff
pub async fn retry<T, E, F, Fut>(f: F, options: RetryOptions) -> std::result::Result<T, E>
where
    F: Fn() -> Fut,
    Fut: Future<Output = std::result::Result<T, E>>,
{
    let mut last_error: Option<E> = None;

    for i in 0..options.attempts {
        match f().await {
            Ok(result) => return Ok(result),
            Err(e) => {
                last_error = Some(e);
                if i < options.attempts - 1 {
                    let wait_time = std::cmp::min(
                        options.delay_ms * 2u64.pow(i),
                        options.max_delay_ms,
                    );
                    sleep(Duration::from_millis(wait_time)).await;
                }
            }
        }
    }

    Err(last_error.unwrap())
}

/// Generate a unique ID
pub fn generate_id(prefix: Option<&str>) -> String {
    let uuid = Uuid::new_v4();
    let timestamp = chrono::Utc::now().timestamp_millis();
    let id = format!("{:x}{}", timestamp, &uuid.to_string()[..12]);

    match prefix {
        Some(p) => format!("{}_{}", p, id),
        None => id,
    }
}

// =============================================================================
// Simple Event Emitter
// =============================================================================

/// Event handler type
pub type EventHandler = Box<dyn Fn(serde_json::Value) -> Pin<Box<dyn Future<Output = ()> + Send>> + Send + Sync>;

/// Simple async event emitter
pub struct SimpleEventEmitter {
    events: RwLock<HashMap<String, Vec<Arc<EventHandler>>>>,
}

impl SimpleEventEmitter {
    /// Create a new event emitter
    pub fn new() -> Self {
        Self {
            events: RwLock::new(HashMap::new()),
        }
    }

    /// Register an event handler
    pub async fn on<F, Fut>(&self, event: &str, handler: F)
    where
        F: Fn(serde_json::Value) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        let mut events = self.events.write().await;
        let handlers = events.entry(event.to_string()).or_insert_with(Vec::new);
        handlers.push(Arc::new(Box::new(move |data| Box::pin(handler(data)))));
    }

    /// Emit an event
    pub async fn emit(&self, event: &str, data: serde_json::Value) {
        let events = self.events.read().await;
        if let Some(handlers) = events.get(event) {
            for handler in handlers {
                handler(data.clone()).await;
            }
        }
    }
}

impl Default for SimpleEventEmitter {
    fn default() -> Self {
        Self::new()
    }
}

// =============================================================================
// HTTP Client (optional feature)
// =============================================================================

#[cfg(feature = "http-client")]
mod http_client {
    use super::*;
    use reqwest::Client;

    /// Simple HTTP client
    pub struct SimpleHttpClient {
        client: Client,
        base_url: String,
        default_headers: HashMap<String, String>,
    }

    impl SimpleHttpClient {
        /// Create a new HTTP client
        pub fn new(base_url: impl Into<String>) -> Self {
            Self {
                client: Client::new(),
                base_url: base_url.into(),
                default_headers: HashMap::new(),
            }
        }

        /// Set default headers
        pub fn with_headers(mut self, headers: HashMap<String, String>) -> Self {
            self.default_headers = headers;
            self
        }

        /// Make a GET request
        pub async fn get(&self, url: &str) -> Result<serde_json::Value> {
            let full_url = format!("{}{}", self.base_url, url);
            let mut request = self.client.get(&full_url);

            for (key, value) in &self.default_headers {
                request = request.header(key, value);
            }

            let response = request
                .send()
                .await
                .map_err(|e| SdkError::Http(e.to_string()))?;

            response
                .json()
                .await
                .map_err(|e| SdkError::Http(e.to_string()))
        }

        /// Make a POST request
        pub async fn post(&self, url: &str, body: serde_json::Value) -> Result<serde_json::Value> {
            let full_url = format!("{}{}", self.base_url, url);
            let mut request = self.client.post(&full_url).json(&body);

            for (key, value) in &self.default_headers {
                request = request.header(key, value);
            }

            let response = request
                .send()
                .await
                .map_err(|e| SdkError::Http(e.to_string()))?;

            response
                .json()
                .await
                .map_err(|e| SdkError::Http(e.to_string()))
        }

        /// Make a PUT request
        pub async fn put(&self, url: &str, body: serde_json::Value) -> Result<serde_json::Value> {
            let full_url = format!("{}{}", self.base_url, url);
            let mut request = self.client.put(&full_url).json(&body);

            for (key, value) in &self.default_headers {
                request = request.header(key, value);
            }

            let response = request
                .send()
                .await
                .map_err(|e| SdkError::Http(e.to_string()))?;

            response
                .json()
                .await
                .map_err(|e| SdkError::Http(e.to_string()))
        }

        /// Make a DELETE request
        pub async fn delete(&self, url: &str) -> Result<serde_json::Value> {
            let full_url = format!("{}{}", self.base_url, url);
            let mut request = self.client.delete(&full_url);

            for (key, value) in &self.default_headers {
                request = request.header(key, value);
            }

            let response = request
                .send()
                .await
                .map_err(|e| SdkError::Http(e.to_string()))?;

            response
                .json()
                .await
                .map_err(|e| SdkError::Http(e.to_string()))
        }
    }
}

#[cfg(feature = "http-client")]
pub use http_client::SimpleHttpClient;

// =============================================================================
// Re-exports
// =============================================================================

pub use lazy_static::lazy_static;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_trigger() {
        let result = parse_trigger("@@rustpress.ecommerce.Order.create@@");
        assert!(result.is_some());
        let parts = result.unwrap();
        assert_eq!(parts.plugin, "ecommerce");
        assert_eq!(parts.class_name, "Order");
        assert_eq!(parts.method, "create");
    }

    #[test]
    fn test_build_trigger() {
        let trigger = build_trigger("ecommerce", "Order", "create");
        assert_eq!(trigger, "@@rustpress.ecommerce.Order.create@@");
    }

    #[test]
    fn test_is_valid_trigger() {
        assert!(is_valid_trigger("@@rustpress.ecommerce.Order.create@@"));
        assert!(!is_valid_trigger("invalid"));
        assert!(!is_valid_trigger("@@rustpress.invalid@@"));
    }

    #[test]
    fn test_generate_id() {
        let id1 = generate_id(None);
        let id2 = generate_id(None);
        assert_ne!(id1, id2);

        let prefixed = generate_id(Some("order"));
        assert!(prefixed.starts_with("order_"));
    }
}
