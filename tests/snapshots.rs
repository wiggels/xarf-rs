//! Snapshot tests via `insta`.
//!
//! These cover output shape — anything user-visible that should not silently
//! drift between releases. Non-deterministic fields (UUIDs, current
//! timestamps, hash digests of synthesised data) are redacted so the
//! snapshots stay stable across runs.
//!
//! Snapshot files live in `tests/snapshots/`. Review pending changes with
//! `cargo insta review` (install via `cargo install cargo-insta`). When you
//! change the public output of the crate, update the snapshot intentionally
//! — the diff is the test.

use insta::{assert_json_snapshot, assert_snapshot};
use serde_json::{json, Map, Value};
use xarf::{
    convert_v3_to_v4, create_evidence_with_options, parse, parse_value, parse_with_options,
    validate, Contact, EvidenceOptions, HashAlgorithm, ParseOptions, ReportBuilder,
    ValidateOptions,
};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn redactions_for_v3_conversion() -> Vec<(&'static str, &'static str)> {
    vec![
        (".report_id", "[uuid]"),
        ("._internal.converted_at", "[timestamp]"),
    ]
}

fn v3_spam() -> Value {
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
            "Attachment": [
                {
                    "ContentType": "message/rfc822",
                    "Description": "Spam email",
                    "Data": "RnJvbTogc3BhbUBleGFtcGxlLmNvbQ==",
                }
            ],
            "AdditionalInfo": {
                "Protocol": "smtp",
                "SMTPFrom": "spam@example.com",
                "Subject": "buy our stuff",
                "DetectionMethod": "spamtrap",
            },
        },
    })
}

// ---------------------------------------------------------------------------
// v3 → v4 conversion snapshots
// ---------------------------------------------------------------------------

#[test]
fn snapshot_v3_spam_conversion() {
    let mut warnings: Vec<String> = Vec::new();
    let v4 = convert_v3_to_v4(v3_spam(), &mut warnings).unwrap();
    assert_json_snapshot!("v3_spam_conversion", v4, {
        ".report_id" => "[uuid]",
        "._internal.converted_at" => "[timestamp]",
    });
}

#[test]
fn snapshot_v3_phishing_conversion() {
    let v3 = json!({
        "Version": "3.0.0",
        "ReporterInfo": {
            "ReporterOrg": "Sec",
            "ReporterContactEmail": "abuse@sec.example",
        },
        "Report": {
            "ReportClass": "Content",
            "ReportType": "Phishing",
            "Date": "2024-01-15T14:30:25Z",
            "Source": {"URL": "https://phish.example/login", "Type": "uri"},
            "AdditionalInfo": {
                "TargetBrand": "Example Bank",
                "DetectionMethod": "crawler",
            },
        },
    });
    let mut warnings: Vec<String> = Vec::new();
    let v4 = convert_v3_to_v4(v3, &mut warnings).unwrap();
    assert_json_snapshot!("v3_phishing_conversion", v4, {
        ".report_id" => "[uuid]",
        "._internal.converted_at" => "[timestamp]",
    });
}

#[test]
fn snapshot_v3_connection_login_attack_conversion() {
    let v3 = json!({
        "Version": "3.0.0",
        "ReporterInfo": {
            "ReporterOrg": "CERT",
            "ReporterContactEmail": "cert@cert.example",
        },
        "Report": {
            "ReportClass": "Network",
            "ReportType": "Login-Attack",
            "Date": "2024-01-15T14:30:25Z",
            "Source": {"IP": "203.0.113.5", "Port": 22, "Type": "ip"},
            "Protocol": "tcp",
            "DestinationIp": "192.0.2.42",
            "DestinationPort": 22,
        },
    });
    let mut warnings: Vec<String> = Vec::new();
    let v4 = convert_v3_to_v4(v3, &mut warnings).unwrap();
    assert_json_snapshot!("v3_login_attack_conversion", v4, {
        ".report_id" => "[uuid]",
        "._internal.converted_at" => "[timestamp]",
    });
}

