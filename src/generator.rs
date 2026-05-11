//! Report generator — `create_report` and `create_evidence`.

use base64::Engine;
use base64::engine::general_purpose::STANDARD as BASE64;
use chrono::SecondsFormat;
use md5::Md5;
use serde_json::{Map, Value, json};
use sha1::Sha1;
use sha2::{Digest, Sha256, Sha512};
use uuid::Uuid;

use crate::error::{Result, ValidationError, ValidationInfo, ValidationWarning, XarfError};
use crate::model::{Contact, Evidence, Report};
use crate::parser::{ParseOptions, ParseResult, parse_value};

/// The XARF specification version this crate targets.
pub const SPEC_VERSION: &str = "4.2.0";

/// Hash algorithm choices for [`create_evidence`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum HashAlgorithm {
    #[default]
    Sha256,
    Sha512,
    Sha1,
    Md5,
}

impl HashAlgorithm {
    pub fn prefix(self) -> &'static str {
        match self {
            Self::Sha256 => "sha256",
            Self::Sha512 => "sha512",
            Self::Sha1 => "sha1",
            Self::Md5 => "md5",
        }
    }
}

/// Build an [`Evidence`] item from raw bytes:
/// computes the requested hash, base64-encodes the payload, and records the
/// pre-encoding size.
pub fn create_evidence(content_type: impl Into<String>, payload: &[u8]) -> Evidence {
    create_evidence_with_options(content_type, payload, EvidenceOptions::default())
}

/// Options accepted by [`create_evidence_with_options`].
#[derive(Debug, Clone, Default)]
pub struct EvidenceOptions {
    pub description: Option<String>,
    pub hash_algorithm: HashAlgorithm,
}

/// Like [`create_evidence`], but lets the caller supply description and hash
/// algorithm.
pub fn create_evidence_with_options(
    content_type: impl Into<String>,
    payload: &[u8],
    options: EvidenceOptions,
) -> Evidence {
    let hex_digest = match options.hash_algorithm {
        HashAlgorithm::Sha256 => hex(Sha256::digest(payload).as_slice()),
        HashAlgorithm::Sha512 => hex(Sha512::digest(payload).as_slice()),
        HashAlgorithm::Sha1 => hex(Sha1::digest(payload).as_slice()),
        HashAlgorithm::Md5 => hex(Md5::digest(payload).as_slice()),
    };
    let hash = format!("{}:{hex_digest}", options.hash_algorithm.prefix());
    let encoded = BASE64.encode(payload);
    Evidence {
        content_type: content_type.into(),
        payload: encoded,
        description: options.description,
        hash: Some(hash),
        size: Some(payload.len() as u64),
    }
}

fn hex(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        s.push_str(&format!("{b:02x}"));
    }
    s
}

/// Builder for [`Report`] objects.
///
/// Pattern:
///
/// ```rust,ignore
/// let report = xarf::ReportBuilder::new("messaging", "spam", "192.0.2.1")
///     .reporter(xarf::Contact::new("Acme", "abuse@acme.example", "acme.example"))
///     .sender(xarf::Contact::new("Acme", "abuse@acme.example", "acme.example"))
///     .extra("protocol", json!("smtp"))
///     .extra("smtp_from", json!("spam@bad.example"))
///     .build()?;
/// ```
#[derive(Debug, Clone)]
pub struct ReportBuilder {
    category: String,
    type_name: String,
    source_identifier: String,
    reporter: Option<Contact>,
    sender: Option<Contact>,
    timestamp: Option<String>,
    report_id: Option<String>,
    xarf_version: Option<String>,
    evidence_source: Option<String>,
    evidence: Vec<Evidence>,
    tags: Vec<String>,
    confidence: Option<f64>,
    description: Option<String>,
    source_port: Option<u16>,
    internal: Option<Map<String, Value>>,
    extras: Map<String, Value>,
}

