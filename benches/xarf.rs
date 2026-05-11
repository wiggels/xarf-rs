//! Criterion benchmarks for the `xarf` crate.
//!
//! The intent is regression detection — not micro-optimisation. Each
//! benchmark exercises a clearly-named user-visible operation and produces
//! stable timings so a CI run can diff against a saved baseline.
//!
//! ## Suggested workflow
//!
//! ```sh
//! # Save the current state as the reference.
//! cargo bench --bench xarf -- --save-baseline main
//!
//! # ... after making changes, compare:
//! cargo bench --bench xarf -- --baseline main
//! ```
//!
//! Criterion writes baselines to `target/criterion/`. A regression of >5%
//! triggers a red "regressed" annotation in the report. Throughput is
//! reported in MB/s for parse benchmarks via [`Throughput::Bytes`] so
//! regressions are interpretable in absolute terms.

use std::hint::black_box;
use std::time::Duration;

use criterion::{BatchSize, BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use serde_json::{Value, json};
use xarf::{
    Contact, EvidenceOptions, HashAlgorithm, ParseOptions, ReportBuilder, ValidateOptions,
    convert_v3_to_v4, create_evidence_with_options, is_v3_report, parse, parse_value,
    parse_with_options, validate,
};

// ---------------------------------------------------------------------------
// Fixtures
// ---------------------------------------------------------------------------

/// Small messaging/spam payload (~500 bytes). Models the common case: a
/// freshly-arrived spam complaint with one evidence item.
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
        "evidence": [{
            "content_type": "text/plain",
            "payload": "aGVsbG8gd29ybGQ=",
            "description": "Snippet",
            "size": 11,
        }],
        "tags": ["scam:advance_fee", "severity:medium"],
        "confidence": 0.95,
    })
    .to_string()
}

/// Connection/DDoS payload (~700 bytes). Different category exercises a
/// different type-schema branch in the master schema.
fn ddos_json() -> String {
    json!({
        "xarf_version": "4.2.0",
        "report_id": "00000000-0000-4000-8000-000000000000",
        "timestamp": "2024-01-15T14:30:25Z",
        "reporter": {"org": "CERT", "contact": "cert@cert.example", "domain": "cert.example"},
        "sender": {"org": "CERT", "contact": "cert@cert.example", "domain": "cert.example"},
        "source_identifier": "203.0.113.5",
        "source_port": 53,
        "category": "connection",
        "type": "ddos",
        "first_seen": "2024-01-15T14:15:00Z",
        "protocol": "udp",
        "destination_ip": "192.0.2.42",
        "destination_port": 80,
        "attack_vector": "dns_amplification",
        "peak_bps": 50_000_000,
        "peak_pps": 100_000,
        "evidence_source": "flow_analysis",
        "confidence": 0.99,
    })
    .to_string()
}

/// Large messaging/spam payload (~50KB) with many evidence items + tags.
/// Stresses the JSON parser and the schema validator's array iteration.
fn large_spam_json() -> String {
    let large_payload: String = "A".repeat(4_096); // 3KB base64-decoded ~ 3KB
    let evidence: Vec<Value> = (0..10)
        .map(|i| {
            json!({
                "content_type": "text/plain",
                "payload": large_payload,
                "description": format!("evidence chunk {i}"),
                "size": large_payload.len(),
            })
        })
        .collect();
    let tags: Vec<String> = (0..20).map(|i| format!("tag{i}:value{i}")).collect();

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
        "tags": tags,
        "confidence": 0.95,
        "description": "Large bulk spam report",
    })
    .to_string()
}

/// v3 spam sample used by the conversion benchmarks.
fn v3_spam_json() -> String {
    json!({
        "Version": "3.0.0",
        "ReporterInfo": {
            "ReporterOrg": "Anti-Spam",
            "ReporterOrgDomain": "antispam.example",
            "ReporterContactEmail": "abuse@antispam.example",
        },
        "Report": {
            "ReportClass": "Messaging",
            "ReportType": "Spam",
            "Date": "2024-01-15T14:30:25Z",
            "Source": {"IP": "192.0.2.1", "Port": 25, "Type": "ip"},
            "Attachment": [{
                "ContentType": "message/rfc822",
                "Description": "Spam email",
                "Data": "RnJvbTogc3BhbUBleGFtcGxlLmNvbQ==",
            }],
            "AdditionalInfo": {
                "Protocol": "smtp",
                "SMTPFrom": "spam@example.com",
                "Subject": "buy our stuff",
                "DetectionMethod": "spamtrap",
            },
        },
    })
    .to_string()
}

// ---------------------------------------------------------------------------
// Parse benchmarks
// ---------------------------------------------------------------------------

