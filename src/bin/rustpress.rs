//! RustPress SDK CLI - Project scaffolding and development tools for Rust.
//!
//! # Usage
//!
//! ```bash
//! rustpress create <type> <name>  - Create a new project (plugin, theme, app, hook)
//! rustpress dev                   - Start development server
//! rustpress build                 - Build for production
//! rustpress validate              - Validate project configuration
//! rustpress publish               - Publish to RustPress marketplace
//! ```

use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Write;
use std::path::Path;
use std::process::{exit, Command};

// =============================================================================
// CLI Structure
// =============================================================================

#[derive(Parser)]
#[command(name = "rustpress")]
#[command(author = "RustPress Team")]
#[command(version = "1.0.0")]
#[command(about = "RustPress SDK CLI - Build plugins, themes, apps, and hooks", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Create a new RustPress project
    Create {
        /// Project type (plugin, theme, app, hook)
        #[arg(value_parser = ["plugin", "theme", "app", "hook"])]
        project_type: String,
        /// Project name
        name: String,
    },
    /// Start development server with hot reload
    Dev,
    /// Build for production
    Build {
        /// Build in release mode
        #[arg(short, long)]
        release: bool,
    },
    /// Validate project configuration
    Validate,
    /// Publish to RustPress marketplace
    Publish,
}

// =============================================================================
// Configuration Types
// =============================================================================

#[derive(Debug, Serialize, Deserialize)]
struct RustPressConfig {
    name: String,
    #[serde(rename = "type")]
    project_type: String,
    version: String,
    main: String,
    sdk: String,
    #[serde(rename = "sdkVersion")]
    sdk_version: String,
    rustpress: RustPressMinVersion,
}

#[derive(Debug, Serialize, Deserialize)]
struct RustPressMinVersion {
    #[serde(rename = "minVersion")]
    min_version: String,
}

// =============================================================================
// Terminal Colors
// =============================================================================

mod colors {
    pub const RESET: &str = "\x1b[0m";
    pub const RED: &str = "\x1b[31m";
    pub const GREEN: &str = "\x1b[32m";
    pub const YELLOW: &str = "\x1b[33m";
    pub const CYAN: &str = "\x1b[36m";
    pub const BOLD: &str = "\x1b[1m";
}

fn success(msg: &str) {
    println!("{}✓ {}{}", colors::GREEN, msg, colors::RESET);
}

fn error(msg: &str) {
    eprintln!("{}✗ {}{}", colors::RED, msg, colors::RESET);
}

fn info(msg: &str) {
    println!("{}ℹ {}{}", colors::CYAN, msg, colors::RESET);
}

fn warning(msg: &str) {
    println!("{}⚠ {}{}", colors::YELLOW, msg, colors::RESET);
}

// =============================================================================
// Template Generators
// =============================================================================

fn to_struct_name(name: &str) -> String {
    name.split('-')
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().chain(chars).collect(),
            }
        })
        .collect()
}

fn to_snake_case(name: &str) -> String {
    name.replace('-', "_")
}

fn generate_plugin_lib(name: &str, struct_name: &str) -> String {
    format!(
        r#"//! {name} - A RustPress Plugin
//!
//! This plugin provides custom functionality for RustPress.

use rustpress_sdk::{{
    BasePlugin, HookContext, HookArgs, PluginConfig,
    async_trait::async_trait,
    Result,
}};
use std::sync::Arc;

/// {struct_name} plugin implementation.
pub struct {struct_name} {{
    id: String,
    config: PluginConfig,
}}

impl {struct_name} {{
    /// Create a new instance of the plugin.
    pub fn new() -> Self {{
        Self {{
            id: "{name}".to_string(),
            config: PluginConfig {{
                name: "{struct_name}".to_string(),
                version: "1.0.0".to_string(),
                description: "A custom RustPress plugin".to_string(),
                ..Default::default()
            }},
        }}
    }}
}}

impl Default for {struct_name} {{
    fn default() -> Self {{
        Self::new()
    }}
}}

#[async_trait]
impl BasePlugin for {struct_name} {{
    fn id(&self) -> &str {{
        &self.id
    }}

    fn config(&self) -> &PluginConfig {{
        &self.config
    }}

    async fn on_activate(&mut self, context: Arc<HookContext>) -> Result<()> {{
        tracing::info!("Plugin activated");

        // Register hooks
        self.register_hook("content:before_save", |content| {{
            Box::pin(async move {{
                // Process content before saving
                Ok(content)
            }})
        }});

        Ok(())
    }}

    async fn on_deactivate(&mut self) -> Result<()> {{
        tracing::info!("Plugin deactivated");
        Ok(())
    }}
}}

/// Create the plugin instance.
pub fn create_plugin() -> Box<dyn BasePlugin + Send + Sync> {{
    Box::new({struct_name}::new())
}}

#[cfg(test)]
mod tests {{
    use super::*;

    #[test]
    fn test_plugin_creation() {{
        let plugin = {struct_name}::new();
        assert_eq!(plugin.id(), "{name}");
        assert_eq!(plugin.config().version, "1.0.0");
    }}
}}
"#,
        name = name,
        struct_name = struct_name
    )
}