impl ReportBuilder {
    pub fn new(
        category: impl Into<String>,
        type_name: impl Into<String>,
        source_identifier: impl Into<String>,
    ) -> Self {
        Self {
            category: category.into(),
            type_name: type_name.into(),
            source_identifier: source_identifier.into(),
            reporter: None,
            sender: None,
            timestamp: None,
            report_id: None,
            xarf_version: None,
            evidence_source: None,
            evidence: Vec::new(),
            tags: Vec::new(),
            confidence: None,
            description: None,
            source_port: None,
            internal: None,
            extras: Map::new(),
        }
    }

    pub fn reporter(mut self, contact: Contact) -> Self {
        self.reporter = Some(contact);
        self
    }

    pub fn sender(mut self, contact: Contact) -> Self {
        self.sender = Some(contact);
        self
    }

    pub fn timestamp(mut self, ts: impl Into<String>) -> Self {
        self.timestamp = Some(ts.into());
        self
    }

    pub fn report_id(mut self, id: impl Into<String>) -> Self {
        self.report_id = Some(id.into());
        self
    }

    pub fn xarf_version(mut self, v: impl Into<String>) -> Self {
        self.xarf_version = Some(v.into());
        self
    }

    pub fn evidence_source(mut self, s: impl Into<String>) -> Self {
        self.evidence_source = Some(s.into());
        self
    }

    pub fn add_evidence(mut self, evidence: Evidence) -> Self {
        self.evidence.push(evidence);
        self
    }

    pub fn tags<I, S>(mut self, tags: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.tags = tags.into_iter().map(Into::into).collect();
        self
    }

    pub fn confidence(mut self, c: f64) -> Self {
        self.confidence = Some(c);
        self
    }

    pub fn description(mut self, d: impl Into<String>) -> Self {
        self.description = Some(d.into());
        self
    }

    pub fn source_port(mut self, p: u16) -> Self {
        self.source_port = Some(p);
        self
    }

    pub fn extra(mut self, key: impl Into<String>, value: Value) -> Self {
        self.extras.insert(key.into(), value);
        self
    }

    pub fn internal(mut self, internal: Map<String, Value>) -> Self {
        self.internal = Some(internal);
        self
    }

    /// Build and validate the report. Returns the validation outcome
    /// (mirrors [`crate::parse`]).
    pub fn build(self) -> Result<ParseResult> {
        self.build_with_options(ParseOptions::default())
    }

    /// Build and validate with explicit [`ParseOptions`]. In strict mode the
    /// result mirrors strict-mode parsing.
    pub fn build_with_options(self, options: ParseOptions) -> Result<ParseResult> {
        let reporter = self.reporter.ok_or_else(|| {
            XarfError::Validation(vec![ValidationError::new(
                "reporter",
                "reporter contact is required",
            )])
        })?;
        let sender = self.sender.ok_or_else(|| {
            XarfError::Validation(vec![ValidationError::new(
                "sender",
                "sender contact is required",
            )])
        })?;

        let timestamp = self
            .timestamp
            .unwrap_or_else(|| chrono::Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true));
        let report_id = self.report_id.unwrap_or_else(|| Uuid::new_v4().to_string());
        let xarf_version = self
            .xarf_version
            .unwrap_or_else(|| SPEC_VERSION.to_string());

        let mut data = Map::new();
        data.insert("xarf_version".into(), Value::String(xarf_version));
        data.insert("report_id".into(), Value::String(report_id));
        data.insert("timestamp".into(), Value::String(timestamp));
        data.insert(
            "reporter".into(),
            serde_json::to_value(&reporter).expect("Contact serialises"),
        );
        data.insert(
            "sender".into(),
            serde_json::to_value(&sender).expect("Contact serialises"),
        );
        data.insert(
            "source_identifier".into(),
            Value::String(self.source_identifier),
        );
        data.insert("category".into(), Value::String(self.category));
        data.insert("type".into(), Value::String(self.type_name));