#[test]
fn snapshot_v3_with_missing_reporter_org_warnings() {
    let mut v3 = v3_spam();
    v3["ReporterInfo"]
        .as_object_mut()
        .unwrap()
        .remove("ReporterOrg");
    let mut warnings: Vec<String> = Vec::new();
    let _ = convert_v3_to_v4(v3, &mut warnings).unwrap();
    assert_snapshot!("v3_missing_reporter_org_warnings", warnings.join("\n"));
}

// ---------------------------------------------------------------------------
// Validation error snapshots
// ---------------------------------------------------------------------------

#[test]
fn snapshot_missing_xarf_version_errors() {
    let data = json!({
        "report_id": "550e8400-e29b-41d4-a716-446655440000",
        "timestamp": "2024-01-15T14:30:25Z",
        "reporter": {"org": "X", "contact": "x@x.example", "domain": "x.example"},
        "sender": {"org": "X", "contact": "x@x.example", "domain": "x.example"},
        "source_identifier": "192.0.2.1",
        "category": "messaging",
        "type": "spam",
        "protocol": "smtp",
        "smtp_from": "spam@bad.example",
        "source_port": 25,
    });
    let result = validate(&data, ValidateOptions::default()).unwrap();
    assert_json_snapshot!(
        "errors_missing_xarf_version",
        result.errors
            .iter()
            .map(|e| json!({"field": e.field, "message": e.message}))
            .collect::<Vec<_>>()
    );
}

#[test]
fn snapshot_invalid_category_errors() {
    let data = json!({
        "xarf_version": "4.2.0",
        "report_id": "550e8400-e29b-41d4-a716-446655440000",
        "timestamp": "2024-01-15T14:30:25Z",
        "reporter": {"org": "X", "contact": "x@x.example", "domain": "x.example"},
        "sender": {"org": "X", "contact": "x@x.example", "domain": "x.example"},
        "source_identifier": "192.0.2.1",
        "category": "imaginary_category",
        "type": "spam",
    });
    let result = validate(&data, ValidateOptions::default()).unwrap();
    assert_json_snapshot!(
        "errors_invalid_category",
        result.errors
            .iter()
            .map(|e| json!({"field": e.field, "message": e.message}))
            .collect::<Vec<_>>()
    );
}

#[test]
fn snapshot_strict_mode_promotes_recommended_messaging_spam() {
    // A minimal messaging/spam without evidence_source / subject (recommended).
    let data = json!({
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
    });
    let result = validate(
        &data,
        ValidateOptions {
            strict: true,
            show_missing_optional: false,
        },
    )
    .unwrap();
    let mut errors: Vec<_> = result
        .errors
        .iter()
        .map(|e| (e.field.clone(), e.message.clone()))
        .collect();
    errors.sort();
    assert_json_snapshot!("errors_strict_messaging_spam", errors);
}

// ---------------------------------------------------------------------------
// Missing-optional info snapshots
// ---------------------------------------------------------------------------

#[test]
fn snapshot_show_missing_optional_for_messaging_spam() {
    let data = json!({
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
    });
    let result = parse_with_options(
        &serde_json::to_string(&data).unwrap(),
        ParseOptions {
            strict: false,
            show_missing_optional: true,
        },
    )
    .unwrap();
    let mut info: Vec<_> = result
        .info
        .unwrap()
        .into_iter()
        .map(|i| (i.field, i.message))
        .collect();
    info.sort();
    assert_json_snapshot!("info_missing_optional_messaging_spam", info);
}

// ---------------------------------------------------------------------------
// Typed Report snapshots
// ---------------------------------------------------------------------------

#[test]
fn snapshot_typed_report_messaging_spam() {
    let raw = std::fs::read_to_string(
        "tests/parser_test_samples/valid/v4/messaging/spam_user_complaint_sample.json",
    )
    .unwrap();
    let report = parse(&raw).unwrap().report.unwrap();
    assert_json_snapshot!("typed_report_messaging_spam", report, {
        ".evidence[].payload" => "[base64-payload]",
    });
}