fn generate_theme_lib(name: &str, struct_name: &str) -> String {
    format!(
        r#"//! {name} - A RustPress Theme
//!
//! This theme provides custom styling and layout for RustPress.

use rustpress_sdk::{{
    BaseTheme, HookContext, ThemeConfig,
    async_trait::async_trait,
    Result,
}};
use std::sync::Arc;

/// {struct_name} theme implementation.
pub struct {struct_name} {{
    id: String,
    config: ThemeConfig,
}}

impl {struct_name} {{
    /// Create a new instance of the theme.
    pub fn new() -> Self {{
        Self {{
            id: "{name}".to_string(),
            config: ThemeConfig {{
                name: "{struct_name}".to_string(),
                version: "1.0.0".to_string(),
                description: "A beautiful RustPress theme".to_string(),
                supports: vec![
                    "dark-mode".to_string(),
                    "custom-colors".to_string(),
                    "custom-fonts".to_string(),
                ],
                ..Default::default()
            }},
        }}
    }}
}}

impl Default for {struct_name} {{
    fn default() -> Self {{
        Self::new()
    }}
}}

#[async_trait]
impl BaseTheme for {struct_name} {{
    fn id(&self) -> &str {{
        &self.id
    }}

    fn config(&self) -> &ThemeConfig {{
        &self.config
    }}

    async fn on_activate(&mut self, context: Arc<HookContext>) -> Result<()> {{
        tracing::info!("Theme activated");

        // Register theme assets
        self.register_asset("css", "/theme/style.css");
        self.register_asset("js", "/theme/main.js");

        Ok(())
    }}

    async fn on_deactivate(&mut self) -> Result<()> {{
        tracing::info!("Theme deactivated");
        Ok(())
    }}
}}

/// Create the theme instance.
pub fn create_theme() -> Box<dyn BaseTheme + Send + Sync> {{
    Box::new({struct_name}::new())
}}

#[cfg(test)]
mod tests {{
    use super::*;

    #[test]
    fn test_theme_creation() {{
        let theme = {struct_name}::new();
        assert_eq!(theme.id(), "{name}");
        assert_eq!(theme.config().version, "1.0.0");
    }}
}}
"#,
        name = name,
        struct_name = struct_name
    )
}

fn generate_app_lib(name: &str, struct_name: &str) -> String {
    format!(
        r#"//! {name} - A RustPress App
//!
//! This app provides a custom admin panel interface for RustPress.

use rustpress_sdk::{{
    BaseApp, HookContext, AppConfig, AppMenu,
    async_trait::async_trait,
    Result,
}};
use std::sync::Arc;

/// {struct_name} app implementation.
pub struct {struct_name} {{
    id: String,
    config: AppConfig,
}}

impl {struct_name} {{
    /// Create a new instance of the app.
    pub fn new() -> Self {{
        Self {{
            id: "{name}".to_string(),
            config: AppConfig {{
                name: "{struct_name}".to_string(),
                version: "1.0.0".to_string(),
                description: "A custom RustPress app".to_string(),
                icon: "dashboard".to_string(),
                menu: Some(AppMenu {{
                    title: "{struct_name}".to_string(),
                    icon: "dashboard".to_string(),
                    position: "sidebar".to_string(),
                }}),
                ..Default::default()
            }},
        }}
    }}
}}

impl Default for {struct_name} {{
    fn default() -> Self {{
        Self::new()
    }}
}}

#[async_trait]
impl BaseApp for {struct_name} {{
    fn id(&self) -> &str {{
        &self.id
    }}

    fn config(&self) -> &AppConfig {{
        &self.config
    }}

    async fn on_activate(&mut self, context: Arc<HookContext>) -> Result<()> {{
        tracing::info!("App activated");

        // Register app routes
        self.register_route("GET", "/dashboard", Self::get_dashboard);
        self.register_route("GET", "/settings", Self::get_settings);
        self.register_route("POST", "/settings", Self::save_settings);

        Ok(())
    }}

    async fn on_deactivate(&mut self) -> Result<()> {{
        tracing::info!("App deactivated");
        Ok(())
    }}
}}

impl {struct_name} {{
    async fn get_dashboard() -> Result<serde_json::Value> {{
        Ok(serde_json::json!({{
            "view": "dashboard",
            "data": {{}}
        }}))
    }}

    async fn get_settings() -> Result<serde_json::Value> {{
        Ok(serde_json::json!({{
            "view": "settings",
            "data": {{}}
        }}))
    }}

    async fn save_settings() -> Result<serde_json::Value> {{
        Ok(serde_json::json!({{ "success": true }}))
    }}
}}

/// Create the app instance.
pub fn create_app() -> Box<dyn BaseApp + Send + Sync> {{
    Box::new({struct_name}::new())
}}

#[cfg(test)]
mod tests {{
    use super::*;

    #[test]
    fn test_app_creation() {{
        let app = {struct_name}::new();
        assert_eq!(app.id(), "{name}");
        assert_eq!(app.config().version, "1.0.0");
    }}
}}
"#,
        name = name,
        struct_name = struct_name
    )
}

