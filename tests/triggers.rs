//! Integration tests for trigger parsing utilities.
//!
//! Covers `parse_trigger`, `build_trigger`, and `is_valid_trigger`
//! including round-trip semantics and rejection of malformed input.

use rustpress_sdk::{build_trigger, is_valid_trigger, parse_trigger, TriggerParts};

#[test]
fn parses_basic_trigger() {
    let parts = parse_trigger("@@rustpress.ecommerce.Order.create@@").unwrap();
    assert_eq!(parts.plugin, "ecommerce");
    assert_eq!(parts.class_name, "Order");
    assert_eq!(parts.method, "create");
}

#[test]
fn parses_trigger_with_underscores() {
    let parts = parse_trigger("@@rustpress.user_mgmt.User_Account.create_user@@").unwrap();
    assert_eq!(parts.plugin, "user_mgmt");
    assert_eq!(parts.class_name, "User_Account");
    assert_eq!(parts.method, "create_user");
}

#[test]
fn parses_trigger_with_hyphens_in_plugin() {
    // The regex allows hyphens in the plugin segment only.
    let parts = parse_trigger("@@rustpress.my-plugin.Order.create@@").unwrap();
    assert_eq!(parts.plugin, "my-plugin");
}

#[test]
fn parses_trigger_with_numbers() {
    let parts = parse_trigger("@@rustpress.v2plugin.Order123.method2@@").unwrap();
    assert_eq!(parts.plugin, "v2plugin");
    assert_eq!(parts.class_name, "Order123");
    assert_eq!(parts.method, "method2");
}

#[test]
fn returns_none_for_missing_prefix() {
    assert!(parse_trigger("rustpress.ecommerce.Order.create").is_none());
}

#[test]
fn returns_none_for_missing_suffix() {
    assert!(parse_trigger("@@rustpress.ecommerce.Order.create").is_none());
}

#[test]
fn returns_none_for_wrong_namespace() {
    assert!(parse_trigger("@@laravel.ecommerce.Order.create@@").is_none());
}

#[test]
fn returns_none_for_too_few_segments() {
    assert!(parse_trigger("@@rustpress.ecommerce.Order@@").is_none());
}

#[test]
fn returns_none_for_too_many_segments() {
    assert!(parse_trigger("@@rustpress.eco.Order.create.extra@@").is_none());
}

#[test]
fn returns_none_for_empty_string() {
    assert!(parse_trigger("").is_none());
}

#[test]
fn returns_none_for_random_text() {
    assert!(parse_trigger("not a trigger at all").is_none());
}

#[test]
fn returns_none_for_special_chars_in_class() {
    // Hyphens are not allowed in the class segment.
    assert!(parse_trigger("@@rustpress.eco.Order-Item.create@@").is_none());
}

#[test]
fn returns_none_for_special_chars_in_method() {
    assert!(parse_trigger("@@rustpress.eco.Order.create-it@@").is_none());
}

#[test]
fn builds_trigger_format() {
    let trigger = build_trigger("ecommerce", "Order", "create");
    assert_eq!(trigger, "@@rustpress.ecommerce.Order.create@@");
}

#[test]
fn builds_trigger_with_underscores() {
    let trigger = build_trigger("user_mgmt", "User_Account", "create_user");
    assert_eq!(trigger, "@@rustpress.user_mgmt.User_Account.create_user@@");
}

#[test]
fn build_then_parse_roundtrip() {
    let trigger = build_trigger("blog", "Post", "publish");
    let parts = parse_trigger(&trigger).expect("round-trip must parse");
    assert_eq!(parts.plugin, "blog");
    assert_eq!(parts.class_name, "Post");
    assert_eq!(parts.method, "publish");
}

#[test]
fn parse_then_build_roundtrip() {
    let original = "@@rustpress.notifications.Email.send@@";
    let parts = parse_trigger(original).unwrap();
    let rebuilt = build_trigger(&parts.plugin, &parts.class_name, &parts.method);
    assert_eq!(original, rebuilt);
}

#[test]
fn validates_valid_triggers() {
    assert!(is_valid_trigger("@@rustpress.eco.Order.create@@"));
    assert!(is_valid_trigger("@@rustpress.a.B.c@@"));
    assert!(is_valid_trigger(
        "@@rustpress.long_plugin_name.SomeClass.someMethod@@"
    ));
}

#[test]
fn rejects_invalid_triggers() {
    assert!(!is_valid_trigger(""));
    assert!(!is_valid_trigger("invalid"));
    assert!(!is_valid_trigger("@@rustpress@@"));
    assert!(!is_valid_trigger("@@rustpress.foo@@"));
    assert!(!is_valid_trigger("@@rustpress.foo.bar@@"));
    assert!(!is_valid_trigger("@@rustpress..Order.create@@"));
}

#[test]
fn validates_consistent_with_parser() {
    // If `is_valid_trigger` says yes, `parse_trigger` must succeed; if it says no, parsing must fail.
    let cases = [
        ("@@rustpress.a.B.c@@", true),
        ("@@rustpress.foo.Bar.baz@@", true),
        ("rustpress.foo.Bar.baz", false),
        ("@@rustpress.foo.Bar@@", false),
        ("", false),
    ];
    for (input, expected_valid) in cases {
        assert_eq!(is_valid_trigger(input), expected_valid, "input={}", input);
        assert_eq!(parse_trigger(input).is_some(), expected_valid, "input={}", input);
    }
}

#[test]
fn trigger_parts_equality() {
    let a = TriggerParts {
        plugin: "eco".into(),
        class_name: "Order".into(),
        method: "create".into(),
    };
    let b = parse_trigger("@@rustpress.eco.Order.create@@").unwrap();
    assert_eq!(a, b);
}
