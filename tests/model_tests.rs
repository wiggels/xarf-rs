//! Unit tests for `xarf::model` (Contact, Evidence, Report, Category).

use serde_json::{Value, json};
use xarf::{Category, Contact, Evidence, Report};

#[test]
fn category_serializes_as_string() {
    let c = Category::Messaging;
    let v = serde_json::to_value(&c).unwrap();
    assert_eq!(v, Value::String("messaging".into()));
}

#[test]
fn category_deserializes_known_values() {
    let c: Category = serde_json::from_str(r#""content""#).unwrap();
    assert_eq!(c, Category::Content);
    assert!(c.is_known());
}

#[test]
fn category_deserializes_unknown_as_other() {
    let c: Category = serde_json::from_str(r#""tribbles""#).unwrap();
    assert_eq!(c, Category::Other("tribbles".into()));
    assert!(!c.is_known());
    // Round-trips back to the same string.
    let s = serde_json::to_string(&c).unwrap();
    assert_eq!(s, r#""tribbles""#);
}

#[test]
fn category_as_str_all_known() {
    for (cat, expected) in [
        (Category::Messaging, "messaging"),
        (Category::Connection, "connection"),
        (Category::Content, "content"),
        (Category::Copyright, "copyright"),
        (Category::Vulnerability, "vulnerability"),
        (Category::Infrastructure, "infrastructure"),
        (Category::Reputation, "reputation"),
    ] {
        assert_eq!(cat.as_str(), expected);
    }
}

#[test]
fn contact_serializes_in_expected_order() {
    let c = Contact::new("Acme", "abuse@acme.example", "acme.example");
    let v = serde_json::to_value(&c).unwrap();
    assert_eq!(
        v,
        json!({
            "org": "Acme",
            "contact": "abuse@acme.example",
            "domain": "acme.example",
        })
    );
}

#[test]
fn evidence_serialises_omits_optional_fields() {
    let e = Evidence {
        content_type: "text/plain".into(),
        payload: "aGVsbG8=".into(),
        description: None,
        hash: None,
        size: None,
    };
    let v = serde_json::to_value(&e).unwrap();
    let obj = v.as_object().unwrap();
    assert!(!obj.contains_key("description"));
    assert!(!obj.contains_key("hash"));
    assert!(!obj.contains_key("size"));
    assert_eq!(obj.get("content_type").unwrap(), "text/plain");
}

#[test]
fn report_extras_round_trip_through_set_and_get() {
    let mut report: Report = serde_json::from_value(minimal_v4_object()).unwrap();
    report.set_extra("custom_extension", json!({"hello": "world"}));
    let v = report.to_json_value();
    assert_eq!(v["custom_extension"]["hello"], "world");
    assert_eq!(
        report.extra("custom_extension").unwrap(),
        &json!({"hello": "world"})
    );
}

#[test]
fn report_strip_internal_removes_internal_block() {
    let mut data = minimal_v4_object();
    data["_internal"] = json!({"ticket": "X-1"});
    let mut report: Report = serde_json::from_value(data).unwrap();
    assert!(report.internal.is_some());
    let stripped = report.strip_internal();
    assert!(stripped.is_some());
    assert!(report.internal.is_none());

    let v = report.to_json_value();
    assert!(v.get("_internal").is_none());
}

#[test]
fn report_serialises_with_type_field_named_type() {
    let report: Report = serde_json::from_value(minimal_v4_object()).unwrap();
    let v = report.to_json_value();
    assert!(v.as_object().unwrap().contains_key("type"));
    assert_eq!(v["type"], "spam");
}

#[test]
fn report_clone_and_equality() {
    let r1: Report = serde_json::from_value(minimal_v4_object()).unwrap();
    let r2 = r1.clone();
    assert_eq!(r1, r2);
}

fn minimal_v4_object() -> Value {
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
}
