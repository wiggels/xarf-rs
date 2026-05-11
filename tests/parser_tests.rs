//! Unit tests for `xarf::parser`.

use serde_json::json;
use xarf::{parse, parse_value, parse_with_options, ParseOptions};

/// A minimum-viable messaging/spam report — passes the schema's
/// conditional rule that `smtp_from` and `source_port` are required when
/// `protocol == "smtp"`.
fn minimal_spam() -> serde_json::Value {
    json!({
        "xarf_version": "4.2.0",
        "report_id": "550e8400-e29b-41d4-a716-446655440000",
        "timestamp": "2024-01-15T14:30:25Z",
        "reporter": {
            "org": "Acme",
            "contact": "abuse@acme.example",
            "domain": "acme.example",
        },
        "sender": {
            "org": "Acme",
            "contact": "abuse@acme.example",
            "domain": "acme.example",
        },
        "source_identifier": "192.0.2.1",
        "source_port": 25,
        "category": "messaging",
        "type": "spam",
        "protocol": "smtp",
        "smtp_from": "spam@bad.example",
    })
}

#[test]
fn parse_string_yields_typed_report() {
    let json = serde_json::to_string(&minimal_spam()).unwrap();
    let result = parse(&json).expect("ok");
    assert!(result.errors.is_empty(), "errors: {:?}", result.errors);
    let report = result.report.expect("typed report");
    assert_eq!(report.category.as_str(), "messaging");
    assert_eq!(report.type_, "spam");
    assert_eq!(report.source_identifier, "192.0.2.1");
    assert_eq!(report.reporter.org, "Acme");
}

#[test]
fn parse_value_path_matches_string_path() {
    let value = minimal_spam();
    let from_value = parse_value(value.clone(), ParseOptions::default()).expect("ok");
    let from_str = parse(&serde_json::to_string(&value).unwrap()).expect("ok");
    assert_eq!(from_value.errors, from_str.errors);
    assert!(from_value.report.is_some());
    assert!(from_str.report.is_some());
}

#[test]
fn malformed_json_returns_invalid_json_error() {
    let err = parse("not json").unwrap_err();
    assert!(matches!(err, xarf::XarfError::InvalidJson(_)));
}

#[test]
fn non_object_top_level_returns_invalid_json_error() {
    let err = parse("[1,2,3]").unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("expected a JSON object"), "got: {msg}");
}

#[test]
fn missing_xarf_version_surfaces_error() {
    let mut data = minimal_spam();
    data.as_object_mut().unwrap().remove("xarf_version");
    let result = parse_value(data, ParseOptions::default()).expect("ok");
    assert!(result
        .errors
        .iter()
        .any(|e| e.message.contains("xarf_version")));
}

#[test]
fn invalid_category_surfaces_error_with_category_in_message() {
    let mut data = minimal_spam();
    data.as_object_mut()
        .unwrap()
        .insert("category".into(), json!("not_a_category"));
    let result = parse_value(data, ParseOptions::default()).expect("ok");
    let combined: String = result
        .errors
        .iter()
        .map(|e| format!("{} {}", e.field, e.message))
        .collect::<Vec<_>>()
        .join("\n");
    assert!(
        combined.to_lowercase().contains("category"),
        "expected category-related error, got:\n{combined}"
    );
}

#[test]
fn unknown_field_warning_in_normal_mode() {
    let mut data = minimal_spam();
    data.as_object_mut()
        .unwrap()
        .insert("totally_made_up".into(), json!("hi"));
    let result = parse_value(data, ParseOptions::default()).expect("ok");
    assert!(
        result
            .warnings
            .iter()
            .any(|w| w.field == "totally_made_up"),
        "expected an unknown-field warning, got: {:?}",
        result.warnings
    );
    // Should not be in errors in non-strict mode.
    assert!(!result
        .errors
        .iter()
        .any(|e| e.field == "totally_made_up"));
}

#[test]
fn unknown_field_promoted_to_error_in_strict_mode() {
    let mut data = minimal_spam();
    data.as_object_mut()
        .unwrap()
        .insert("totally_made_up".into(), json!("hi"));
    let result = parse_value(
        data,
        ParseOptions {
            strict: true,
            show_missing_optional: false,
        },
    )
    .expect("ok");
    assert!(result.errors.iter().any(|e| e.field == "totally_made_up"));
    assert!(result.warnings.is_empty());
}

#[test]
fn strict_mode_requires_recommended_fields() {
    // The minimal_spam dict omits `evidence_source` (recommended) and several
    // others. Non-strict should be fine; strict should emit errors.
    let normal = parse_value(minimal_spam(), ParseOptions::default()).expect("ok");
    assert!(normal.errors.is_empty());

    let strict = parse_value(
        minimal_spam(),
        ParseOptions {
            strict: true,
            show_missing_optional: false,
        },
    )
    .expect("ok");
    assert!(
        !strict.errors.is_empty(),
        "strict mode should require recommended fields"
    );
}

