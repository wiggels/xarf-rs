//! Core data model for XARF v4 reports.
//!
//! The design choice here is deliberate: rather than encode all 32 concrete
//! report subtypes as a Rust enum (which would force compile-time knowledge of
//! every category-specific field and lock the crate to one frozen version of
//! the spec), we model the report as a single [`Report`] struct with
//! strongly-typed *core* fields plus a `BTreeMap` of category-specific
//! "extra" fields preserved verbatim from JSON.
//!
//! This mirrors what a Go drop-in for XARF would feel like (`encoding/json`
//! unmarshalling with explicit fields + a catch-all map). It allows:
//!
//! * Forward compatibility — schemas evolve, new fields appear, the crate
//!   keeps round-tripping without churn.
//! * Correct strict-mode validation — all rules live in the bundled JSON
//!   Schemas, not in Rust types.
//! * Lossless re-serialization — every byte of input that wasn't malformed
//!   ends up back in the output.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

/// Contact information for the reporter or sender. Shared by both because the
/// JSON shape is identical.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Contact {
    pub org: String,
    pub contact: String,
    pub domain: String,
}

impl Contact {
    pub fn new(
        org: impl Into<String>,
        contact: impl Into<String>,
        domain: impl Into<String>,
    ) -> Self {
        Self {
            org: org.into(),
            contact: contact.into(),
            domain: domain.into(),
        }
    }
}

/// A single evidence item attached to a report.
///
/// `payload` is the base64-encoded body per RFC 4648. Use
/// [`crate::create_evidence`] to compute the hash, size, and encoding from raw
/// bytes.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Evidence {
    pub content_type: String,
    pub payload: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hash: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub size: Option<u64>,
}

/// The seven XARF v4 categories. `Other` is **not** a valid category — it
/// exists only so deserialization of malformed input can produce a structured
/// error rather than panicking. Validation always rejects [`Category::Other`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Category {
    Messaging,
    Connection,
    Content,
    Copyright,
    Vulnerability,
    Infrastructure,
    Reputation,
    /// Unknown / unrecognised value (kept for round-tripping malformed input).
    Other(String),
}

impl Category {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Messaging => "messaging",
            Self::Connection => "connection",
            Self::Content => "content",
            Self::Copyright => "copyright",
            Self::Vulnerability => "vulnerability",
            Self::Infrastructure => "infrastructure",
            Self::Reputation => "reputation",
            Self::Other(s) => s.as_str(),
        }
    }

    pub fn from_str_value(s: &str) -> Self {
        match s {
            "messaging" => Self::Messaging,
            "connection" => Self::Connection,
            "content" => Self::Content,
            "copyright" => Self::Copyright,
            "vulnerability" => Self::Vulnerability,
            "infrastructure" => Self::Infrastructure,
            "reputation" => Self::Reputation,
            other => Self::Other(other.to_string()),
        }
    }

    /// `true` for the seven canonical categories defined by the v4 spec.
    pub fn is_known(&self) -> bool {
        !matches!(self, Self::Other(_))
    }
}

impl Serialize for Category {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for Category {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Ok(Self::from_str_value(&s))
    }
}

/// A XARF v4 report. Core spec fields are strongly typed; category-specific
/// fields live in [`Report::extra`] keyed by the JSON property name.
///
/// `extra` is sorted (it's a `BTreeMap`) so re-serializations are
/// deterministic.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Report {
    pub xarf_version: String,
    pub report_id: String,
    pub timestamp: String,
    pub reporter: Contact,
    pub sender: Contact,
    pub source_identifier: String,
    pub category: Category,
    #[serde(rename = "type")]
    pub type_: String,

    // Recommended core fields
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_port: Option<u16>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub evidence_source: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub evidence: Option<Vec<Evidence>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub confidence: Option<f64>,

    // Optional core fields
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub legacy_version: Option<String>,
    /// Internal operational metadata. NEVER transmitted. Use
    /// [`Report::strip_internal`] before emitting to another system.
    #[serde(rename = "_internal", default, skip_serializing_if = "Option::is_none")]
    pub internal: Option<Map<String, Value>>,

    /// Category-specific and forward-compatible extra fields. Sorted for
    /// deterministic serialization.
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

impl Report {
    /// Remove the `_internal` block. Always do this before transmission.
    pub fn strip_internal(&mut self) -> Option<Map<String, Value>> {
        self.internal.take()
    }

    /// Look up an extra (category-specific) field by name.
    pub fn extra(&self, key: &str) -> Option<&Value> {
        self.extra.get(key)
    }

    /// Insert or replace an extra field. Returns the previous value.
    pub fn set_extra(&mut self, key: impl Into<String>, value: Value) -> Option<Value> {
        self.extra.insert(key.into(), value)
    }

    /// Convert the report to a plain `serde_json::Value` (object). The result
    /// has the core fields first followed by category extras, but ordering of
    /// the extras within the object follows `BTreeMap` order — that is,
    /// alphabetic.
    pub fn to_json_value(&self) -> Value {
        serde_json::to_value(self)
            .expect("XARF report serialization is infallible for valid Report values")
    }
}
