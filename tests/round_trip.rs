//! End-to-end round-trip tests: parse a canonical sample, mutate the typed
//! report, re-serialize, re-parse, and verify everything survives.

use std::fs;

use serde_json::{Value, json};
use xarf::{ReportBuilder, parse};

#[test]
fn round_trip_through_every_spec_sample() {
    // For each canonical spec sample, parse → re-serialise → re-parse and
    // expect the second parse to produce no new errors.
    for entry in fs::read_dir("tests/spec_samples").unwrap() {
        let path = entry.unwrap().path();
        if path.extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }
        let original = fs::read_to_string(&path).unwrap();
        let result = parse(&original).unwrap();
        assert!(
            result.errors.is_empty(),
            "{}: original parse errors: {:?}",
            path.display(),
            result.errors
        );
        let report = result.report.unwrap();
        let reserialised = serde_json::to_string(&report).unwrap();
        let second = parse(&reserialised).unwrap();
        assert!(
            second.errors.is_empty(),
            "{}: re-parse errors: {:?}",
            path.display(),
            second.errors
        );
        // Core identity must survive intact.
        let r2 = second.report.unwrap();
        assert_eq!(report.report_id, r2.report_id);
        assert_eq!(report.category.as_str(), r2.category.as_str());
        assert_eq!(report.type_, r2.type_);
        assert_eq!(report.source_identifier, r2.source_identifier);
    }
}

#[test]
fn mutate_then_reserialize_preserves_changes() {
    let json = fs::read_to_string(
        "tests/parser_test_samples/valid/v4/messaging/spam_user_complaint_sample.json",
    )
    .unwrap();
    let mut report = parse(&json).unwrap().report.unwrap();

    // Add an extension field.
    report.set_extra("custom_meta", json!({"foo": "bar"}));
    report.tags = Some(vec!["scam:419".into(), "severity:high".into()]);
    report.confidence = Some(0.99);

    let reserialised = serde_json::to_string(&report).unwrap();
    let second = parse(&reserialised).unwrap();
    let r2 = second.report.unwrap();
    assert_eq!(r2.confidence, Some(0.99));
    assert_eq!(r2.tags.unwrap().len(), 2);
    assert_eq!(r2.extra.get("custom_meta").unwrap(), &json!({"foo": "bar"}));
}

#[test]
fn build_then_reparse_produces_identical_typed_report() {
    let result = ReportBuilder::new("messaging", "spam", "192.0.2.1")
        .reporter(xarf::Contact::new(
            "Acme",
            "abuse@acme.example",
            "acme.example",
        ))
        .sender(xarf::Contact::new(
            "Acme",
            "abuse@acme.example",
            "acme.example",
        ))
        .source_port(25)
        .extra("protocol", json!("smtp"))
        .extra("smtp_from", json!("spam@bad.example"))
        .extra("subject", json!("Cheap meds!"))
        .build()
        .unwrap();
    assert!(result.errors.is_empty());
    let report = result.report.unwrap();

    let serialised = serde_json::to_string(&report).unwrap();
    let reparsed = parse(&serialised).unwrap().report.unwrap();
    assert_eq!(reparsed.report_id, report.report_id);
    assert_eq!(reparsed.timestamp, report.timestamp);
    assert_eq!(reparsed.source_port, Some(25));
    assert_eq!(reparsed.extra.get("subject").unwrap(), "Cheap meds!");
}

#[test]
fn unknown_extension_fields_survive_round_trip() {
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
        "future_field_v5": {"nested": [1, 2, 3]},
    });
    let report = xarf::parse_value(data, xarf::ParseOptions::default())
        .unwrap()
        .report
        .unwrap();
    let serialised = serde_json::to_value(&report).unwrap();
    assert_eq!(serialised["future_field_v5"], json!({"nested": [1, 2, 3]}));
}

#[test]
fn strip_internal_means_internal_does_not_re_appear() {
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
        "_internal": {"ticket": "ABUSE-1", "analyst": "you"},
    });
    let mut report = xarf::parse_value(data, xarf::ParseOptions::default())
        .unwrap()
        .report
        .unwrap();
    assert!(report.internal.is_some());

    let stripped = report.strip_internal();
    assert!(stripped.is_some());

    let v: Value = serde_json::to_value(&report).unwrap();
    assert!(
        v.as_object().unwrap().get("_internal").is_none(),
        "stripped report still contains _internal"
    );
}