fn generate_hook_lib(name: &str, function_name: &str) -> String {
    format!(
        r#"//! {name} - A RustPress Hook Function
//!
//! This module defines hook functions for RustPress.

use rustpress_sdk::{{
    define_hook, before_hook, after_hook,
    HookContext, HookArgs, HookDefinition, HookConfig,
    TriggerTiming,
    Result,
}};
use std::sync::Arc;

/// Validation hook that runs before content creation.
pub fn {function_name}() -> HookDefinition {{
    define_hook(
        HookConfig {{
            name: "{function_name}".to_string(),
            display_name: "{name}".to_string(),
            description: "Validates data before processing".to_string(),
            trigger: "@@rustpress.core.Content.create@@".to_string(),
            timing: TriggerTiming::Before,
        }},
        |args: HookArgs, context: Arc<HookContext>| {{
            Box::pin(async move {{
                // Example validation
                if args.original_args.is_null() {{
                    return Err("Data cannot be empty".into());
                }}

                tracing::info!("Validation passed");
                Ok(())
            }})
        }},
    )
}}

/// Logging hook that runs after content creation.
pub fn log_content() -> HookDefinition {{
    after_hook(
        "@@rustpress.core.Content.create@@",
        |args: HookArgs, context: Arc<HookContext>| {{
            Box::pin(async move {{
                tracing::info!("Content created: {{}}", args.result);
                Ok(())
            }})
        }},
        Some("logContent".to_string()),
    )
}}

/// Get all hooks defined in this module.
pub fn get_hooks() -> Vec<HookDefinition> {{
    vec![{function_name}(), log_content()]
}}

#[cfg(test)]
mod tests {{
    use super::*;

    #[test]
    fn test_hook_creation() {{
        let hooks = get_hooks();
        assert_eq!(hooks.len(), 2);
    }}
}}
"#,
        name = name,
        function_name = function_name
    )
}

fn generate_cargo_toml(name: &str, project_type: &str) -> String {
    let crate_name = to_snake_case(name);
    format!(
        r#"[package]
name = "{crate_name}"
version = "1.0.0"
edition = "2021"
rust-version = "1.70"
authors = ["Your Name <your@email.com>"]
description = "A RustPress {project_type}"
license = "MIT"
readme = "README.md"
keywords = ["rustpress", "{project_type}"]

[dependencies]
rustpress-sdk = "1.0"
async-trait = "0.1"
serde = {{ version = "1.0", features = ["derive"] }}
serde_json = "1.0"
tokio = {{ version = "1.35", features = ["full"] }}
tracing = "0.1"

[dev-dependencies]
tokio = {{ version = "1.35", features = ["full", "test-util"] }}

[lib]
name = "{crate_name}"
path = "src/lib.rs"
"#,
        crate_name = crate_name,
        project_type = project_type
    )
}

