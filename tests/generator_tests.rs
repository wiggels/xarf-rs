//! Unit tests for `xarf::generator` (ReportBuilder, create_evidence, etc.).

use base64::Engine;
use base64::engine::general_purpose::STANDARD as BASE64;
use serde_json::{Map, json};
use xarf::{
    Contact, CreateReportOptions, EvidenceOptions, HashAlgorithm, ReportBuilder, SPEC_VERSION,
    create_evidence, create_evidence_with_options, create_report,
};

fn reporter() -> Contact {
    Contact::new("Acme", "abuse@acme.example", "acme.example")
}

#[test]
fn report_builder_auto_fills_metadata() {
    let result = ReportBuilder::new("messaging", "spam", "192.0.2.1")
        .reporter(reporter())
        .sender(reporter())
        .source_port(25)
        .extra("protocol", json!("smtp"))
        .extra("smtp_from", json!("spam@bad.example"))
        .build()
        .expect("ok");
    assert!(result.errors.is_empty(), "errors: {:?}", result.errors);
    let report = result.report.expect("typed report");
    assert_eq!(report.xarf_version, SPEC_VERSION);
    assert!(!report.report_id.is_empty());
    assert!(!report.timestamp.is_empty());
}

#[test]
fn report_builder_caller_provided_metadata_wins() {
    let report = ReportBuilder::new("messaging", "spam", "192.0.2.1")
        .reporter(reporter())
        .sender(reporter())
        .source_port(25)
        .timestamp("2025-01-15T00:00:00Z")
        .report_id("550e8400-e29b-41d4-a716-446655440000")
        .xarf_version("4.0.0")
        .extra("protocol", json!("smtp"))
        .extra("smtp_from", json!("spam@bad.example"))
        .build()
        .unwrap()
        .report
        .unwrap();
    assert_eq!(report.xarf_version, "4.0.0");
    assert_eq!(report.report_id, "550e8400-e29b-41d4-a716-446655440000");
    assert_eq!(report.timestamp, "2025-01-15T00:00:00Z");
}

#[test]
fn report_builder_missing_reporter_yields_error() {
    let err = ReportBuilder::new("messaging", "spam", "192.0.2.1")
        .sender(reporter())
        .build()
        .unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("reporter") || matches!(err, xarf::XarfError::Validation(_)));
}

#[test]
fn report_builder_with_evidence() {
    let ev = create_evidence("text/plain", b"hello, xarf!");
    let report = ReportBuilder::new("messaging", "spam", "192.0.2.1")
        .reporter(reporter())
        .sender(reporter())
        .source_port(25)
        .extra("protocol", json!("smtp"))
        .extra("smtp_from", json!("x@bad.example"))
        .add_evidence(ev)
        .build()
        .unwrap()
        .report
        .unwrap();
    let evidence = report.evidence.unwrap();
    assert_eq!(evidence.len(), 1);
    assert_eq!(evidence[0].content_type, "text/plain");
    assert!(evidence[0].hash.as_ref().unwrap().starts_with("sha256:"));
    assert_eq!(evidence[0].size, Some(12));
}

#[test]
fn create_evidence_sha256_default() {
    let ev = create_evidence("text/plain", b"hello");
    assert!(ev.hash.as_ref().unwrap().starts_with("sha256:"));
    assert_eq!(ev.size, Some(5));
    let decoded = BASE64.decode(&ev.payload).unwrap();
    assert_eq!(decoded, b"hello");
}

#[test]
fn create_evidence_all_hash_algorithms_produce_correct_prefix() {
    for (algo, prefix) in [
        (HashAlgorithm::Sha256, "sha256:"),
        (HashAlgorithm::Sha512, "sha512:"),
        (HashAlgorithm::Sha1, "sha1:"),
        (HashAlgorithm::Md5, "md5:"),
    ] {
        let ev = create_evidence_with_options(
            "application/octet-stream",
            b"data",
            EvidenceOptions {
                hash_algorithm: algo,
                description: Some("test".into()),
            },
        );
        assert!(ev.hash.as_ref().unwrap().starts_with(prefix));
        assert_eq!(ev.description.as_deref(), Some("test"));
    }
}

