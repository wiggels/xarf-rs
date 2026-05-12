//! High-level `parse()` entry point.
//!
//! Steps in order:
//!
//! 1. JSON-decode the input (string → `Value`, or accept a pre-built `Value`).
//! 2. If it looks like a v3 report, run the v3→v4 conversion and add a
//!    deprecation warning.
//! 3. Validate against the bundled v4 schema (master + type-specific).
//! 4. In non-strict mode, attempt to deserialize the data into a typed
//!    [`Report`] even when validation surfaced issues — callers can inspect
//!    `errors` independently.

use serde_json::Value;

use crate::error::{Result, ValidationError, ValidationInfo, ValidationWarning, XarfError};
use crate::model::Report;
use crate::v3_compat;
use crate::validator::{ValidateOptions, validate};

/// Outcome of [`parse`] / [`parse_value`].
///
/// - `report` is `Some` when serde could materialize a typed [`Report`].
/// - `errors` lists every schema or business-rule violation discovered. The
///   list may be non-empty even when `report` is `Some`: lossy parses surface
///   the typed view *and* the errors, leaving the decision to the caller.
#[derive(Debug, Clone)]
pub struct ParseResult {
    pub report: Option<Report>,
    pub errors: Vec<ValidationError>,
    pub warnings: Vec<ValidationWarning>,
    pub info: Option<Vec<ValidationInfo>>,
}

/// Options accepted by [`parse`] and [`parse_value`].
#[derive(Debug, Clone, Copy, Default)]
pub struct ParseOptions {
    /// Strict-mode validation; see [`crate::ValidateOptions::strict`].
    pub strict: bool,
    /// Surface missing optional/recommended fields in
    /// [`ParseResult::info`].
    pub show_missing_optional: bool,
}

impl From<ParseOptions> for ValidateOptions {
    fn from(o: ParseOptions) -> Self {
        Self {
            strict: o.strict,
            show_missing_optional: o.show_missing_optional,
        }
    }
}

/// Parse a XARF report from a JSON string.
///
/// Returns `Err(XarfError::InvalidJson)` only when the input is not valid
/// JSON (or not a JSON object). All other failures surface as entries in
/// [`ParseResult::errors`] / [`ParseResult::warnings`].
pub fn parse(json: &str) -> Result<ParseResult> {
    parse_with_options(json, ParseOptions::default())
}

/// Parse a XARF report from a JSON string with explicit [`ParseOptions`].
pub fn parse_with_options(json: &str, options: ParseOptions) -> Result<ParseResult> {
    let value: Value =
        serde_json::from_str(json).map_err(|e| XarfError::InvalidJson(format!("{e}")))?;
    if !value.is_object() {
        return Err(XarfError::InvalidJson(format!(
            "expected a JSON object, got {}",
            value_type_name(&value)
        )));
    }
    parse_value(value, options)
}

/// Parse a XARF report from a pre-decoded [`serde_json::Value`].
pub fn parse_value(mut value: Value, options: ParseOptions) -> Result<ParseResult> {
    let mut warnings: Vec<ValidationWarning> = Vec::new();

    // ------------------------------------------------------------------
    // Step 1 — v3 detection + conversion
    // ------------------------------------------------------------------
    if v3_compat::is_v3_report(&value) {
        let mut conversion_msgs: Vec<String> = Vec::new();
        value = v3_compat::convert_v3_to_v4(value, &mut conversion_msgs)?;
        warnings.push(ValidationWarning::new("", v3_compat::deprecation_warning()));
        warnings.extend(
            conversion_msgs
                .into_iter()
                .map(|m| ValidationWarning::new("", m)),
        );
    }

    // ------------------------------------------------------------------
    // Step 2 — Schema validation
    // ------------------------------------------------------------------
    let validation = validate(&value, options.into())?;
    let mut errors = validation.errors;
    warnings.extend(validation.warnings);

    // ------------------------------------------------------------------
    // Step 3 — Strict-mode early return if validation already failed
    // ------------------------------------------------------------------
    if options.strict && !errors.is_empty() {
        return Ok(ParseResult {
            report: None,
            errors,
            warnings,
            info: validation.info,
        });
    }

    // ------------------------------------------------------------------
    // Step 4 — Typed deserialisation (best effort in non-strict mode)
    // ------------------------------------------------------------------
    let report = match serde_json::from_value::<Report>(value) {
        Ok(r) => Some(r),
        Err(e) => {
            // Attach the deserialization failure as an error so callers see
            // why `report` is `None`.
            errors.push(ValidationError::new("", format!("deserialize: {e}")));
            None
        }
    };

    Ok(ParseResult {
        report,
        errors,
        warnings,
        info: validation.info,
    })
}

fn value_type_name(v: &Value) -> &'static str {
    match v {
        Value::Null => "null",
        Value::Bool(_) => "boolean",
        Value::Number(_) => "number",
        Value::String(_) => "string",
        Value::Array(_) => "array",
        Value::Object(_) => "object",
    }
}