#[test]
fn show_missing_optional_populates_info() {
    let result = parse_with_options(
        &serde_json::to_string(&minimal_spam()).unwrap(),
        ParseOptions {
            strict: false,
            show_missing_optional: true,
        },
    )
    .expect("ok");
    let info = result.info.expect("info populated");
    assert!(!info.is_empty(), "expected at least one missing-optional");
    // Every entry should be prefixed with RECOMMENDED or OPTIONAL.
    for entry in &info {
        assert!(
            entry.message.starts_with("RECOMMENDED") || entry.message.starts_with("OPTIONAL"),
            "entry {entry:?} should start with prefix"
        );
    }
}

#[test]
fn report_round_trips_through_serde() {
    let json = serde_json::to_string(&minimal_spam()).unwrap();
    let report = parse(&json).unwrap().report.unwrap();
    let re_serialised = serde_json::to_value(&report).unwrap();

    // The serialised report must preserve the core fields.
    assert_eq!(re_serialised.get("category").unwrap(), "messaging");
    assert_eq!(re_serialised.get("type").unwrap(), "spam");
    assert_eq!(re_serialised.get("source_identifier").unwrap(), "192.0.2.1");
    // Category-specific fields landed in `extra` and re-emit at the top level.
    assert_eq!(re_serialised.get("protocol").unwrap(), "smtp");
    assert_eq!(
        re_serialised.get("smtp_from").unwrap(),
        "spam@bad.example"
    );
}

#[test]
fn evidence_round_trips() {
    let mut data = minimal_spam();
    data.as_object_mut().unwrap().insert(
        "evidence".into(),
        json!([
            {
                "content_type": "text/plain",
                "payload": "aGVsbG8=",
                "description": "hi",
                "hash": "sha256:2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824",
                "size": 5,
            }
        ]),
    );
    let result = parse_value(data, ParseOptions::default()).expect("ok");
    assert!(result.errors.is_empty(), "errors: {:?}", result.errors);
    let report = result.report.unwrap();
    let evidence = report.evidence.unwrap();
    assert_eq!(evidence.len(), 1);
    assert_eq!(evidence[0].content_type, "text/plain");
    assert_eq!(evidence[0].size, Some(5));
}

#[test]
fn internal_field_round_trips_via_alias() {
    let mut data = minimal_spam();
    data.as_object_mut().unwrap().insert(
        "_internal".into(),
        json!({"ticket": "X-1", "analyst": "you"}),
    );
    let report = parse_value(data, ParseOptions::default())
        .unwrap()
        .report
        .unwrap();
    assert!(report.internal.is_some());
    assert_eq!(
        report.internal.as_ref().unwrap().get("ticket").unwrap(),
        "X-1"
    );

    // Strip and re-serialise → no `_internal` in output.
    let mut stripped = report.clone();
    stripped.strip_internal();
    let json = serde_json::to_string(&stripped).unwrap();
    assert!(!json.contains("_internal"));
}

#[test]
fn tags_round_trip() {
    let mut data = minimal_spam();
    data.as_object_mut().unwrap().insert(
        "tags".into(),
        json!(["malware:emotet", "campaign:winter-2024"]),
    );
    let report = parse_value(data, ParseOptions::default())
        .unwrap()
        .report
        .unwrap();
    let tags = report.tags.unwrap();
    assert_eq!(tags.len(), 2);
    assert_eq!(tags[0], "malware:emotet");
}

#[test]
fn invalid_tag_format_surfaces_error() {
    let mut data = minimal_spam();
    data.as_object_mut().unwrap().insert(
        "tags".into(),
        // No colon → fails the pattern.
        json!(["badtag"]),
    );
    let result = parse_value(data, ParseOptions::default()).expect("ok");
    assert!(
        result.errors.iter().any(|e| e.field.starts_with("tags")),
        "expected a tags.* error, got: {:?}",
        result.errors
    );
}

#[test]
fn extra_fields_preserved_across_parse() {
    let mut data = minimal_spam();
    data.as_object_mut()
        .unwrap()
        .insert("subject".into(), json!("Cheap meds!"));
    data.as_object_mut()
        .unwrap()
        .insert("smtp_to".into(), json!("victim@example.org"));
    let report = parse_value(data, ParseOptions::default())
        .unwrap()
        .report
        .unwrap();
    assert_eq!(
        report.extra.get("subject").unwrap(),
        &json!("Cheap meds!")
    );
    assert_eq!(
        report.extra.get("smtp_to").unwrap(),
        &json!("victim@example.org")
    );
}
