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

use serde_json::{Map, Value};

use crate::error::{Result, ValidationError, ValidationInfo, ValidationWarning, XarfError};
use crate::schemas::{self, registry};

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
            let known = known_field_names(category, type_name);
            for key in obj.keys() {
                if !known.contains(key.as_str()) && key != "_internal" {
                    warnings.push(ValidationWarning::new(
                        key.clone(),
                        format!("Unknown field '{key}' is not defined in the XARF schema"),
                    ));
                }
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

/// Names of every field defined by the core schema *plus* every field defined
/// by the type schema for `(category, type)` and its `allOf` ancestors.
fn known_field_names(category: &str, type_name: &str) -> BTreeSet<&'static str> {
    let mut names: BTreeSet<&'static str> = BTreeSet::new();
    for n in CORE_FIELDS.iter().copied() {
        names.insert(n);
    }
    if let Some(set) = type_schema_fields(category, type_name) {
        for n in set {
            names.insert(n);
        }
    }
    names
}

/// Compile-time list of every property the core schema defines. Kept in sync
/// with `schemas/v4/xarf-core.json`.
const CORE_FIELDS: &[&str] = &[
    "xarf_version",
    "report_id",
    "timestamp",
    "reporter",
    "sender",
    "source_identifier",
    "source_port",
    "category",
    "type",
    "evidence_source",
    "evidence",
    "tags",
    "confidence",
    "description",
    "legacy_version",
    "_internal",
];

/// Returns the property names defined by `schemas/v4/types/<category>-<type>`
/// **plus** any names defined by base schemas it `$ref`s (e.g. content-base).
fn type_schema_fields(category: &str, type_name: &str) -> Option<Vec<&'static str>> {
    // Computed on first call; the schema documents are static and the result
    // is too. Doing it dynamically (rather than encoding lookup tables by hand)
    // keeps us correct as the spec evolves.
    let schema = registry().type_schema(category, type_name)?;
    let mut acc: Vec<String> = Vec::new();
    collect_property_names(schema, &mut acc);
    // Leak short strings into 'static. Field-set sizes are bounded (<150 names
    // total across the entire spec) and registry initialisation runs once, so
    // leaking is the cheapest way to expose &'static str references without
    // shifting every other call-site to owned strings.
    Some(acc.into_iter().map(leak_str).collect())
}

fn leak_str(s: String) -> &'static str {
    Box::leak(s.into_boxed_str())
}

fn collect_property_names(schema: &Value, out: &mut Vec<String>) {
    if let Some(Value::Object(props)) = schema.get("properties") {
        for k in props.keys() {
            if !out.iter().any(|existing| existing == k) {
                out.push(k.clone());
            }
        }
    }
    if let Some(Value::Array(all_of)) = schema.get("allOf") {
        for sub in all_of {
            // $ref to a base schema → resolve via the registry's retriever.
            if let Some(Value::String(href)) = sub.get("$ref") {
                if href.contains("../xarf-core.json") {
                    // Core fields already accounted for elsewhere.
                    continue;
                }
                // Type bases live in the same directory; resolve to the
                // registry's parsed copies. We only need content-base today,
                // but the loop is generic.
                if href.contains("content-base.json") {
                    if let Some(base) = registry()
                        .type_schema("content", "phishing")
                        .and_then(|_| try_get_base_schema("content-base"))
                    {
                        collect_property_names(&base, out);
                    }
                }
                continue;
            }
            collect_property_names(sub, out);
        }
    }
}

/// Helper that pulls the canonical content-base schema out of the retriever
/// via a synthetic URI lookup. Falls back to `None` if the registry doesn't
/// know about it (should never happen).
fn try_get_base_schema(stem: &str) -> Option<Value> {
    let uri = format!("https://xarf.org/schemas/v4/types/{stem}.json");
    let reg = registry();
    // The registry exposes the retriever through the validator, but for
    // property-name extraction we can read from the bundled string directly
    // via include_str. To avoid a separate map, embed only the content-base
    // case here (the only base schema in the v4 spec).
    if stem == "content-base" {
        let s = include_str!("../schemas/v4/types/content-base.json");
        if let Ok(v) = serde_json::from_str::<Value>(s) {
            // Touch the registry so the compiler keeps the dependency.
            let _ = reg.master_schema();
            let _ = uri;
            return Some(v);
        }
    }
    None
}

