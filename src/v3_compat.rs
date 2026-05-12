//! XARF v3 compatibility: detect v3 reports and convert them to v4 JSON.
//!
//! Behavioural parity with `xarf-python/xarf/v3_compat.py` and
//! `xarf-javascript/src/v3-legacy.ts`. Conversion never mutates the input;
//! it returns a fresh JSON `Value`.

use base64::Engine;
use base64::engine::general_purpose::STANDARD as BASE64;
use chrono::SecondsFormat;
use serde_json::{Map, Value, json};
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::error::XarfError;
use crate::generator::SPEC_VERSION;

/// The canonical deprecation message attached to every v3 → v4 conversion.
pub fn deprecation_warning() -> String {
    "DEPRECATION WARNING: XARF v3 format detected. \
     The v3 format has been automatically converted to v4. \
     Please update your systems to generate v4 reports directly. \
     v3 support will be removed in a future major version."
        .to_string()
}

/// Heuristic from the JS/Python reference implementations: a top-level
/// `Version` string of `"3"`, `"3.0"`, or `"3.0.0"` combined with
/// `ReporterInfo` and `Report` keys.
pub fn is_v3_report(value: &Value) -> bool {
    let Some(obj) = value.as_object() else {
        return false;
    };
    let version_ok = matches!(
        obj.get("Version").and_then(Value::as_str),
        Some("3" | "3.0" | "3.0.0")
    );
    version_ok && obj.contains_key("ReporterInfo") && obj.contains_key("Report")
}

/// Convert a v3-shaped JSON object into a v4 JSON object.
///
/// `conversion_warnings` accumulates non-fatal messages (e.g. "no
/// ReporterOrg, defaulting to 'Unknown Organization'"). Returns
/// [`XarfError::V3Conversion`] for unrecoverable problems.
pub fn convert_v3_to_v4(
    v3_data: Value,
    conversion_warnings: &mut Vec<String>,
) -> Result<Value, XarfError> {
    let Value::Object(top) = v3_data else {
        return Err(XarfError::V3Conversion(
            "v3 report must be a JSON object".to_string(),
        ));
    };

    let report = top
        .get("Report")
        .and_then(Value::as_object)
        .cloned()
        .unwrap_or_default();
    let reporter_info = top
        .get("ReporterInfo")
        .and_then(Value::as_object)
        .cloned()
        .unwrap_or_default();

    let report_type = report
        .get("ReportType")
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string();

    let (category, v4_type) = map_v3_type(&report_type).ok_or_else(|| {
        let supported = V3_TYPE_MAPPING
            .iter()
            .map(|(k, _)| *k)
            .collect::<Vec<_>>()
            .join(", ");
        XarfError::V3Conversion(format!(
            "unknown ReportType '{report_type}'. Supported types: {supported}"
        ))
    })?;

    let source_identifier = extract_source_identifier(&report)?;
    let contact = extract_contact_info(&reporter_info, conversion_warnings)?;
    let evidence = convert_attachments(&report, conversion_warnings);

    let mut out = Map::new();
    out.insert(
        "xarf_version".into(),
        Value::String(SPEC_VERSION.to_string()),
    );
    out.insert(
        "report_id".into(),
        Value::String(Uuid::new_v4().to_string()),
    );
    out.insert(
        "timestamp".into(),
        report.get("Date").cloned().unwrap_or(Value::String(
            chrono::Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true),
        )),
    );
    out.insert("reporter".into(), contact.clone());
    out.insert("sender".into(), contact);
    out.insert("source_identifier".into(), Value::String(source_identifier));
    out.insert("category".into(), Value::String(category.to_string()));
    out.insert("type".into(), Value::String(v4_type.to_string()));
    out.insert("legacy_version".into(), Value::String("3".to_string()));

    let mut internal = Map::new();
    internal.insert("original_report_type".into(), Value::String(report_type));
    internal.insert(
        "converted_at".into(),
        Value::String(chrono::Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true)),
    );
    out.insert("_internal".into(), Value::Object(internal));

    if let Some(desc) = report.get("AttackDescription").and_then(Value::as_str) {
        out.insert("description".into(), Value::String(desc.to_string()));
    }

    // evidence_source mirrors AdditionalInfo.DetectionMethod when present.
    if let Some(method) = report
        .get("AdditionalInfo")
        .and_then(Value::as_object)
        .and_then(|m| m.get("DetectionMethod"))
        .and_then(Value::as_str)
    {
        out.insert("evidence_source".into(), Value::String(method.to_string()));
    }

    if let Some(ev) = evidence {
        out.insert("evidence".into(), Value::Array(ev));
    }

    match category {
        "messaging" => add_messaging_fields(&mut out, &report)?,
        "connection" => add_connection_fields(&mut out, &report)?,
        "content" => add_content_fields(&mut out, &report)?,
        _ => {}
    }

    Ok(Value::Object(out))
}

