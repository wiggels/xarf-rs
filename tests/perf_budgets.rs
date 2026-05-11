//! Wall-clock perf budgets, committed and shipped with the crate.
//!
//! Criterion baselines live in `target/criterion/` and are gitignored, so a
//! contributor running `cargo bench` only sees regressions against *their
//! own* prior runs. These tests are the portable backstop: they run as part
//! of `cargo test`, ship with the repo, and assert wall-clock budgets that
//! catch catastrophic regressions (typically 10×+ slowdowns) on any machine
//! without needing CI infrastructure.
//!
//! ## Design
//!
//! Each test does `N` iterations and divides total wall-clock time by `N`.
//! This averages out short-burst jitter (GC pauses, page faults, scheduler
//! preemption) better than measuring a single iteration. The budgets are
//! set ~**10–50× above** the observed timings on a 2024 commodity laptop,
//! so:
//!
//! * Slow CI runners and older hardware won't false-positive.
//! * A real algorithmic regression (e.g. the master validator being
//!   re-compiled on every `parse()` call — exactly the bug the benchmarks
//!   surfaced before caching landed) WILL trip them.
//!
//! Treat the budgets as a "did we just shoot ourselves in the foot?"
//! signal, not a precision instrument. For precise regression detection
//! across hardware, use the criterion benchmarks in `benches/xarf.rs` with
//! a CI workflow that saves baselines per-runner (see `.github/workflows/`).
//!
//! ## Running
//!
//! ```sh
//! # Just like any other test.
//! cargo test --test perf_budgets
//!
//! # Or run them in release mode for realistic numbers
//! # (debug-mode times are roughly 5–10× slower, but the budgets are set
//! # generous enough that they still pass).
//! cargo test --release --test perf_budgets
//! ```
//!
//! Each test prints its observed median to stderr so you can spot
//! "passes-but-suspiciously-slow" cases. To see those numbers without
//! capture suppression, run with `--nocapture`:
//!
//! ```sh
//! cargo test --release --test perf_budgets -- --nocapture
//! ```

use std::time::{Duration, Instant};

use serde_json::{Value, json};
use xarf::{
    Contact, EvidenceOptions, HashAlgorithm, ParseOptions, ReportBuilder, ValidateOptions,
    convert_v3_to_v4, create_evidence_with_options, is_v3_report, parse, validate,
};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Run `op` `iters` times, three independent batches, and return the **min**
/// per-call average. Min-of-batches is the standard way to defend against
/// scheduler/GC outliers: a single kernel preemption can inflate a batch
/// average by orders of magnitude when the per-call cost is in the
/// nanoseconds, but it almost never hits all three batches at once.
///
/// We also warm up first so lazy `SchemaRegistry` init and one-shot
/// validator compilation don't pollute the measurement.
fn measure<F: FnMut()>(iters: u32, mut op: F) -> Duration {
    op(); // warmup — also forces lazy registry init for tests that need it
    let mut best = Duration::MAX;
    for _ in 0..3 {
        let start = Instant::now();
        for _ in 0..iters {
            op();
        }
        let avg = start.elapsed() / iters;
        if avg < best {
            best = avg;
        }
    }
    best
}

/// Assert `actual <= budget`. Prints both numbers either way so passes are
/// visible too — "we're still 50× under budget" is useful information.
#[track_caller]
fn assert_under_budget(name: &str, actual: Duration, budget: Duration) {
    eprintln!(
        "[perf] {:<48} {:>8.2?} (budget {:>8.2?}, {:>5.1}× headroom)",
        name,
        actual,
        budget,
        budget.as_secs_f64() / actual.as_secs_f64().max(1e-12),
    );
    assert!(
        actual <= budget,
        "perf regression in `{name}`: {actual:.2?} exceeds budget {budget:.2?}"
    );
}

// ---------------------------------------------------------------------------
// Fixtures
// ---------------------------------------------------------------------------

fn small_spam_json() -> String {
    json!({
        "xarf_version": "4.2.0",
        "report_id": "550e8400-e29b-41d4-a716-446655440000",
        "timestamp": "2024-01-15T14:30:25Z",
        "reporter": {"org": "Acme", "contact": "abuse@acme.example", "domain": "acme.example"},
        "sender": {"org": "Acme", "contact": "abuse@acme.example", "domain": "acme.example"},
        "source_identifier": "192.0.2.1",
        "source_port": 25,
        "category": "messaging",
        "type": "spam",
        "protocol": "smtp",
        "smtp_from": "spam@bad.example",
        "subject": "Cheap meds!",
        "evidence_source": "spamtrap",
        "tags": ["scam:advance_fee"],
        "confidence": 0.95,
    })
    .to_string()
}

