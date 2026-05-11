//! Unit tests for `xarf::v3_compat`.

use serde_json::{json, Value};
use xarf::{convert_v3_to_v4, deprecation_warning, is_v3_report};

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

fn v3_phishing() -> Value {
    json!({
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
            "AdditionalInfo": {},
        },
    })
}

#[test]
fn detects_v3_with_three_zero_zero_version() {
    assert!(is_v3_report(&v3_spam()));
}

#[test]
fn detects_v3_with_short_version_strings() {
    let mut v = v3_spam();
    v["Version"] = json!("3");
    assert!(is_v3_report(&v));
    v["Version"] = json!("3.0");
    assert!(is_v3_report(&v));
}

#[test]
fn rejects_v4_report_as_v3() {
    let v4 = json!({"xarf_version": "4.2.0", "category": "messaging"});
    assert!(!is_v3_report(&v4));
}

#[test]
fn rejects_v3_missing_reporter_info() {
    let mut v = v3_spam();
    v.as_object_mut().unwrap().remove("ReporterInfo");
    assert!(!is_v3_report(&v));
}

#[test]
fn rejects_v3_missing_report() {
    let mut v = v3_spam();
    v.as_object_mut().unwrap().remove("Report");
    assert!(!is_v3_report(&v));
}

#[test]
fn rejects_non_object() {
    assert!(!is_v3_report(&json!([1, 2, 3])));
    assert!(!is_v3_report(&json!("hello")));
}

#[test]
fn converts_spam_to_v4() {
    let mut warnings = Vec::new();
    let v4 = convert_v3_to_v4(v3_spam(), &mut warnings).unwrap();
    assert_eq!(v4["category"], "messaging");
    assert_eq!(v4["type"], "spam");
    assert_eq!(v4["source_identifier"], "192.0.2.1");
    assert_eq!(v4["protocol"], "smtp");
    assert_eq!(v4["smtp_from"], "spam@example.com");
    assert_eq!(v4["subject"], "buy our stuff");
    assert_eq!(v4["evidence_source"], "spamtrap");
    assert_eq!(v4["source_port"], 25);
    assert_eq!(v4["legacy_version"], "3");
    assert_eq!(v4["reporter"]["org"], "Anti-Spam");
    assert_eq!(v4["reporter"]["contact"], "abuse@antispam.example");
    assert_eq!(v4["reporter"]["domain"], "antispam.example");
    assert!(v4["evidence"].is_array());
    assert_eq!(v4["evidence"][0]["content_type"], "message/rfc822");
    // _internal should carry original type and converted-at metadata.
    assert_eq!(v4["_internal"]["original_report_type"], "Spam");
}

#[test]
fn converts_phishing_to_v4() {
    let mut warnings = Vec::new();
    let v4 = convert_v3_to_v4(v3_phishing(), &mut warnings).unwrap();
    assert_eq!(v4["category"], "content");
    assert_eq!(v4["type"], "phishing");
    assert_eq!(v4["url"], "https://phish.example/login");
    assert_eq!(v4["source_identifier"], "https://phish.example/login");
}

#[test]
fn missing_reporter_org_emits_warning() {
    let mut v = v3_spam();
    v["ReporterInfo"]
        .as_object_mut()
        .unwrap()
        .remove("ReporterOrg");
    let mut warnings = Vec::new();
    let v4 = convert_v3_to_v4(v, &mut warnings).unwrap();
    assert_eq!(v4["reporter"]["org"], "Unknown Organization");
    assert!(!warnings.is_empty());
    assert!(warnings[0].contains("Unknown Organization"));
}

#[test]
fn unknown_v3_type_yields_error() {
    let mut v = v3_spam();
    v["Report"]["ReportType"] = json!("MadeUpType");
    let mut warnings = Vec::new();
    let err = convert_v3_to_v4(v, &mut warnings).unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("unknown ReportType"));
}

#[test]
fn missing_contact_email_yields_error() {
    let mut v = v3_spam();
    v["ReporterInfo"]
        .as_object_mut()
        .unwrap()
        .remove("ReporterContactEmail");
    v["ReporterInfo"]
        .as_object_mut()
        .unwrap()
        .remove("ReporterOrgEmail");
    let mut warnings = Vec::new();
    let err = convert_v3_to_v4(v, &mut warnings).unwrap_err();
    assert!(err.to_string().contains("reporter email"));
}

#[test]
fn malformed_contact_email_yields_error() {
    let mut v = v3_spam();
    v["ReporterInfo"]
        .as_object_mut()
        .unwrap()
        .insert("ReporterContactEmail".into(), json!("notanemail"));
    v["ReporterInfo"]
        .as_object_mut()
        .unwrap()
        .insert("ReporterOrgEmail".into(), json!("notanemail"));
    let mut warnings = Vec::new();
    let err = convert_v3_to_v4(v, &mut warnings).unwrap_err();
    assert!(err.to_string().contains("not a valid email"));
}

