//! Error and result types for the XARF crate.

use thiserror::Error;

/// Top-level error type for the XARF crate. Returned by parsing, validation,
/// and v3-conversion entry points whose failure modes are fatal.
#[derive(Debug, Error)]
pub enum XarfError {
    /// The input was not syntactically valid JSON (or not a JSON object).
    #[error("invalid JSON: {0}")]
    InvalidJson(String),

    /// Schema-validation failure with a structured list of issues. Non-fatal
    /// failures (recoverable into a [`crate::ParseResult`]) are exposed as
    /// `errors` on the result; this variant only surfaces when no result can
    /// be produced.
    #[error("schema validation failed ({} error(s))", .0.len())]
    Validation(Vec<ValidationError>),

    /// A bundled schema failed to parse or compile. This should never happen
    /// for a published release of the crate; it indicates a programmer error.
    #[error("schema error: {0}")]
    Schema(String),

    /// A v3 report could not be converted to v4 (e.g. unknown `ReportType`,
    /// missing source identifier, malformed reporter email).
    #[error("v3 conversion failed: {0}")]
    V3Conversion(String),

    /// Evidence payload encoding/decoding or size-limit failure.
    #[error("evidence error: {0}")]
    Evidence(String),
}

/// A single structured validation issue.
///
/// `field` is a JSON-path-style descriptor of the offending location (empty
/// string for whole-document failures). `message` is a human-readable
/// description.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationError {
    pub field: String,
    pub message: String,
}

impl ValidationError {
    pub fn new(field: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            field: field.into(),
            message: message.into(),
        }
    }
}

/// A non-fatal warning surfaced alongside successful validation results.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationWarning {
    pub field: String,
    pub message: String,
}

impl ValidationWarning {
    pub fn new(field: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            field: field.into(),
            message: message.into(),
        }
    }
}

/// Informational metadata about absent optional/recommended fields.
/// Produced only when callers request `show_missing_optional`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationInfo {
    pub field: String,
    pub message: String,
}

impl ValidationInfo {
    pub fn new(field: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            field: field.into(),
            message: message.into(),
        }
    }
}

pub type Result<T> = std::result::Result<T, XarfError>;
