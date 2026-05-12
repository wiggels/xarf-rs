//! Schema-based validator for raw XARF v4 report JSON.
//!
//! Performs three jobs on a single pass:
//!
//! 1. Runs the bundled JSON Schema (master + type-specific) against the input.
//! 2. Detects unknown fields and reports them as warnings (or errors in strict
//!    mode).
//! 3. Optionally enumerates missing optional/recommended fields as
//!    informational hints.
//!
//! This mirrors the Python `XARFValidator` / TS `XARFValidator` reference
//! implementations.

use std::collections::BTreeSet;

use serde_json::Value;

use crate::error::{Result, ValidationError, ValidationInfo, ValidationWarning};
use crate::schemas::registry;

/// Outcome of [`validate`]. `valid` is `true` iff `errors` is empty.
#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub valid: bool,
    pub errors: Vec<ValidationError>,
    pub warnings: Vec<ValidationWarning>,
    pub info: Option<Vec<ValidationInfo>>,
}

/// Options for [`validate`].
#[derive(Debug, Clone, Copy, Default)]
pub struct ValidateOptions {
    /// When `true`, fields marked `x-recommended: true` in the schema are
    /// treated as required and unknown-field warnings are promoted to errors.
    pub strict: bool,
    /// When `true`, [`ValidationResult::info`] is populated with details on
    /// every absent optional/recommended field.
    pub show_missing_optional: bool,
}

/// Validate raw report data against the bundled XARF v4 schemas.
///
/// Accepts a `serde_json::Value` so callers can pre-parse from string or
/// build dynamically. Returns a [`ValidationResult`]; never panics on bad
/// data.
pub fn validate(data: &Value, options: ValidateOptions) -> Result<ValidationResult> {
    // ------------------------------------------------------------------
    // Step 1 — JSON Schema validation against the (cached) master schema
    // ------------------------------------------------------------------
    let validator = registry().master_validator(options.strict)?;

    let mut errors: Vec<ValidationError> = Vec::new();
    let mut seen_errors: BTreeSet<(String, String)> = BTreeSet::new();
    for err in validator.iter_errors(data) {
        let field = err
            .instance_path()
            .as_str()
            .trim_start_matches('/')
            .replace('/', ".");
        let message = err.to_string();
        let key = (field.clone(), message.clone());
        if seen_errors.insert(key) {
            errors.push(ValidationError::new(field, message));
        }
    }

    // ------------------------------------------------------------------
    // Step 2 — Unknown-field detection (warnings, or errors in strict mode)
    // ------------------------------------------------------------------
    let mut warnings: Vec<ValidationWarning> = Vec::new();
    let category = data.get("category").and_then(Value::as_str).unwrap_or("");
    let type_name = data.get("type").and_then(Value::as_str).unwrap_or("");

    if !category.is_empty() && !type_name.is_empty() {
        if let Value::Object(obj) = data {
            let type_fields = registry().type_known_fields(category, type_name);
            for key in obj.keys() {
                if key == "_internal" || is_known_field(key, type_fields) {
                    continue;
                }
                warnings.push(ValidationWarning::new(
                    key.clone(),
                    format!("Unknown field '{key}' is not defined in the XARF schema"),
                ));
            }
        }
    }

    // Strict mode: promote unknown-field warnings to errors.
    if options.strict && !warnings.is_empty() {
        for w in warnings.drain(..) {
            let key = (w.field.clone(), w.message.clone());
            if seen_errors.insert(key) {
                errors.push(ValidationError::new(w.field, w.message));
            }
        }
    }

    // ------------------------------------------------------------------
    // Step 3 — Missing optional/recommended-field discovery
    // ------------------------------------------------------------------
    let info = if options.show_missing_optional && !category.is_empty() && !type_name.is_empty() {
        Some(collect_missing_optional(data, category, type_name))
    } else {
        None
    };

    Ok(ValidationResult {
        valid: errors.is_empty(),
        errors,
        warnings,
        info,
    })
}

/// Check whether `key` is a known field for `(category, type)`. Looks up the
/// type's precomputed sorted field list (which already includes core fields);
/// no allocation, single binary search.
fn is_known_field(key: &str, type_fields: Option<&[String]>) -> bool {
    type_fields
        .map(|fs| fs.binary_search_by(|n| n.as_str().cmp(key)).is_ok())
        .unwrap_or_else(|| CORE_FIELD_NAMES.binary_search(&key).is_ok())
}

/// Alphabetically-sorted list of core fields, for the fallback path when the
/// `(category, type)` combo is unknown.
const CORE_FIELD_NAMES: &[&str] = &[
    "_internal",
    "category",
    "confidence",
    "description",
    "evidence",
    "evidence_source",
    "legacy_version",
    "report_id",
    "reporter",
    "sender",
    "source_identifier",
    "source_port",
    "tags",
    "timestamp",
    "type",
    "xarf_version",
];

/// Collect informational entries for every optional/recommended field that is
/// absent from `data`. Order follows: core fields (alphabetic) then type
/// fields (insertion order from the schema).
fn collect_missing_optional(data: &Value, category: &str, type_name: &str) -> Vec<ValidationInfo> {
    let mut info: Vec<ValidationInfo> = Vec::new();
    let Value::Object(obj) = data else {
        return info;
    };
    let reg = registry();

    for meta in reg.core_optional_fields() {
        if obj.contains_key(&meta.name) {
            continue;
        }
        info.push(meta_to_info(meta));
    }

    if let Some(opt) = reg.type_optional_fields(category, type_name) {
        for meta in opt {
            if obj.contains_key(&meta.name) {
                continue;
            }
            info.push(meta_to_info(meta));
        }
    }

    info
}

fn meta_to_info(meta: &crate::schemas::FieldMeta) -> ValidationInfo {
    let prefix = if meta.recommended {
        "RECOMMENDED"
    } else {
        "OPTIONAL"
    };
    ValidationInfo::new(meta.name.clone(), format!("{prefix}: {}", meta.description))
}

/// Convenience wrapper that returns just the `errors` list, useful for
/// quick checks. Equivalent to `validate(...).errors`.
pub fn quick_errors(data: &Value, strict: bool) -> Result<Vec<ValidationError>> {
    Ok(validate(
        data,
        ValidateOptions {
            strict,
            show_missing_optional: false,
        },
    )?
    .errors)
}