fn bench_parse(c: &mut Criterion) {
    let mut group = c.benchmark_group("parse");
    // Stabilise small/fast benchmarks: run more samples, shorter warmup.
    group.measurement_time(Duration::from_secs(5));

    let small = small_spam_json();
    group.throughput(Throughput::Bytes(small.len() as u64));
    group.bench_function("small_spam_str", |b| {
        b.iter(|| {
            let r = parse(black_box(&small)).unwrap();
            black_box(r);
        });
    });

    let ddos = ddos_json();
    group.throughput(Throughput::Bytes(ddos.len() as u64));
    group.bench_function("ddos_str", |b| {
        b.iter(|| {
            let r = parse(black_box(&ddos)).unwrap();
            black_box(r);
        });
    });

    let large = large_spam_json();
    group.throughput(Throughput::Bytes(large.len() as u64));
    group.bench_function("large_spam_str", |b| {
        b.iter(|| {
            let r = parse(black_box(&large)).unwrap();
            black_box(r);
        });
    });

    // parse_value avoids the JSON-decode step, isolating validation +
    // typed-deserialisation cost.
    let small_value: Value = serde_json::from_str(&small).unwrap();
    group.throughput(Throughput::Bytes(small.len() as u64));
    group.bench_function("small_spam_prevalued", |b| {
        b.iter_batched(
            || small_value.clone(),
            |v| {
                let r = parse_value(v, ParseOptions::default()).unwrap();
                black_box(r);
            },
            BatchSize::SmallInput,
        );
    });

    // Strict mode forces a different (recommended → required) schema variant.
    group.throughput(Throughput::Bytes(small.len() as u64));
    group.bench_function("small_spam_strict", |b| {
        b.iter(|| {
            let r = parse_with_options(
                black_box(&small),
                ParseOptions {
                    strict: true,
                    show_missing_optional: false,
                },
            )
            .unwrap();
            black_box(r);
        });
    });

    // show_missing_optional adds the missing-field discovery pass.
    group.throughput(Throughput::Bytes(small.len() as u64));
    group.bench_function("small_spam_show_missing", |b| {
        b.iter(|| {
            let r = parse_with_options(
                black_box(&small),
                ParseOptions {
                    strict: false,
                    show_missing_optional: true,
                },
            )
            .unwrap();
            black_box(r);
        });
    });

    group.finish();
}

// ---------------------------------------------------------------------------
// Validate-only benchmarks (no typed deserialisation)
// ---------------------------------------------------------------------------

fn bench_validate(c: &mut Criterion) {
    let mut group = c.benchmark_group("validate");

    let small_value: Value = serde_json::from_str(&small_spam_json()).unwrap();
    group.bench_function("small_spam_normal", |b| {
        b.iter(|| {
            let r = validate(black_box(&small_value), ValidateOptions::default()).unwrap();
            black_box(r);
        });
    });
    group.bench_function("small_spam_strict", |b| {
        b.iter(|| {
            let r = validate(
                black_box(&small_value),
                ValidateOptions {
                    strict: true,
                    show_missing_optional: false,
                },
            )
            .unwrap();
            black_box(r);
        });
    });

    let ddos_value: Value = serde_json::from_str(&ddos_json()).unwrap();
    group.bench_function("ddos_normal", |b| {
        b.iter(|| {
            let r = validate(black_box(&ddos_value), ValidateOptions::default()).unwrap();
            black_box(r);
        });
    });

    group.finish();
}

// ---------------------------------------------------------------------------
// Generator benchmarks
// ---------------------------------------------------------------------------

fn bench_generate(c: &mut Criterion) {
    let mut group = c.benchmark_group("generate");

    group.bench_function("minimal_spam_build", |b| {
        b.iter(|| {
            let r = ReportBuilder::new(
                black_box("messaging"),
                black_box("spam"),
                black_box("192.0.2.1"),
            )
            .reporter(Contact::new("Acme", "abuse@acme.example", "acme.example"))
            .sender(Contact::new("Acme", "abuse@acme.example", "acme.example"))
            .timestamp("2024-01-15T14:30:25Z")
            .report_id("550e8400-e29b-41d4-a716-446655440000")
            .source_port(25)
            .extra("protocol", json!("smtp"))
            .extra("smtp_from", json!("spam@bad.example"))
            .build()
            .unwrap();
            black_box(r);
        });
    });

    group.bench_function("spam_build_with_evidence", |b| {
        let payload = b"hello, xarf!".to_vec();
        b.iter(|| {
            let ev = create_evidence_with_options(
                "text/plain",
                &payload,
                EvidenceOptions {
                    description: Some("snippet".into()),
                    hash_algorithm: HashAlgorithm::Sha256,
                },
            );
            let r = ReportBuilder::new("messaging", "spam", "192.0.2.1")
                .reporter(Contact::new("Acme", "abuse@acme.example", "acme.example"))
                .sender(Contact::new("Acme", "abuse@acme.example", "acme.example"))
                .timestamp("2024-01-15T14:30:25Z")
                .report_id("550e8400-e29b-41d4-a716-446655440000")
                .source_port(25)
                .extra("protocol", json!("smtp"))
                .extra("smtp_from", json!("spam@bad.example"))
                .add_evidence(ev)
                .build()
                .unwrap();
            black_box(r);
        });
    });

    group.finish();
}