#[test]
fn missing_source_identifier_yields_error() {
    let mut v = v3_spam();
    v["Report"].as_object_mut().unwrap().remove("Source");
    let mut warnings = Vec::new();
    let err = convert_v3_to_v4(v, &mut warnings).unwrap_err();
    assert!(err.to_string().contains("source identifier"));
}

#[test]
fn samples_fallback_when_attachment_missing() {
    let mut v = v3_spam();
    let report = v["Report"].as_object_mut().unwrap();
    let att = report.remove("Attachment").unwrap();
    report.insert("Samples".into(), att);
    let mut warnings = Vec::new();
    let v4 = convert_v3_to_v4(v, &mut warnings).unwrap();
    assert!(v4["evidence"].is_array());
    assert_eq!(v4["evidence"].as_array().unwrap().len(), 1);
}

#[test]
fn evidence_hash_and_size_computed_from_decoded_bytes() {
    let mut warnings = Vec::new();
    let v4 = convert_v3_to_v4(v3_spam(), &mut warnings).unwrap();
    let ev = &v4["evidence"][0];
    assert!(ev["hash"].as_str().unwrap().starts_with("sha256:"));
    // "From: spam@example.com" is 22 bytes
    assert_eq!(ev["size"], 22);
}

#[test]
fn deprecation_warning_text_mentions_v3() {
    let w = deprecation_warning();
    assert!(w.to_lowercase().contains("v3"));
    assert!(w.to_lowercase().contains("deprecat"));
}

#[test]
fn pascal_and_lower_case_v3_types_both_mapped() {
    for t in ["Spam", "spam"] {
        let mut v = v3_spam();
        v["Report"]["ReportType"] = json!(t);
        let mut warnings = Vec::new();
        let v4 = convert_v3_to_v4(v, &mut warnings).unwrap();
        assert_eq!(v4["category"], "messaging");
        assert_eq!(v4["type"], "spam");
    }
}

#[test]
fn parse_round_trip_converts_v3() {
    let json = serde_json::to_string(&v3_spam()).unwrap();
    let result = xarf::parse(&json).unwrap();
    assert!(result
        .warnings
        .iter()
        .any(|w| w.message.to_lowercase().contains("v3")));
    let report = result.report.expect("report");
    assert_eq!(report.category.as_str(), "messaging");
    assert_eq!(report.type_, "spam");
    assert_eq!(report.legacy_version.as_deref(), Some("3"));
}

#[test]
fn v3_connection_with_protocol_at_top_level_works() {
    let mut v = json!({
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
    let mut warnings = Vec::new();
    let v4 = convert_v3_to_v4(v.take(), &mut warnings).unwrap();
    assert_eq!(v4["category"], "connection");
    assert_eq!(v4["type"], "login_attack");
    assert_eq!(v4["protocol"], "tcp");
    assert_eq!(v4["destination_ip"], "192.0.2.42");
    assert_eq!(v4["destination_port"], 22);
    assert_eq!(v4["source_port"], 22);
}

#[test]
fn v3_copyright_with_url_source_works() {
    let v = json!({
        "Version": "3.0.0",
        "ReporterInfo": {
            "ReporterOrg": "Rights",
            "ReporterContactEmail": "dmca@rights.example",
        },
        "Report": {
            "ReportClass": "Copyright",
            "ReportType": "Copyright",
            "Date": "2024-01-15T14:30:25Z",
            "Source": {"URL": "https://pirated.example/movie.mp4"},
        },
    });
    let mut warnings = Vec::new();
    let v4 = convert_v3_to_v4(v, &mut warnings).unwrap();
    assert_eq!(v4["category"], "copyright");
    assert_eq!(v4["type"], "copyright");
    assert_eq!(v4["source_identifier"], "https://pirated.example/movie.mp4");
}

#[test]
fn v3_infrastructure_botnet_converts_without_extras() {
    let v = json!({
        "Version": "3.0.0",
        "ReporterInfo": {
            "ReporterOrg": "CERT",
            "ReporterContactEmail": "cert@cert.example",
        },
        "Report": {
            "ReportType": "Botnet",
            "Date": "2024-01-15T11:30:15Z",
            "Source": {"IP": "198.51.100.25"},
            "AdditionalInfo": {"MalwareFamily": "Conficker"},
        },
    });
    let mut warnings = Vec::new();
    let v4 = convert_v3_to_v4(v, &mut warnings).unwrap();
    assert_eq!(v4["category"], "infrastructure");
    assert_eq!(v4["type"], "botnet");
    // No compromise_evidence is added — the converter doesn't synthesise one.
    assert!(v4.get("compromise_evidence").is_none());
}