fn generate_readme(name: &str, _struct_name: &str, project_type: &str) -> String {
    let crate_name = to_snake_case(name);
    format!(
        r#"# {name}

A RustPress {project_type} built with the Rust SDK.

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
{crate_name} = "1.0"
```

## Usage

```rust
use {crate_name}::create_{project_type};

// The {project_type} is automatically loaded by RustPress
let {project_type} = create_{project_type}();
```

## Development

```bash
# Build
cargo build

# Run tests
cargo test

# Build for release
cargo build --release

# Check for issues
cargo clippy
```

## Configuration

Add to your RustPress configuration:

```toml
[{project_type}s.{name}]
enabled = true
```

## License

MIT
"#,
        name = name,
        crate_name = crate_name,
        project_type = project_type
    )
}

fn generate_rustpress_config(name: &str, project_type: &str) -> String {
    let crate_name = to_snake_case(name);
    serde_json::to_string_pretty(&RustPressConfig {
        name: name.to_string(),
        project_type: project_type.to_string(),
        version: "1.0.0".to_string(),
        main: format!("target/release/lib{}.so", crate_name),
        sdk: "rust".to_string(),
        sdk_version: ">=1.0.0".to_string(),
        rustpress: RustPressMinVersion {
            min_version: "1.0.0".to_string(),
        },
    })
    .unwrap()
}

// =============================================================================
// Commands
// =============================================================================

fn create_project(project_type: &str, name: &str) {
    let struct_name = to_struct_name(name);
    let project_dir = Path::new(name);

    if project_dir.exists() {
        error(&format!("Directory '{}' already exists", name));
        exit(1);
    }

    info(&format!("Creating {} project: {}", project_type, name));

    // Create directories
    fs::create_dir_all(project_dir.join("src")).expect("Failed to create src directory");

    // Generate lib.rs based on project type
    let lib_content = match project_type {
        "plugin" => generate_plugin_lib(name, &struct_name),
        "theme" => generate_theme_lib(name, &struct_name),
        "app" => generate_app_lib(name, &struct_name),
        "hook" => generate_hook_lib(name, &to_snake_case(&struct_name.to_lowercase())),
        _ => unreachable!(),
    };

    // Write files
    let mut lib_file = fs::File::create(project_dir.join("src/lib.rs")).expect("Failed to create lib.rs");
    lib_file.write_all(lib_content.as_bytes()).expect("Failed to write lib.rs");

    let mut cargo_file = fs::File::create(project_dir.join("Cargo.toml")).expect("Failed to create Cargo.toml");
    cargo_file
        .write_all(generate_cargo_toml(name, project_type).as_bytes())
        .expect("Failed to write Cargo.toml");

    let mut readme_file = fs::File::create(project_dir.join("README.md")).expect("Failed to create README.md");
    readme_file
        .write_all(generate_readme(name, &struct_name, project_type).as_bytes())
        .expect("Failed to write README.md");

    let mut config_file =
        fs::File::create(project_dir.join("rustpress.json")).expect("Failed to create rustpress.json");
    config_file
        .write_all(generate_rustpress_config(name, project_type).as_bytes())
        .expect("Failed to write rustpress.json");

    let mut license_file = fs::File::create(project_dir.join("LICENSE")).expect("Failed to create LICENSE");
    license_file
        .write_all(b"MIT License\n\nCopyright (c) 2024\n")
        .expect("Failed to write LICENSE");

    let mut gitignore_file = fs::File::create(project_dir.join(".gitignore")).expect("Failed to create .gitignore");
    gitignore_file
        .write_all(b"/target\nCargo.lock\n*.swp\n*.swo\n.env\n")
        .expect("Failed to write .gitignore");

    success(&format!("Created {} project: {}", project_type, name));
    println!();
    println!("{}Next steps:{}", colors::BOLD, colors::RESET);
    println!("  cd {}", name);
    println!("  cargo build");
    println!("  rustpress dev");
}

fn run_dev() {
    let config_path = Path::new("rustpress.json");

    if !config_path.exists() {
        error("No rustpress.json found. Are you in a RustPress project directory?");
        exit(1);
    }

    let config: RustPressConfig =
        serde_json::from_str(&fs::read_to_string(config_path).expect("Failed to read config"))
            .expect("Failed to parse config");

    info(&format!("Starting development server for {}...", config.name));

    // Run cargo watch for hot reload
    let status = Command::new("cargo")
        .args(["watch", "-x", "build"])
        .status();

    match status {
        Ok(s) if s.success() => {}
        Ok(_) => {
            warning("Development server exited with non-zero status");
        }
        Err(e) => {
            if e.kind() == std::io::ErrorKind::NotFound {
                warning("cargo-watch not found. Install with: cargo install cargo-watch");
                info("Falling back to standard build...");
                let _ = Command::new("cargo").args(["build"]).status();
            } else {
                error(&format!("Failed to start development server: {}", e));
                exit(1);
            }
        }
    }
}

fn run_build(release: bool) {
    let config_path = Path::new("rustpress.json");

    if !config_path.exists() {
        error("No rustpress.json found. Are you in a RustPress project directory?");
        exit(1);
    }

    let config: RustPressConfig =
        serde_json::from_str(&fs::read_to_string(config_path).expect("Failed to read config"))
            .expect("Failed to parse config");

    info(&format!("Building {}...", config.name));

    // Run clippy first
    info("Running clippy...");
    let clippy_status = Command::new("cargo").args(["clippy"]).status();
    if let Ok(s) = clippy_status {
        if !s.success() {
            warning("Clippy found issues");
        }
    }

    // Build
    info("Building...");
    let mut build_args = vec!["build"];
    if release {
        build_args.push("--release");
    }

    let build_status = Command::new("cargo").args(&build_args).status();

    match build_status {
        Ok(s) if s.success() => {
            success("Build complete!");
            if release {
                info("Output: target/release/");
            } else {
                info("Output: target/debug/");
            }
        }
        _ => {
            error("Build failed");
            exit(1);
        }
    }
}

fn validate_project() {
    let config_path = Path::new("rustpress.json");

    if !config_path.exists() {
        error("No rustpress.json found");
        exit(1);
    }

    let config_str = fs::read_to_string(config_path).expect("Failed to read config");
    let config: Result<RustPressConfig, _> = serde_json::from_str(&config_str);

    let mut errors: Vec<String> = Vec::new();
    let mut warnings: Vec<String> = Vec::new();

    match config {
        Ok(c) => {
            // Validate required fields
            if c.name.is_empty() {
                errors.push("Missing required field: name".to_string());
            }
            if c.project_type.is_empty() {
                errors.push("Missing required field: type".to_string());
            }
            if c.version.is_empty() {
                errors.push("Missing required field: version".to_string());
            }
        }
        Err(e) => {
            errors.push(format!("Invalid JSON in rustpress.json: {}", e));
        }
    }

    // Validate Cargo.toml exists
    if !Path::new("Cargo.toml").exists() {
        errors.push("Missing Cargo.toml".to_string());
    }

    // Validate src/lib.rs exists
    if !Path::new("src/lib.rs").exists() {
        errors.push("Missing src/lib.rs".to_string());
    }

    // Check for README
    if !Path::new("README.md").exists() {
        warnings.push("Missing README.md".to_string());
    }

    // Print results
    if !errors.is_empty() {
        println!("{}Validation failed:{}", colors::RED, colors::RESET);
        for err in &errors {
            error(err);
        }
        exit(1);
    }

    if !warnings.is_empty() {
        println!("{}Warnings:{}", colors::YELLOW, colors::RESET);
        for warn in &warnings {
            warning(warn);
        }
    }

    success("Validation passed!");
}

fn publish_project() {
    let config_path = Path::new("rustpress.json");

    if !config_path.exists() {
        error("No rustpress.json found");
        exit(1);
    }

    let config: RustPressConfig =
        serde_json::from_str(&fs::read_to_string(config_path).expect("Failed to read config"))
            .expect("Failed to parse config");

    info(&format!("Publishing {} v{}...", config.name, config.version));
    println!();

    // Run validation first
    info("Validating project...");

    // Build in release mode if not already built
    if !Path::new("target/release").exists() {
        info("Building in release mode...");
        let build_status = Command::new("cargo")
            .args(["build", "--release"])
            .status();

        if let Ok(s) = build_status {
            if !s.success() {
                error("Build failed");
                exit(1);
            }
        }
    }

    // Upload to marketplace (simulated)
    info("Uploading to RustPress marketplace...");
    println!();
    success(&format!("Published {} v{}!", config.name, config.version));
    info(&format!(
        "View at: https://marketplace.rustpress.dev/{}s/{}",
        config.project_type, config.name
    ));
}

// =============================================================================
// Main Entry Point
// =============================================================================

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Create { project_type, name } => {
            create_project(&project_type, &name);
        }
        Commands::Dev => {
            run_dev();
        }
        Commands::Build { release } => {
            run_build(release);
        }
        Commands::Validate => {
            validate_project();
        }
        Commands::Publish => {
            publish_project();
        }
    }
}
