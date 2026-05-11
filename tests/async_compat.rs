//! Async / tokio compatibility tests.
//!
//! The crate intentionally exposes a synchronous API: parsing, validating,
//! and generating XARF reports is pure CPU work over in-memory data (no
//! network, no filesystem at runtime — the schemas are embedded with
//! `include_str!`). That means there is no async API to design around; the
//! existing entry points work fine from any tokio task as long as the public
//! types cross task boundaries cleanly.
//!
//! These tests prove two things:
//!
//! 1. **Static thread-safety guarantees.** All public types (`Report`,
//!    `Contact`, `Evidence`, `ParseResult`, error variants) implement `Send`
//!    and `Sync`, so callers can pass them across tokio tasks, channels,
//!    and `tokio::spawn` boundaries without wrapping.
//! 2. **The schema registry is shared safely.** A single
//!    `once_cell::sync::Lazy<SchemaRegistry>` underpins every call, so
//!    concurrent parse/validate calls from multiple tasks must not panic or
//!    race.
//!
//! If you do want to push parsing off the runtime thread for very large
//! batches, use `tokio::task::spawn_blocking` exactly like you would for any
//! other CPU-bound work — see [`runs_inside_spawn_blocking_for_large_batches`].

use serde_json::json;
use static_assertions::assert_impl_all;
use xarf::{
    parse, validate, Category, Contact, Evidence, ParseOptions, ParseResult, Report,
    ReportBuilder, ValidateOptions, ValidationError, ValidationInfo, ValidationResult,
    ValidationWarning, XarfError,
};

// ---------------------------------------------------------------------------
// 1. Compile-time guarantees
// ---------------------------------------------------------------------------

assert_impl_all!(Report: Send, Sync, Unpin);
assert_impl_all!(Contact: Send, Sync, Unpin);
assert_impl_all!(Evidence: Send, Sync, Unpin);
assert_impl_all!(Category: Send, Sync, Unpin);
assert_impl_all!(ParseResult: Send, Sync);
assert_impl_all!(ValidationResult: Send, Sync);
assert_impl_all!(ValidationError: Send, Sync);
assert_impl_all!(ValidationWarning: Send, Sync);
assert_impl_all!(ValidationInfo: Send, Sync);
assert_impl_all!(XarfError: Send, Sync);
assert_impl_all!(ReportBuilder: Send, Sync);
assert_impl_all!(ParseOptions: Send, Sync, Copy);
assert_impl_all!(ValidateOptions: Send, Sync, Copy);

// ---------------------------------------------------------------------------
// 2. Sync-callable from a single-threaded tokio runtime
// ---------------------------------------------------------------------------

fn sample_messaging_spam_json() -> String {
    json!({
        "xarf_version": "4.2.0",
        "report_id": "550e8400-e29b-41d4-a716-446655440000",
        "timestamp": "2024-01-15T14:30:25Z",
        "reporter": {"org": "X", "contact": "x@x.example", "domain": "x.example"},
        "sender": {"org": "X", "contact": "x@x.example", "domain": "x.example"},
        "source_identifier": "192.0.2.1",
        "source_port": 25,
        "category": "messaging",
        "type": "spam",
        "protocol": "smtp",
        "smtp_from": "spam@bad.example",
    })
    .to_string()
}

#[tokio::test(flavor = "current_thread")]
async fn parse_callable_from_single_thread_runtime() {
    let json = sample_messaging_spam_json();
    let result = parse(&json).unwrap();
    assert!(result.errors.is_empty());
    assert!(result.report.is_some());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn parse_callable_from_multi_thread_runtime() {
    let json = sample_messaging_spam_json();
    let result = parse(&json).unwrap();
    assert!(result.errors.is_empty());
}

// ---------------------------------------------------------------------------
// 3. Crossing task boundaries: report flows through spawn → join
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn typed_report_crosses_spawn_boundary() {
    let json = sample_messaging_spam_json();
    let handle = tokio::spawn(async move {
        let result = parse(&json).unwrap();
        result.report.unwrap()
    });
    let report: Report = handle.await.unwrap();
    assert_eq!(report.category.as_str(), "messaging");
    assert_eq!(report.type_, "spam");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn errors_cross_spawn_boundary() {
    let handle = tokio::spawn(async {
        let bad = json!({"xarf_version": "4.2.0"}).to_string();
        parse(&bad).unwrap().errors
    });
    let errors = handle.await.unwrap();
    assert!(!errors.is_empty());
}

// ---------------------------------------------------------------------------
// 4. Concurrent calls from many tasks share the schema registry safely
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 8)]
async fn many_concurrent_parses_share_the_schema_registry() {
    let mut joins = Vec::new();
    for _ in 0..64 {
        let json = sample_messaging_spam_json();
        joins.push(tokio::spawn(async move {
            let result = parse(&json).unwrap();
            assert!(result.errors.is_empty());
            result.report.is_some()
        }));
    }
    let mut ok = 0;
    for j in joins {
        if j.await.unwrap() {
            ok += 1;
        }
    }
    assert_eq!(ok, 64);
}

// ---------------------------------------------------------------------------
// 5. spawn_blocking for very large batches — the idiomatic pattern
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn runs_inside_spawn_blocking_for_large_batches() {
    // Simulate a "very large batch" of reports being processed off the
    // runtime. In a real consumer this is what you'd do to keep the async
    // runtime responsive even if the batch is huge.
    let payloads: Vec<String> = (0..200).map(|_| sample_messaging_spam_json()).collect();
    let processed = tokio::task::spawn_blocking(move || {
        let mut valid = 0usize;
        for p in &payloads {
            let r = parse(p).unwrap();
            if r.errors.is_empty() {
                valid += 1;
            }
        }
        valid
    })
    .await
    .unwrap();
    assert_eq!(processed, 200);
}

// ---------------------------------------------------------------------------
// 6. Builder used inside an async function compiles and runs
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "current_thread")]
async fn report_builder_works_from_async_function() {
    let result = build_report_async().await;
    assert!(result.errors.is_empty(), "errors: {:?}", result.errors);
}

async fn build_report_async() -> xarf::ParseResult {
    ReportBuilder::new("messaging", "spam", "192.0.2.1")
        .reporter(Contact::new("Acme", "abuse@acme.example", "acme.example"))
        .sender(Contact::new("Acme", "abuse@acme.example", "acme.example"))
        .source_port(25)
        .timestamp("2024-01-15T14:30:25Z")
        .report_id("550e8400-e29b-41d4-a716-446655440000")
        .extra("protocol", json!("smtp"))
        .extra("smtp_from", json!("spam@bad.example"))
        .build()
        .unwrap()
}

// ---------------------------------------------------------------------------
// 7. validate() callable from async context (no hidden blocking I/O)
// ---------------------------------------------------------------------------

#[tokio::test(flavor = "current_thread")]
async fn validate_callable_from_async_context() {
    let value: serde_json::Value = serde_json::from_str(&sample_messaging_spam_json()).unwrap();
    let result = validate(&value, ValidateOptions::default()).unwrap();
    assert!(result.valid);
}

// ---------------------------------------------------------------------------
// 8. The synchronous API does not need a runtime
// ---------------------------------------------------------------------------

/// Plain non-async function — proves no runtime is required for normal use.
/// Doubles as a callsite proof that callers porting from `xarf-python`
/// (synchronous) need no scaffolding to use this crate.
#[test]
fn sync_api_does_not_require_a_runtime() {
    let result = parse(&sample_messaging_spam_json()).unwrap();
    assert!(result.errors.is_empty());
}