const V3_TYPE_MAPPING: &[(&str, (&str, &str))] = &[
    ("Spam", ("messaging", "spam")),
    ("spam", ("messaging", "spam")),
    ("Login-Attack", ("connection", "login_attack")),
    ("login-attack", ("connection", "login_attack")),
    ("Port-Scan", ("connection", "port_scan")),
    ("port-scan", ("connection", "port_scan")),
    ("DDoS", ("connection", "ddos")),
    ("ddos", ("connection", "ddos")),
    ("Phishing", ("content", "phishing")),
    ("phishing", ("content", "phishing")),
    ("Malware", ("content", "malware")),
    ("malware", ("content", "malware")),
    ("Botnet", ("infrastructure", "botnet")),
    ("botnet", ("infrastructure", "botnet")),
    ("Copyright", ("copyright", "copyright")),
    ("copyright", ("copyright", "copyright")),
];

fn map_v3_type(t: &str) -> Option<(&'static str, &'static str)> {
    V3_TYPE_MAPPING
        .iter()
        .find(|(k, _)| *k == t)
        .map(|(_, v)| *v)
}

fn extract_source_identifier(report: &Map<String, Value>) -> Result<String, XarfError> {
    if let Some(source) = report.get("Source").and_then(Value::as_object) {
        if let Some(ip) = source.get("IP").and_then(Value::as_str) {
            return Ok(ip.to_string());
        }
    }
    if let Some(ip) = report.get("SourceIp").and_then(Value::as_str) {
        return Ok(ip.to_string());
    }
    if let Some(source) = report.get("Source").and_then(Value::as_object) {
        if let Some(url) = source.get("URL").and_then(Value::as_str) {
            return Ok(url.to_string());
        }
    }
    if let Some(url) = report.get("Url").and_then(Value::as_str) {
        return Ok(url.to_string());
    }
    Err(XarfError::V3Conversion(
        "no source identifier found (expected Source.IP, SourceIp, Source.URL, or Url)".into(),
    ))
}

fn extract_contact_info(
    reporter_info: &Map<String, Value>,
    warnings: &mut Vec<String>,
) -> Result<Value, XarfError> {
    let contact = reporter_info
        .get("ReporterContactEmail")
        .and_then(Value::as_str)
        .or_else(|| reporter_info.get("ReporterOrgEmail").and_then(Value::as_str))
        .ok_or_else(|| {
            XarfError::V3Conversion(
                "missing reporter email (ReporterContactEmail and ReporterOrgEmail are both absent)".into(),
            )
        })?
        .to_string();

    let (_local, domain) = contact
        .split_once('@')
        .filter(|(_, d)| !d.is_empty())
        .ok_or_else(|| {
            XarfError::V3Conversion(format!(
                "reporter email '{contact}' is not a valid email address"
            ))
        })?;
    let domain = domain.to_string();

    let org = match reporter_info.get("ReporterOrg").and_then(Value::as_str) {
        Some(s) if !s.is_empty() => s.to_string(),
        _ => {
            warnings
                .push("No ReporterOrg found in v3 report, using \"Unknown Organization\"".into());
            "Unknown Organization".to_string()
        }
    };

    Ok(json!({
        "org": org,
        "contact": contact,
        "domain": domain,
    }))
}