fn large_spam_json() -> String {
    let payload: String = "A".repeat(4_096);
    let evidence: Vec<Value> = (0..10)
        .map(|i| {
            json!({
                "content_type": "text/plain",
                "payload": payload,
                "description": format!("evidence chunk {i}"),
            })
        })
        .collect();
    json!({
        "xarf_version": "4.2.0",
        "report_id": "550e8400-e29b-41d4-a716-446655440000",
        "timestamp": "2024-01-15T14:30:25Z",
        "reporter": {"org": "Acme", "contact": "abuse@acme.example", "domain": "acme.example"},
        "sender": {"org": "Acme", "contact": "abuse@acme.example", "domain": "acme.example"},
        "source_identifier": "192.0.2.1",
        "source_port": 25,
        "category": "messaging",
        "type": "spam",
        "protocol": "smtp",
        "smtp_from": "spam@bad.example",
        "evidence_source": "spamtrap",
        "evidence": evidence,
    })
    .to_string()
}

fn v3_spam_json() -> String {
    json!({
        "Version": "3.0.0",
        "ReporterInfo": {
            "ReporterOrg": "Anti-Spam",
            "ReporterContactEmail": "abuse@antispam.example",
        },
        "Report": {
            "ReportType": "Spam",
            "Date": "2024-01-15T14:30:25Z",
            "Source": {"IP": "192.0.2.1", "Port": 25},
            "Attachment": [{
                "ContentType": "message/rfc822",
                "Description": "spam",
                "Data": "RnJvbTogc3BhbUBleGFtcGxlLmNvbQ==",
            }],
            "AdditionalInfo": {
                "Protocol": "smtp",
                "SMTPFrom": "spam@example.com",
                "DetectionMethod": "spamtrap",
            },
        },
    })
    .to_string()
}

// ---------------------------------------------------------------------------
// Parse budgets
// ---------------------------------------------------------------------------
//
// Observed on a 2024 Apple M-series laptop in release mode:
//
//   parse small_spam   ~9µs   → budget 1ms   (~110× headroom)
//   parse large_spam   ~23µs  → budget 5ms   (~217× headroom)
//   parse strict       ~10µs  → budget 1ms   (~100× headroom)
//
// Debug builds run ~5–10× slower, still comfortably under budget.

#[test]
fn parse_small_spam_under_1ms() {
    let json = small_spam_json();
    let avg = measure(500, || {
        let r = parse(&json).unwrap();
        std::hint::black_box(r);
    });
    assert_under_budget("parse small_spam", avg, Duration::from_millis(1));
}

#[test]
fn parse_large_spam_under_5ms() {
    let json = large_spam_json();
    let avg = measure(100, || {
        let r = parse(&json).unwrap();
        std::hint::black_box(r);
    });
    assert_under_budget("parse large_spam (40KB)", avg, Duration::from_millis(5));
}

#[test]
fn parse_strict_mode_under_1ms() {
    let json = small_spam_json();
    let avg = measure(500, || {
        let r = xarf::parse_with_options(
            &json,
            ParseOptions {
                strict: true,
                show_missing_optional: false,
            },
        )
        .unwrap();
        std::hint::black_box(r);
    });
    assert_under_budget("parse strict", avg, Duration::from_millis(1));
}

// ---------------------------------------------------------------------------
// Validate-only budget
// ---------------------------------------------------------------------------

#[test]
fn validate_small_spam_under_500us() {
    let value: Value = serde_json::from_str(&small_spam_json()).unwrap();
    let avg = measure(500, || {
        let r = validate(&value, ValidateOptions::default()).unwrap();
        std::hint::black_box(r);
    });
    assert_under_budget("validate small_spam", avg, Duration::from_micros(500));
}

// ---------------------------------------------------------------------------
// v3 conversion budgets
// ---------------------------------------------------------------------------

#[test]
fn is_v3_report_under_2us() {
    // Observed ~95ns in release and ~300ns in debug; the 2µs budget is
    // generous enough that scheduler jitter in single-digit-iteration batches
    // can't flake the test, but catches any genuine algorithmic regression
    // (e.g. accidentally re-parsing JSON inside the detector).
    let value: Value = serde_json::from_str(&v3_spam_json()).unwrap();
    let avg = measure(50_000, || {
        let is = is_v3_report(&value);
        std::hint::black_box(is);
    });
    assert_under_budget("is_v3_report detection", avg, Duration::from_micros(2));
}