/// Collect informational entries for every optional/recommended field that is
/// absent from `data`. Order follows: core fields (alphabetic) then type
/// fields (insertion order from the schema).
fn collect_missing_optional(
    data: &Value,
    category: &str,
    type_name: &str,
) -> Vec<ValidationInfo> {
    let mut info: Vec<ValidationInfo> = Vec::new();
    let obj = match data {
        Value::Object(o) => o,
        _ => return info,
    };
    let required = schemas::core_required_fields();

    // Core schema fields (excluding required and _internal).
    let mut core_names = schemas::core_property_names();
    core_names.sort();
    for name in &core_names {
        if required.iter().any(|r| r == name) || name == "_internal" {
            continue;
        }
        if obj.contains_key(name) {
            continue;
        }
        let (prefix, description) = core_field_metadata(name);
        info.push(ValidationInfo::new(
            name.clone(),
            format!("{prefix}: {description}"),
        ));
    }

    // Type schema fields.
    if let Some(type_schema) = registry().type_schema(category, type_name) {
        let mut seen: BTreeSet<String> = BTreeSet::new();
        collect_type_optional_fields(type_schema, &BTreeSet::new(), &mut info, obj, &mut seen);
    }

    info
}

fn collect_type_optional_fields(
    schema: &Value,
    accumulated_required: &BTreeSet<String>,
    out: &mut Vec<ValidationInfo>,
    data: &Map<String, Value>,
    seen: &mut BTreeSet<String>,
) {
    let core_fields: BTreeSet<&str> = CORE_FIELDS.iter().copied().collect();
    let skip = ["category", "type", "_internal"];

    let mut effective_required: BTreeSet<String> = accumulated_required.clone();
    if let Some(Value::Array(arr)) = schema.get("required") {
        for v in arr {
            if let Some(s) = v.as_str() {
                effective_required.insert(s.to_string());
            }
        }
    }

    if let Some(Value::Object(props)) = schema.get("properties") {
        for (k, v) in props {
            if core_fields.contains(k.as_str()) || skip.contains(&k.as_str()) {
                continue;
            }
            if effective_required.contains(k) {
                continue;
            }
            if data.contains_key(k) || seen.contains(k) {
                continue;
            }
            let description = v
                .get("description")
                .and_then(Value::as_str)
                .map(String::from)
                .unwrap_or_else(|| format!("Optional field: {k}"));
            let recommended = v.get("x-recommended") == Some(&Value::Bool(true));
            let prefix = if recommended { "RECOMMENDED" } else { "OPTIONAL" };
            seen.insert(k.clone());
            out.push(ValidationInfo::new(
                k.clone(),
                format!("{prefix}: {description}"),
            ));
        }
    }

    if let Some(Value::Array(all_of)) = schema.get("allOf") {
        for sub in all_of {
            if let Some(Value::String(href)) = sub.get("$ref") {
                if !href.contains("-base.json") {
                    continue;
                }
                // Only `content-base.json` exists today.
                if href.contains("content-base.json") {
                    if let Some(base) = try_get_base_schema("content-base") {
                        collect_type_optional_fields(
                            &base,
                            &effective_required,
                            out,
                            data,
                            seen,
                        );
                    }
                }
            } else {
                collect_type_optional_fields(sub, &effective_required, out, data, seen);
            }
        }
    }
}

fn core_field_metadata(name: &str) -> (&'static str, String) {
    let schema = registry().core_schema();
    let default_msg = format!("Optional field: {name}");
    let Some(props) = schema.get("properties").and_then(Value::as_object) else {
        return ("OPTIONAL", default_msg);
    };
    let Some(field) = props.get(name).and_then(Value::as_object) else {
        return ("OPTIONAL", default_msg);
    };
    let recommended = field.get("x-recommended") == Some(&Value::Bool(true));
    let description = field
        .get("description")
        .and_then(Value::as_str)
        .map(String::from)
        .unwrap_or(default_msg);
    let prefix = if recommended { "RECOMMENDED" } else { "OPTIONAL" };
    (prefix, description)
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

#[allow(dead_code)]
fn _used(_x: XarfError) {}