fn convert_attachments(
    report: &Map<String, Value>,
    warnings: &mut Vec<String>,
) -> Option<Vec<Value>> {
    let attachments = report
        .get("Attachment")
        .or_else(|| report.get("Samples"))
        .and_then(Value::as_array)?;
    if attachments.is_empty() {
        return None;
    }
    let mut out: Vec<Value> = Vec::new();
    for att in attachments {
        let Some(att) = att.as_object() else {
            continue;
        };
        let description = att.get("Description").and_then(Value::as_str);
        if description.is_none() {
            warnings.push("Evidence attachment has no description, omitting field".into());
        }
        let raw_b64 = att.get("Data").and_then(Value::as_str).unwrap_or("");
        let raw_bytes = BASE64.decode(raw_b64).unwrap_or_default();
        let digest = Sha256::digest(&raw_bytes);
        let hash = format!("sha256:{}", crate::hex::encode(&digest));

        let content_type = att
            .get("ContentType")
            .and_then(Value::as_str)
            .unwrap_or("application/octet-stream")
            .to_string();

        let mut item = Map::new();
        item.insert("content_type".into(), Value::String(content_type));
        item.insert("payload".into(), Value::String(raw_b64.to_string()));
        item.insert("hash".into(), Value::String(hash));
        item.insert(
            "size".into(),
            Value::Number(serde_json::Number::from(raw_bytes.len())),
        );
        if let Some(desc) = description {
            item.insert("description".into(), Value::String(desc.to_string()));
        }
        out.push(Value::Object(item));
    }
    Some(out)
}

fn add_messaging_fields(
    out: &mut Map<String, Value>,
    report: &Map<String, Value>,
) -> Result<(), XarfError> {
    let additional = report
        .get("AdditionalInfo")
        .and_then(Value::as_object)
        .cloned()
        .unwrap_or_default();
    let protocol = report
        .get("Protocol")
        .and_then(Value::as_str)
        .or_else(|| additional.get("Protocol").and_then(Value::as_str))
        .ok_or_else(|| {
            XarfError::V3Conversion("missing protocol for messaging type".to_string())
        })?;
    out.insert("protocol".into(), Value::String(protocol.to_string()));

    if let Some(s) = report
        .get("SmtpMailFromAddress")
        .and_then(Value::as_str)
        .or_else(|| additional.get("SMTPFrom").and_then(Value::as_str))
    {
        out.insert("smtp_from".into(), Value::String(s.to_string()));
    }
    if let Some(s) = report.get("SmtpRcptToAddress").and_then(Value::as_str) {
        out.insert("smtp_to".into(), Value::String(s.to_string()));
    }
    if let Some(s) = report
        .get("SmtpMessageSubject")
        .and_then(Value::as_str)
        .or_else(|| additional.get("Subject").and_then(Value::as_str))
    {
        out.insert("subject".into(), Value::String(s.to_string()));
    }
    let source_port = report
        .get("Source")
        .and_then(Value::as_object)
        .and_then(|s| s.get("Port"))
        .or_else(|| report.get("SourcePort"));
    if let Some(p) = source_port {
        out.insert("source_port".into(), p.clone());
    }
    Ok(())
}

fn add_connection_fields(
    out: &mut Map<String, Value>,
    report: &Map<String, Value>,
) -> Result<(), XarfError> {
    let protocol = report
        .get("Protocol")
        .and_then(Value::as_str)
        .ok_or_else(|| {
            XarfError::V3Conversion("missing protocol for connection type".to_string())
        })?;
    out.insert("protocol".into(), Value::String(protocol.to_string()));
    if let Some(date) = report.get("Date") {
        out.insert("first_seen".into(), date.clone());
    }
    if let Some(dst) = report.get("DestinationIp") {
        out.insert("destination_ip".into(), dst.clone());
    }
    let source_port = report
        .get("Source")
        .and_then(Value::as_object)
        .and_then(|s| s.get("Port"))
        .or_else(|| report.get("SourcePort"));
    if let Some(p) = source_port {
        out.insert("source_port".into(), p.clone());
    }
    if let Some(dp) = report.get("DestinationPort") {
        out.insert("destination_port".into(), dp.clone());
    }
    if let Some(ac) = report.get("AttackCount") {
        out.insert("attack_count".into(), ac.clone());
    }
    Ok(())
}

fn add_content_fields(
    out: &mut Map<String, Value>,
    report: &Map<String, Value>,
) -> Result<(), XarfError> {
    let additional = report
        .get("AdditionalInfo")
        .and_then(Value::as_object)
        .cloned()
        .unwrap_or_default();
    let source = report
        .get("Source")
        .and_then(Value::as_object)
        .cloned()
        .unwrap_or_default();
    let url = report
        .get("Url")
        .and_then(Value::as_str)
        .or_else(|| additional.get("URL").and_then(Value::as_str))
        .or_else(|| source.get("URL").and_then(Value::as_str))
        .ok_or_else(|| {
            XarfError::V3Conversion(format!(
                "missing URL for content type '{}'. Content reports require a URL field",
                out.get("type").and_then(Value::as_str).unwrap_or("")
            ))
        })?;
    out.insert("url".into(), Value::String(url.to_string()));
    Ok(())
}
