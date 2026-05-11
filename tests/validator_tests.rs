//! Unit tests for `xarf::validator`.

use serde_json::json;
use xarf::{ValidateOptions, validate};

fn good_connection_ddos() -> serde_json::Value {
    json!({
        "xarf_version": "4.2.0",
        "report_id": "00000000-0000-4000-8000-000000000000",
        "timestamp": "2024-01-15T14:30:25Z",
        "reporter": {
            "org": "CERT",
            "contact": "cert@example.org",
            "domain": "example.org",
        },
        "sender": {
            "org": "CERT",
            "contact": "cert@example.org",
            "domain": "example.org",
        },
        "source_identifier": "203.0.113.5",
        "source_port": 53,
        "category": "connection",
        "type": "ddos",
        "first_seen": "2024-01-15T14:15:00Z",
        "protocol": "udp",
        "destination_ip": "192.0.2.42",
    })
}

#[test]
fn valid_ddos_report_passes_validation() {
    let v = good_connection_ddos();
    let result = validate(&v, ValidateOptions::default()).unwrap();
    assert!(result.valid, "errors: {:?}", result.errors);
    assert!(result.errors.is_empty());
}

#[test]
fn missing_first_seen_fails_connection_ddos() {
    let mut v = good_connection_ddos();
    v.as_object_mut().unwrap().remove("first_seen");
    let result = validate(&v, ValidateOptions::default()).unwrap();
    assert!(!result.valid);
    assert!(
        result
            .errors
            .iter()
            .any(|e| e.message.contains("first_seen"))
    );
}

#[test]
fn missing_protocol_fails_connection_ddos() {
    let mut v = good_connection_ddos();
    v.as_object_mut().unwrap().remove("protocol");
    let result = validate(&v, ValidateOptions::default()).unwrap();
    assert!(!result.valid);
    assert!(result.errors.iter().any(|e| e.message.contains("protocol")));
}

#[test]
fn invalid_protocol_enum_fails() {
    let mut v = good_connection_ddos();
    v.as_object_mut()
        .unwrap()
        .insert("protocol".into(), json!("carrier_pigeon"));
    let result = validate(&v, ValidateOptions::default()).unwrap();
    assert!(!result.valid);
}

#[test]
fn evidence_payload_must_be_string() {
    let mut v = good_connection_ddos();
    v.as_object_mut().unwrap().insert(
        "evidence".into(),
        json!([{"content_type": "text/plain", "payload": 12345}]),
    );
    let result = validate(&v, ValidateOptions::default()).unwrap();
    assert!(!result.valid);
}

#[test]
fn xarf_version_must_match_4_x_y_pattern() {
    let mut v = good_connection_ddos();
    v.as_object_mut()
        .unwrap()
        .insert("xarf_version".into(), json!("3.0.0"));
    let result = validate(&v, ValidateOptions::default()).unwrap();
    assert!(!result.valid);
    assert!(
        result
            .errors
            .iter()
            .any(|e| e.message.to_lowercase().contains("4"))
    );
}

#[test]
fn report_id_must_be_uuid_format() {
    let mut v = good_connection_ddos();
    v.as_object_mut()
        .unwrap()
        .insert("report_id".into(), json!("not-a-uuid"));
    let result = validate(&v, ValidateOptions::default()).unwrap();
    assert!(!result.valid);
}

#[test]
fn confidence_must_be_in_range() {
    let mut v = good_connection_ddos();
    v.as_object_mut()
        .unwrap()
        .insert("confidence".into(), json!(2.0));
    let result = validate(&v, ValidateOptions::default()).unwrap();
    assert!(!result.valid);
}

#[test]
fn source_port_must_be_positive() {
    let mut v = good_connection_ddos();
    v.as_object_mut()
        .unwrap()
        .insert("source_port".into(), json!(0));
    let result = validate(&v, ValidateOptions::default()).unwrap();
    assert!(!result.valid);
}

#[test]
fn unknown_category_type_pair_fails_master_schema() {
    let mut v = good_connection_ddos();
    v.as_object_mut()
        .unwrap()
        .insert("category".into(), json!("messaging"));
    // messaging/ddos is not a valid combination.
    let result = validate(&v, ValidateOptions::default()).unwrap();
    assert!(!result.valid);
}