#[test]
fn convert_v3_to_v4_under_500us() {
    let value: Value = serde_json::from_str(&v3_spam_json()).unwrap();
    let avg = measure(500, || {
        let mut warnings = Vec::new();
        let r = convert_v3_to_v4(value.clone(), &mut warnings).unwrap();
        std::hint::black_box(r);
    });
    assert_under_budget("convert_v3_to_v4", avg, Duration::from_micros(500));
}

#[test]
fn parse_v3_full_pipeline_under_5ms() {
    let json = v3_spam_json();
    let avg = measure(100, || {
        let r = parse(&json).unwrap();
        std::hint::black_box(r);
    });
    assert_under_budget("parse v3 → v4 full", avg, Duration::from_millis(5));
}

// ---------------------------------------------------------------------------
// Evidence-creation budget (hash + base64)
// ---------------------------------------------------------------------------

#[test]
fn create_evidence_sha256_64kib_under_5ms() {
    let payload: Vec<u8> = (0..65_536).map(|i| (i % 251) as u8).collect();
    let avg = measure(50, || {
        let ev = create_evidence_with_options(
            "application/octet-stream",
            &payload,
            EvidenceOptions {
                description: None,
                hash_algorithm: HashAlgorithm::Sha256,
            },
        );
        std::hint::black_box(ev);
    });
    assert_under_budget(
        "create_evidence sha256 64KiB",
        avg,
        Duration::from_millis(5),
    );
}

#[test]
fn create_evidence_md5_64kib_under_10ms() {
    let payload: Vec<u8> = (0..65_536).map(|i| (i % 251) as u8).collect();
    let avg = measure(50, || {
        let ev = create_evidence_with_options(
            "application/octet-stream",
            &payload,
            EvidenceOptions {
                description: None,
                hash_algorithm: HashAlgorithm::Md5,
            },
        );
        std::hint::black_box(ev);
    });
    assert_under_budget("create_evidence md5 64KiB", avg, Duration::from_millis(10));
}

// ---------------------------------------------------------------------------
// Builder budget
// ---------------------------------------------------------------------------

#[test]
fn builder_minimal_spam_under_1ms() {
    let avg = measure(500, || {
        let r = ReportBuilder::new("messaging", "spam", "192.0.2.1")
            .reporter(Contact::new("Acme", "abuse@acme.example", "acme.example"))
            .sender(Contact::new("Acme", "abuse@acme.example", "acme.example"))
            .timestamp("2024-01-15T14:30:25Z")
            .report_id("550e8400-e29b-41d4-a716-446655440000")
            .source_port(25)
            .extra("protocol", json!("smtp"))
            .extra("smtp_from", json!("spam@bad.example"))
            .build()
            .unwrap();
        std::hint::black_box(r);
    });
    assert_under_budget("builder minimal_spam", avg, Duration::from_millis(1));
}

// ---------------------------------------------------------------------------
// Registry lookup budget
// ---------------------------------------------------------------------------

#[test]
fn registry_known_combination_under_2us() {
    // Observed ~30ns in release; budget is 2µs so debug-mode and noisy
    // shared CI runners don't false-positive on a near-noise-floor metric.
    let avg = measure(100_000, || {
        let hit = xarf::schemas::registry().is_known_combination("messaging", "spam");
        std::hint::black_box(hit);
    });
    assert_under_budget(
        "registry is_known_combination hit",
        avg,
        Duration::from_micros(2),
    );
}

// ---------------------------------------------------------------------------
// Throughput budget — high-volume scenario
// ---------------------------------------------------------------------------

/// The XARF v4 spec's `xarf-parser-tests` perf section says a parser
/// should handle **at least 1000 reports per second** in single-threaded
/// processing. That translates to ≤1ms average per parse, which our other
/// budgets cover individually. This is the explicit end-to-end version of
/// the contract.
#[test]
fn meets_spec_throughput_target_of_1000_reports_per_second() {
    let json = small_spam_json();
    let iters = 1000u32;
    parse(&json).unwrap(); // warmup
    let start = Instant::now();
    for _ in 0..iters {
        let r = parse(&json).unwrap();
        std::hint::black_box(r);
    }
    let elapsed = start.elapsed();
    let rps = iters as f64 / elapsed.as_secs_f64();
    eprintln!("[perf] throughput: {rps:.0} reports/s ({elapsed:.2?} for {iters} reports)");
    assert!(
        rps >= 1_000.0,
        "spec target is ≥1000 reports/s, observed {rps:.0}/s"
    );
}