        if let Some(p) = self.source_port {
            data.insert("source_port".into(), json!(p));
        }
        if let Some(s) = self.evidence_source {
            data.insert("evidence_source".into(), Value::String(s));
        }
        if !self.evidence.is_empty() {
            data.insert(
                "evidence".into(),
                serde_json::to_value(self.evidence).expect("Vec<Evidence> serialises"),
            );
        }
        if !self.tags.is_empty() {
            data.insert(
                "tags".into(),
                Value::Array(self.tags.into_iter().map(Value::String).collect()),
            );
        }
        if let Some(c) = self.confidence {
            data.insert("confidence".into(), json!(c));
        }
        if let Some(d) = self.description {
            data.insert("description".into(), Value::String(d));
        }
        if let Some(internal) = self.internal {
            data.insert("_internal".into(), Value::Object(internal));
        }
        for (k, v) in self.extras {
            data.insert(k, v);
        }

        parse_value(Value::Object(data), options)
    }
}

/// Functional shorthand for [`ReportBuilder`]: matches the Python API
/// signature for callers porting code 1:1.
#[allow(clippy::too_many_arguments)]
pub fn create_report(
    category: &str,
    type_name: &str,
    source_identifier: &str,
    reporter: Contact,
    sender: Contact,
    extras: Map<String, Value>,
    options: CreateReportOptions,
) -> Result<ParseResult> {
    let mut builder = ReportBuilder::new(category, type_name, source_identifier)
        .reporter(reporter)
        .sender(sender);
    for (k, v) in extras {
        // Route well-known keys through dedicated setters so caller intent
        // matches builder semantics (timestamp generation, etc.). Unknown
        // keys flow through `.extra()` verbatim.
        match k.as_str() {
            "timestamp" => {
                if let Some(s) = v.as_str() {
                    builder = builder.timestamp(s);
                }
            }
            "report_id" => {
                if let Some(s) = v.as_str() {
                    builder = builder.report_id(s);
                }
            }
            "xarf_version" => {
                if let Some(s) = v.as_str() {
                    builder = builder.xarf_version(s);
                }
            }
            "evidence_source" => {
                if let Some(s) = v.as_str() {
                    builder = builder.evidence_source(s);
                }
            }
            "evidence" => {
                if let Value::Array(arr) = &v {
                    for item in arr {
                        if let Ok(ev) = serde_json::from_value::<Evidence>(item.clone()) {
                            builder = builder.add_evidence(ev);
                        } else {
                            builder = builder.extra(k.clone(), v.clone());
                            break;
                        }
                    }
                }
            }
            "tags" => {
                if let Value::Array(arr) = &v {
                    let tags: Vec<String> = arr
                        .iter()
                        .filter_map(|t| t.as_str().map(String::from))
                        .collect();
                    builder = builder.tags(tags);
                }
            }
            "confidence" => {
                if let Some(c) = v.as_f64() {
                    builder = builder.confidence(c);
                }
            }
            "description" => {
                if let Some(s) = v.as_str() {
                    builder = builder.description(s);
                }
            }
            "source_port" => {
                if let Some(n) = v.as_u64() {
                    if n <= u16::MAX as u64 {
                        builder = builder.source_port(n as u16);
                    }
                }
            }
            "_internal" => {
                if let Value::Object(obj) = v {
                    builder = builder.internal(obj);
                }
            }
            _ => {
                builder = builder.extra(k, v);
            }
        }
    }
    builder.build_with_options(options.into())
}

/// Options for the functional `create_report` shorthand.
#[derive(Debug, Clone, Copy, Default)]
pub struct CreateReportOptions {
    pub strict: bool,
    pub show_missing_optional: bool,
}

impl From<CreateReportOptions> for ParseOptions {
    fn from(o: CreateReportOptions) -> Self {
        Self {
            strict: o.strict,
            show_missing_optional: o.show_missing_optional,
        }
    }
}

#[allow(dead_code)]
fn _used(_e: ValidationError, _w: ValidationWarning, _i: ValidationInfo, _r: Report) {}