// ---------------------------------------------------------------------------
// create_evidence benchmarks — per algorithm, per payload size
// ---------------------------------------------------------------------------

fn bench_create_evidence(c: &mut Criterion) {
    let mut group = c.benchmark_group("create_evidence");

    let sizes = [256usize, 4_096, 65_536, 1_048_576];
    for size in sizes {
        let payload: Vec<u8> = (0..size).map(|i| (i % 251) as u8).collect();
        group.throughput(Throughput::Bytes(size as u64));

        for algo in [
            HashAlgorithm::Sha256,
            HashAlgorithm::Sha512,
            HashAlgorithm::Sha1,
            HashAlgorithm::Md5,
        ] {
            group.bench_with_input(
                BenchmarkId::new(algo.prefix(), size),
                &payload,
                |b, payload| {
                    b.iter(|| {
                        let ev = create_evidence_with_options(
                            black_box("application/octet-stream"),
                            black_box(payload),
                            EvidenceOptions {
                                description: None,
                                hash_algorithm: algo,
                            },
                        );
                        black_box(ev);
                    });
                },
            );
        }
    }

    group.finish();
}

// ---------------------------------------------------------------------------
// v3 conversion benchmarks
// ---------------------------------------------------------------------------

fn bench_v3(c: &mut Criterion) {
    let mut group = c.benchmark_group("v3");

    let raw = v3_spam_json();
    let value: Value = serde_json::from_str(&raw).unwrap();
    group.throughput(Throughput::Bytes(raw.len() as u64));

    group.bench_function("is_v3_report", |b| {
        b.iter(|| {
            let is = is_v3_report(black_box(&value));
            black_box(is);
        });
    });

    group.bench_function("convert_v3_to_v4", |b| {
        b.iter_batched(
            || (value.clone(), Vec::<String>::new()),
            |(v, mut warnings)| {
                let r = convert_v3_to_v4(v, &mut warnings).unwrap();
                black_box(r);
            },
            BatchSize::SmallInput,
        );
    });

    // Full parse() path on a v3 sample exercises detection + conversion +
    // validation + typed deserialisation back-to-back.
    group.throughput(Throughput::Bytes(raw.len() as u64));
    group.bench_function("parse_v3_full_pipeline", |b| {
        b.iter(|| {
            let r = parse(black_box(&raw)).unwrap();
            black_box(r);
        });
    });

    group.finish();
}

// ---------------------------------------------------------------------------
// Schema registry benchmarks
// ---------------------------------------------------------------------------

fn bench_registry(c: &mut Criterion) {
    let mut group = c.benchmark_group("registry");

    group.bench_function("known_combination_hit", |b| {
        b.iter(|| {
            let hit = xarf::schemas::registry()
                .is_known_combination(black_box("messaging"), black_box("spam"));
            black_box(hit);
        });
    });

    group.bench_function("known_combination_miss", |b| {
        b.iter(|| {
            let hit = xarf::schemas::registry()
                .is_known_combination(black_box("messaging"), black_box("not_a_type"));
            black_box(hit);
        });
    });

    // Compiling a fresh master validator on every call models a worst-case
    // scenario for callers that don't cache. Should remain bounded.
    group.bench_function("compile_master_validator", |b| {
        b.iter(|| {
            let v = xarf::schemas::registry().master_validator(false).unwrap();
            black_box(v);
        });
    });

    group.finish();
}

// ---------------------------------------------------------------------------
// Round-trip benchmarks
// ---------------------------------------------------------------------------

fn bench_round_trip(c: &mut Criterion) {
    let mut group = c.benchmark_group("round_trip");

    let small = small_spam_json();
    group.throughput(Throughput::Bytes(small.len() as u64));
    group.bench_function("parse_then_serialize_small", |b| {
        b.iter(|| {
            let report = parse(black_box(&small)).unwrap().report.unwrap();
            let out = serde_json::to_string(&report).unwrap();
            black_box(out);
        });
    });

    let large = large_spam_json();
    group.throughput(Throughput::Bytes(large.len() as u64));
    group.bench_function("parse_then_serialize_large", |b| {
        b.iter(|| {
            let report = parse(black_box(&large)).unwrap().report.unwrap();
            let out = serde_json::to_string(&report).unwrap();
            black_box(out);
        });
    });

    group.finish();
}

// ---------------------------------------------------------------------------
// Wire up
// ---------------------------------------------------------------------------

criterion_group!(
    benches,
    bench_parse,
    bench_validate,
    bench_generate,
    bench_create_evidence,
    bench_v3,
    bench_registry,
    bench_round_trip,
);
criterion_main!(benches);