#[test]
fn create_evidence_known_sha256_digest() {
    // sha256("hello, xarf!") = 22e9... — verify against known digest
    let ev = create_evidence("text/plain", b"hello, xarf!");
    let expected = format!(
        "sha256:{}",
        hex::encode_for_test(&sha2_sha256(b"hello, xarf!"))
    );
    assert_eq!(ev.hash.as_deref(), Some(expected.as_str()));
}

#[test]
fn create_report_function_accepts_extras_map() {
    let mut extras = Map::new();
    extras.insert("protocol".into(), json!("smtp"));
    extras.insert("smtp_from".into(), json!("spam@bad.example"));
    extras.insert("source_port".into(), json!(25));
    let result = create_report(
        "messaging",
        "spam",
        "192.0.2.1",
        reporter(),
        reporter(),
        extras,
        CreateReportOptions::default(),
    )
    .unwrap();
    assert!(result.errors.is_empty(), "errors: {:?}", result.errors);
    assert!(result.report.is_some());
}

#[test]
fn create_report_routes_evidence_into_typed_field() {
    let ev_json = json!([{
        "content_type": "text/plain",
        "payload": "aGVsbG8=",
        "hash": "sha256:2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824",
        "size": 5,
    }]);
    let mut extras = Map::new();
    extras.insert("protocol".into(), json!("smtp"));
    extras.insert("smtp_from".into(), json!("spam@bad.example"));
    extras.insert("source_port".into(), json!(25));
    extras.insert("evidence".into(), ev_json);
    let report = create_report(
        "messaging",
        "spam",
        "192.0.2.1",
        reporter(),
        reporter(),
        extras,
        CreateReportOptions::default(),
    )
    .unwrap()
    .report
    .unwrap();
    assert_eq!(report.evidence.unwrap().len(), 1);
}

#[test]
fn report_builder_in_strict_mode_returns_errors_for_missing_recommended() {
    let result = ReportBuilder::new("connection", "ddos", "203.0.113.5")
        .reporter(reporter())
        .sender(reporter())
        .source_port(53)
        .extra("first_seen", json!("2024-01-15T14:15:00Z"))
        .extra("protocol", json!("udp"))
        .extra("destination_ip", json!("192.0.2.42"))
        .build_with_options(xarf::ParseOptions {
            strict: true,
            show_missing_optional: false,
        })
        .unwrap();
    // Recommended fields are absent → strict mode should produce errors.
    assert!(!result.errors.is_empty());
}

#[test]
fn report_builder_tags_round_trip() {
    let report = ReportBuilder::new("messaging", "spam", "192.0.2.1")
        .reporter(reporter())
        .sender(reporter())
        .source_port(25)
        .extra("protocol", json!("smtp"))
        .extra("smtp_from", json!("spam@bad.example"))
        .tags(vec!["malware:emotet", "campaign:spring"])
        .build()
        .unwrap()
        .report
        .unwrap();
    assert_eq!(
        report.tags.unwrap(),
        vec!["malware:emotet".to_string(), "campaign:spring".to_string()]
    );
}

#[test]
fn report_builder_internal_field_round_trip() {
    let mut internal = Map::new();
    internal.insert("ticket".into(), json!("ABUSE-1"));
    internal.insert("analyst".into(), json!("you"));
    let report = ReportBuilder::new("messaging", "spam", "192.0.2.1")
        .reporter(reporter())
        .sender(reporter())
        .source_port(25)
        .extra("protocol", json!("smtp"))
        .extra("smtp_from", json!("spam@bad.example"))
        .internal(internal)
        .build()
        .unwrap()
        .report
        .unwrap();
    let i = report.internal.unwrap();
    assert_eq!(i.get("ticket").unwrap(), "ABUSE-1");
    assert_eq!(i.get("analyst").unwrap(), "you");
}

// -- helpers -----------------------------------------------------------------

fn sha2_sha256(input: &[u8]) -> Vec<u8> {
    use sha2::{Digest, Sha256};
    Sha256::digest(input).to_vec()
}

mod hex {
    pub fn encode_for_test(bytes: &[u8]) -> String {
        let mut s = String::with_capacity(bytes.len() * 2);
        for b in bytes {
            s.push_str(&format!("{b:02x}"));
        }
        s
    }
}