#[test]
fn valid_for_every_category() {
    // Build a tiny valid sample for each category and assert it passes.
    let samples: Vec<(&str, &str, serde_json::Value)> = vec![
        (
            "messaging",
            "spam",
            json!({
                "category": "messaging",
                "type": "spam",
                "protocol": "smtp",
                "smtp_from": "x@bad.example",
                "source_port": 25,
            }),
        ),
        (
            "connection",
            "ddos",
            json!({
                "category": "connection",
                "type": "ddos",
                "first_seen": "2024-01-15T14:15:00Z",
                "protocol": "udp",
                "source_port": 53,
                "destination_ip": "192.0.2.42",
            }),
        ),
        (
            "content",
            "phishing",
            json!({
                "category": "content",
                "type": "phishing",
                "url": "https://phish.example/login",
            }),
        ),
        (
            "copyright",
            "copyright",
            json!({
                "category": "copyright",
                "type": "copyright",
                "infringing_url": "https://pirated.example/movie.mp4",
            }),
        ),
        (
            "vulnerability",
            "open_service",
            json!({
                "category": "vulnerability",
                "type": "open_service",
                "service": "memcached",
            }),
        ),
        (
            "infrastructure",
            "botnet",
            json!({
                "category": "infrastructure",
                "type": "botnet",
                "compromise_evidence": "C2 communication observed on 1.2.3.4",
            }),
        ),
        (
            "reputation",
            "blocklist",
            json!({
                "category": "reputation",
                "type": "blocklist",
                "threat_type": "spam_source",
            }),
        ),
    ];

    for (category, type_name, extras) in samples {
        let mut v = json!({
            "xarf_version": "4.2.0",
            "report_id": "550e8400-e29b-41d4-a716-446655440000",
            "timestamp": "2024-01-15T14:30:25Z",
            "reporter": {"org": "X", "contact": "x@x.example", "domain": "x.example"},
            "sender": {"org": "X", "contact": "x@x.example", "domain": "x.example"},
            "source_identifier": "1.2.3.4",
        });
        for (k, val) in extras.as_object().unwrap() {
            v.as_object_mut().unwrap().insert(k.clone(), val.clone());
        }
        let result = validate(&v, ValidateOptions::default()).unwrap();
        assert!(
            result.valid,
            "{}/{} should validate, errors: {:?}",
            category, type_name, result.errors
        );
    }
}

#[test]
fn strict_mode_treats_recommended_as_required() {
    let v = good_connection_ddos();
    let strict = validate(
        &v,
        ValidateOptions {
            strict: true,
            show_missing_optional: false,
        },
    )
    .unwrap();
    // good_connection_ddos lacks recommended fields like evidence_source,
    // destination_port, attack_vector → strict mode should flag.
    assert!(!strict.valid);
}

#[test]
fn show_missing_optional_lists_optional_fields() {
    let v = good_connection_ddos();
    let result = validate(
        &v,
        ValidateOptions {
            strict: false,
            show_missing_optional: true,
        },
    )
    .unwrap();
    let info = result.info.expect("info populated");
    assert!(!info.is_empty());
    // confidence and description are core optional fields that should always
    // appear in this list.
    assert!(info.iter().any(|i| i.field == "confidence"));
}

#[test]
fn registry_lists_all_thirty_two_combinations() {
    let count = xarf::schemas::registry().known_combinations().count();
    assert_eq!(count, 32, "expected 32 canonical combinations");
}

#[test]
fn registry_recognises_canonical_combos() {
    let r = xarf::schemas::registry();
    assert!(r.is_known_combination("messaging", "spam"));
    assert!(r.is_known_combination("connection", "ddos"));
    assert!(r.is_known_combination("content", "phishing"));
    assert!(r.is_known_combination("copyright", "p2p"));
    assert!(!r.is_known_combination("connection", "auth_failure"));
    assert!(!r.is_known_combination("messaging", "spam_v2"));
    assert!(!r.is_known_combination("nothing", "nothing"));
}