// ---------------------------------------------------------------------------
// Generator snapshots
// ---------------------------------------------------------------------------

#[test]
fn snapshot_built_report_with_deterministic_metadata() {
    let evidence = create_evidence_with_options(
        "text/plain",
        b"hello, xarf!",
        EvidenceOptions {
            description: Some("greeting".into()),
            hash_algorithm: HashAlgorithm::Sha256,
        },
    );
    let report = ReportBuilder::new("messaging", "spam", "192.0.2.1")
        .reporter(Contact::new("Acme", "abuse@acme.example", "acme.example"))
        .sender(Contact::new("Acme", "abuse@acme.example", "acme.example"))
        .report_id("550e8400-e29b-41d4-a716-446655440000")
        .timestamp("2024-01-15T14:30:25Z")
        .xarf_version("4.2.0")
        .source_port(25)
        .evidence_source("spamtrap")
        .extra("protocol", json!("smtp"))
        .extra("smtp_from", json!("spam@bad.example"))
        .extra("subject", json!("Cheap meds!"))
        .tags(vec!["scam:advance_fee", "severity:medium"])
        .add_evidence(evidence)
        .build()
        .unwrap()
        .report
        .unwrap();
    assert_json_snapshot!("built_messaging_spam_report", report);
}

#[test]
fn snapshot_create_evidence_for_each_hash_algorithm() {
    let payload = b"snapshot-fixed-payload";
    let mut results: Vec<(&'static str, String)> = Vec::new();
    for algo in [
        HashAlgorithm::Sha256,
        HashAlgorithm::Sha512,
        HashAlgorithm::Sha1,
        HashAlgorithm::Md5,
    ] {
        let ev = create_evidence_with_options(
            "text/plain",
            payload,
            EvidenceOptions {
                description: None,
                hash_algorithm: algo,
            },
        );
        results.push((algo.prefix(), ev.hash.unwrap()));
    }
    assert_json_snapshot!("evidence_hashes_per_algorithm", results);
}

// ---------------------------------------------------------------------------
// Parse warnings snapshot (unknown field handling)
// ---------------------------------------------------------------------------

#[test]
fn snapshot_warnings_for_unknown_fields() {
    let mut data = json!({
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
        "future_v5_field_a": "x",
        "future_v5_field_b": 42,
    });
    let extras = data.as_object_mut().unwrap();
    extras.insert("alpha_unknown".into(), json!("x"));
    extras.insert("zulu_unknown".into(), json!("z"));

    let result = parse_value(data, ParseOptions::default()).unwrap();
    let mut warnings: Vec<_> = result
        .warnings
        .iter()
        .map(|w| (w.field.clone(), w.message.clone()))
        .collect();
    warnings.sort();
    assert_json_snapshot!("warnings_unknown_fields", warnings);
}

// ---------------------------------------------------------------------------
// Round-trip stability snapshot
// ---------------------------------------------------------------------------

#[test]
fn snapshot_round_trip_for_canonical_spec_sample() {
    let raw = std::fs::read_to_string("tests/spec_samples/messaging-spam.json").unwrap();
    let report = parse(&raw).unwrap().report.unwrap();
    let reserialised: Value = serde_json::to_value(&report).unwrap();
    // Snapshot the parsed-then-serialised shape so any silent change in field
    // ordering or representation is caught.
    assert_json_snapshot!("round_trip_spec_messaging_spam", reserialised, {
        ".evidence[].payload" => "[base64-payload]",
    });
}

// silence unused — keeps the import list complete for future expansion.
#[allow(dead_code)]
fn _used_imports(_m: Map<String, Value>) {}
#[allow(dead_code)]
fn _ref_redactions(_v: Vec<(&'static str, &'static str)>) {}
#[allow(dead_code)]
fn _ref_helper() -> Vec<(&'static str, &'static str)> {
    redactions_for_v3_conversion()
}
