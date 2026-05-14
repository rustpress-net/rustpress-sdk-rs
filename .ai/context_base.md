# rustpress-sdk-rs — AI Context

> **Purpose**: Orient an AI agent to this repo without reading the whole tree. Pair with the RustPress organisation context in `rustpress-core-base/.ai/context/CONTEXT_BASE.md`.

## Project

`rustpress-sdk-rs` is the **official Rust SDK for RustPress**. It is the type-system-blessed surface that Rust plugin, theme, and app authors program against when extending RustPress (the CMS in `rustpress-core-base`). It's a thin, doc-heavy crate: traits, structs, enums, and a handful of utility fns — no heavy runtime. The runtime lives in `rustpress-core-base`'s `crates/rustpress-core`; this SDK is what you put in your `Cargo.toml` when you're building *on top of* RustPress, not modifying it.

API parity with `rustpress-sdk-ts`, `rustpress-sdk-py`, and `rustpress-sdk-js` is a hard requirement — the "build a RustPress app in your language of choice" narrative depends on it.

## Tech stack

- **Language**: Rust 2021, MSRV 1.70
- **Crate name**: `rustpress-sdk` (published to crates.io at v1.0.0)
- **Async runtime**: tokio 1.35
- **Key deps**: `async-trait`, `serde`/`serde_json`, `chrono`, `uuid`, `thiserror`, `tracing`, `regex`, optional `reqwest` (feature `http-client`, default-on)
- **Build**: `cargo build` / `cargo test` / `cargo doc`
- **Binary**: ships a `rustpress` CLI binary (`src/bin/rustpress.rs`) for scaffolding

## Directory layout

```
rustpress-sdk-rs/
├── Cargo.toml          # name = rustpress-sdk, v1.0.0, MIT
├── README.md           # cargo install + quick-start examples
├── LICENSE             # MIT
└── src/
    ├── lib.rs          # 1035 lines — the entire public API
    └── bin/
        └── rustpress.rs # CLI scaffolder
└── tests/              # currently empty in main (audit baseline) — being added
    ├── hooks.rs        # uncommitted test scaffold
    └── triggers.rs     # uncommitted test scaffold
```

## Public API / what this repo exposes

All exports live in `src/lib.rs`. The shape (from the audit):

- **Enums** (`src/lib.rs:84–163`): `TriggerTiming`, `NotificationType`, `NotificationChannel`, `ContentStatus`
- **Structs**: `User` (170), `Session` (190), `TriggerArgs` (208), `HookMetadata` (224), `PluginMetadata` (314), `ThemeMetadata` (372), `AppMetadata` (390), `RustPressContext`
- **Traits**: `Plugin` (async via `async_trait`), `Logger`, `Hook`
- **Functions**: `define_hook`, `before_hook`, `after_hook`, `parse_trigger`, `build_trigger`, `is_valid_trigger`, `generate_id`, `retry`, `sleep_ms`
- **Feature-gated**: `SimpleHttpClient` (behind `http-client`), `SimpleEventEmitter`

Trigger strings follow the `@@rustpress.<feature>.<resource>.<action>@@` convention shared across all four SDKs.

## How to build / test

```bash
cargo build                       # default features (http-client on)
cargo build --no-default-features # minimal, no reqwest
cargo test                        # currently only ~35 lines of inline tests (lib.rs:999–1035)
cargo doc --open                  # opens local rustdoc
cargo install --path .            # installs the rustpress CLI
```

CI is wired via `rustpress-net/rustpress-core-devops/actions/ci-rust@main` (run check + test + clippy + fmt).

## Cross-repo dependencies

- **Depends on**: nothing in the RustPress org — this crate is intentionally standalone so plugin authors don't need the core checked out to compile against the SDK.
- **Depended on by**: all out-of-tree Rust plugins, themes, and apps. In-tree plugins under `rustpress-core-base/plugins/*` use the lower-level `rustpress-core` workspace crate directly instead.

## Conventions

- **License**: MIT (single license — note: differs from core-base which is `MIT OR Apache-2.0`; align before v1.0 publish)
- **Commits**: Conventional Commits
- **Public API stability**: this crate is at v1.0 — every public item is a stability commitment. Breaking changes bump to v2.

## Status

- Release readiness: **🟡 ALMOST READY** (see `AUDIT-sdks.md`)
- Closest to publishable of the four SDKs (most tests, simplest layout)
- Cargo.toml stamped at `version = "1.0.0"` but not yet published to crates.io
- Phase: alpha hardening — needs the test backbone written before v1.0.0 publish

## Known issues / TODOs

From `AUDIT-sdks.md` (Rust SDK section):

- **P0**: Test coverage is ~35 lines of inline tests covering only trigger utilities. Need real coverage for hook execution, plugin lifecycle, `RustPressContext`, retry/sleep helpers. The uncommitted `tests/hooks.rs` and `tests/triggers.rs` files are the start of this work.
- **P0**: No `examples/` directory — crates.io trust signal. Add at least: minimal plugin, minimal hook, minimal theme metadata example.
- **P1**: Add CI badges to README.
- **P1**: Document error handling (`thiserror`-based) explicitly.
- **P1**: Performance benchmarks for hook dispatch.
- **P1**: Align license to `MIT OR Apache-2.0` to match the rest of the org.
- **P1**: Repo URL in Cargo.toml still references `github.com/rustpress/rustpress-sdk-rs` (should be `rustpress-net/`).

## When working in this repo

- Every public symbol must have a `///` doc comment with at least one runnable doctest where reasonable — this crate is the canonical example surface.
- Keep parity with `rustpress-sdk-ts`, `rustpress-sdk-py`, `rustpress-sdk-js`. Adding/removing/renaming a public symbol here means doing the same in the other three SDKs in a coordinated PR.
- Do not pull in heavy deps. The SDK should compile fast and add minimal weight to a downstream crate's build.
- `http-client` feature is optional but on by default — guard new HTTP-using code with `#[cfg(feature = "http-client")]`.
