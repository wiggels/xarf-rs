//! Embedded XARF v4 JSON schemas and a lazily-compiled validator registry.
//!
//! The schemas in `schemas/v4/` are bundled into the binary via `include_str!`
//! so consumers of the crate need no I/O. They are parsed once, into
//! [`serde_json::Value`]s, and then on-demand compiled into
//! `jsonschema::Validator` instances keyed by `category/type`.

use std::collections::HashMap;
use std::sync::{Arc, LazyLock, OnceLock};

use jsonschema::{Retrieve, Uri, Validator};
use serde_json::Value;

use crate::error::XarfError;

const CORE_SCHEMA_URI: &str = "https://xarf.org/schemas/v4/xarf-core.json";
const CORE_SCHEMA_JSON: &str = include_str!("../schemas/v4/xarf-core.json");

const MASTER_SCHEMA_URI: &str = "https://xarf.org/schemas/v4/xarf-v4-master.json";
const MASTER_SCHEMA_JSON: &str = include_str!("../schemas/v4/xarf-v4-master.json");

// ---------------------------------------------------------------------------
// Type schemas — every (category, type) combination XARF v4 defines.
// File names use `_` for category subtypes (e.g. `content-brand_infringement`)
// EXCEPT for a handful that use `-` (`copyright-link-site`, `copyright-ugc-platform`,
// `infrastructure-compromised-server`, `connection-login-attack`,
// `connection-port-scan`, `connection-sql-injection`, `connection-vulnerability-scan`,
// `connection-infected-host`, `vulnerability-open-service`,
// `reputation-threat-intelligence`).
// ---------------------------------------------------------------------------

macro_rules! type_schemas {
    ( $( ($category:literal, $type_name:literal, $file:literal) ),* $(,)? ) => {
        const TYPE_SCHEMA_SOURCES: &[(&str, &str, &str, &str)] = &[
            $(
                (
                    $category,
                    $type_name,
                    concat!("https://xarf.org/schemas/v4/types/", $file, ".json"),
                    include_str!(concat!("../schemas/v4/types/", $file, ".json")),
                ),
            )*
        ];
    };
}

type_schemas! {
    // messaging
    ("messaging",     "spam",                     "messaging-spam"),
    ("messaging",     "bulk_messaging",           "messaging-bulk-messaging"),
    // connection
    ("connection",    "login_attack",             "connection-login-attack"),
    ("connection",    "port_scan",                "connection-port-scan"),
    ("connection",    "ddos",                     "connection-ddos"),
    ("connection",    "scraping",                 "connection-scraping"),
    ("connection",    "sql_injection",            "connection-sql-injection"),
    ("connection",    "vulnerability_scan",       "connection-vulnerability-scan"),
    ("connection",    "infected_host",            "connection-infected-host"),
    ("connection",    "reconnaissance",           "connection-reconnaissance"),
    // content
    ("content",       "phishing",                 "content-phishing"),
    ("content",       "malware",                  "content-malware"),
    ("content",       "fraud",                    "content-fraud"),
    ("content",       "csam",                     "content-csam"),
    ("content",       "csem",                     "content-csem"),
    ("content",       "exposed_data",             "content-exposed-data"),
    ("content",       "brand_infringement",       "content-brand_infringement"),
    ("content",       "suspicious_registration",  "content-suspicious_registration"),
    ("content",       "remote_compromise",        "content-remote_compromise"),
    // copyright
    ("copyright",     "copyright",                "copyright-copyright"),
    ("copyright",     "cyberlocker",              "copyright-cyberlocker"),
    ("copyright",     "link_site",                "copyright-link-site"),
    ("copyright",     "p2p",                      "copyright-p2p"),
    ("copyright",     "usenet",                   "copyright-usenet"),
    ("copyright",     "ugc_platform",             "copyright-ugc-platform"),
    // vulnerability
    ("vulnerability", "cve",                      "vulnerability-cve"),
    ("vulnerability", "misconfiguration",         "vulnerability-misconfiguration"),
    ("vulnerability", "open_service",             "vulnerability-open-service"),
    // infrastructure
    ("infrastructure", "botnet",                  "infrastructure-botnet"),
    ("infrastructure", "compromised_server",      "infrastructure-compromised-server"),
    // reputation
    ("reputation",    "blocklist",                "reputation-blocklist"),
    ("reputation",    "threat_intelligence",      "reputation-threat-intelligence"),
}

// content-base.json is referenced indirectly by all 9 content types via
// `"$ref": "./content-base.json"`. It is not itself a leaf type schema but
// must be resolvable by the retriever.
const CONTENT_BASE_URI: &str = "https://xarf.org/schemas/v4/types/content-base.json";
const CONTENT_BASE_JSON: &str = include_str!("../schemas/v4/types/content-base.json");

// ---------------------------------------------------------------------------
// Registry
// ---------------------------------------------------------------------------

/// Static schema retriever: hands the `jsonschema` engine the embedded JSON
/// for any `$ref` URI we know about. Cheap to clone (one `Arc` bump).
#[derive(Clone, Debug)]
struct StaticRetriever {
    documents: Arc<HashMap<String, Value>>,
}

impl Retrieve for StaticRetriever {
    fn retrieve(
        &self,
        uri: &Uri<String>,
    ) -> std::result::Result<Value, Box<dyn std::error::Error + Send + Sync>> {
        let key = uri.as_str();
        self.documents
            .get(key)
            .cloned()
            .ok_or_else(|| format!("unknown schema reference: {key}").into())
    }
}

/// Description + recommendation flag for one field, surfaced by
/// `show_missing_optional` validation.
#[derive(Debug, Clone)]
pub struct FieldMeta {
    pub name: String,
    pub description: String,
    pub recommended: bool,
}

/// One row in the type-schema table. Stored in a slice sorted by
/// `(category, type_name)` so lookups are a zero-alloc binary search.
#[derive(Debug)]
struct TypeSchemaEntry {
    category: &'static str,
    type_name: &'static str,
    parsed: Value,
    /// Field names defined by this type (own + base + core), sorted, for
    /// known-field membership tests during validation.
    known_fields: Box<[String]>,
    /// Optional (i.e. non-required, non-core) fields with description +
    /// recommendation flag, in schema-declaration order.
    optional_fields: Box<[FieldMeta]>,
}

/// Parsed schema bodies indexed by `category/type` and by URI. Cheap to clone
/// (it's behind an `Arc`); compilation is on-demand so we only pay for what we
/// actually validate against.
#[derive(Debug)]
pub struct SchemaRegistry {
    core_schema: Value,
    master_schema: Value,
    master_schema_strict: Value,
    /// Property names of the core schema (`xarf_version`, etc), in iteration order.
    core_property_names: Box<[String]>,
    /// Required-field names of the core schema.
    core_required: Box<[String]>,
    /// Optional core fields with description + recommendation flag.
    core_optional: Box<[FieldMeta]>,
    type_schemas: Box<[TypeSchemaEntry]>,
    retriever: StaticRetriever,
    retriever_strict: StaticRetriever,
    /// Cached compiled master validator (normal mode). Compiled lazily on
    /// first use; reused for every subsequent `validate()` / `parse()` call.
    /// Caching shaves ~2ms off every parse on commodity hardware — most of
    /// the previous wall-clock cost.
    master_validator: OnceLock<Validator>,
    /// Cached compiled master validator (strict mode).
    master_validator_strict: OnceLock<Validator>,
}

impl SchemaRegistry {
    fn build() -> Result<Self, XarfError> {
        let core_schema: Value = serde_json::from_str(CORE_SCHEMA_JSON)
            .map_err(|e| XarfError::Schema(format!("core schema parse: {e}")))?;
        let master_schema: Value = serde_json::from_str(MASTER_SCHEMA_JSON)
            .map_err(|e| XarfError::Schema(format!("master schema parse: {e}")))?;
        let content_base: Value = serde_json::from_str(CONTENT_BASE_JSON)
            .map_err(|e| XarfError::Schema(format!("content-base parse: {e}")))?;

        let core_property_names: Box<[String]> = core_schema
            .get("properties")
            .and_then(Value::as_object)
            .map(|o| o.keys().cloned().collect())
            .unwrap_or_default();
        let core_required: Box<[String]> = core_schema
            .get("required")
            .and_then(Value::as_array)
            .map(|a| a.iter().filter_map(|v| v.as_str().map(String::from)).collect())
            .unwrap_or_default();
        let core_optional = build_core_optional(&core_schema, &core_required);

        let mut type_schemas: Vec<TypeSchemaEntry> = Vec::with_capacity(TYPE_SCHEMA_SOURCES.len());
        let mut documents = HashMap::new();
        documents.insert(CORE_SCHEMA_URI.to_string(), core_schema.clone());
        documents.insert(MASTER_SCHEMA_URI.to_string(), master_schema.clone());
        documents.insert(CONTENT_BASE_URI.to_string(), content_base.clone());

        for (category, type_name, uri, source) in TYPE_SCHEMA_SOURCES {
            let parsed: Value = serde_json::from_str(source).map_err(|e| {
                XarfError::Schema(format!(
                    "type schema parse failed for {category}/{type_name}: {e}"
                ))
            })?;
            documents.insert(uri.to_string(), parsed.clone());
            let known_fields = build_known_fields(&parsed, &content_base);
            let optional_fields = build_optional_fields(&parsed, &content_base);
            type_schemas.push(TypeSchemaEntry {
                category,
                type_name,
                parsed,
                known_fields,
                optional_fields,
            });
        }
        type_schemas.sort_by(|a, b| (a.category, a.type_name).cmp(&(b.category, b.type_name)));
        let type_schemas = type_schemas.into_boxed_slice();

        // Build strict variants: deep-copy each document and promote any
        // `x-recommended: true` property into its parent `required` array.
        let mut master_schema_strict = master_schema.clone();
        promote_recommended_to_required(&mut master_schema_strict);

        let mut documents_strict: HashMap<String, Value> = documents
            .iter()
            .map(|(k, v)| {
                let mut clone = v.clone();
                promote_recommended_to_required(&mut clone);
                (k.clone(), clone)
            })
            .collect();
        // The master schema URI in the strict retriever should also point to
        // the promoted version (its `if/then` blocks reference type schemas
        // by URI, which the retriever resolves).
        documents_strict.insert(MASTER_SCHEMA_URI.to_string(), master_schema_strict.clone());

        Ok(Self {
            core_schema,
            master_schema,
            master_schema_strict,
            core_property_names,
            core_required,
            core_optional,
            type_schemas,
            retriever: StaticRetriever {
                documents: Arc::new(documents),
            },
            retriever_strict: StaticRetriever {
                documents: Arc::new(documents_strict),
            },
            master_validator: OnceLock::new(),
            master_validator_strict: OnceLock::new(),
        })
    }

    /// Return the (parsed) core schema JSON for property inspection.
    pub fn core_schema(&self) -> &Value {
        &self.core_schema
    }

    /// Return the (parsed) master schema JSON.
    pub fn master_schema(&self) -> &Value {
        &self.master_schema
    }

    /// Return the (parsed) type schema for `(category, type)`, if any.
    pub fn type_schema(&self, category: &str, type_name: &str) -> Option<&Value> {
        self.find_entry(category, type_name).map(|e| &e.parsed)
    }

    fn find_entry(&self, category: &str, type_name: &str) -> Option<&TypeSchemaEntry> {
        self.type_schemas
            .binary_search_by(|e| (e.category, e.type_name).cmp(&(category, type_name)))
            .ok()
            .map(|i| &self.type_schemas[i])
    }

    /// Borrow the (lazily-compiled, then cached) master validator. The first
    /// call per `strict` flavour compiles the validator (~2ms on commodity
    /// hardware); every subsequent call is a pointer dereference.
    ///
    /// The compiled `jsonschema::Validator` is fully thread-safe — concurrent
    /// `validate()` calls from many tasks share it without contention.
    pub fn master_validator(&self, strict: bool) -> Result<&Validator, XarfError> {
        let cell = if strict {
            &self.master_validator_strict
        } else {
            &self.master_validator
        };
        if let Some(v) = cell.get() {
            return Ok(v);
        }
        let compiled = if strict {
            self.compile(&self.master_schema_strict, true)?
        } else {
            self.compile(&self.master_schema, false)?
        };
        // `set` may race with another thread that already filled the cell;
        // in that case discard our value and return the winner's. Either way
        // every caller sees the same compiled validator.
        Ok(cell.get_or_init(|| compiled))
    }

    /// Compile a `jsonschema::Validator` for the core schema.
    pub fn core_validator(&self) -> Result<Validator, XarfError> {
        self.compile(&self.core_schema, false)
    }

    /// Compile a `jsonschema::Validator` for `(category, type)` if defined.
    pub fn type_validator(
        &self,
        category: &str,
        type_name: &str,
    ) -> Result<Option<Validator>, XarfError> {
        let Some(schema) = self.type_schema(category, type_name) else {
            return Ok(None);
        };
        self.compile(schema, false).map(Some)
    }

    fn compile(&self, schema: &Value, strict: bool) -> Result<Validator, XarfError> {
        let retriever = if strict {
            self.retriever_strict.clone()
        } else {
            self.retriever.clone()
        };
        // Enable format validation so `uuid`, `email`, `hostname`, `date-time`,
        // and `uri` annotations are enforced rather than treated as hints.
        // The XARF v4 spec relies on these to catch malformed report_ids and
        // contact emails, matching the Python reference implementation which
        // wires up `jsonschema.FormatChecker()`.
        jsonschema::options()
            .with_retriever(retriever)
            .should_validate_formats(true)
            .build(schema)
            .map_err(|e| XarfError::Schema(format!("schema compile: {e}")))
    }

    /// All `(category, type)` combinations supported by the spec, sorted.
    pub fn known_combinations(&self) -> impl Iterator<Item = (&'static str, &'static str)> + '_ {
        self.type_schemas.iter().map(|e| (e.category, e.type_name))
    }

    /// Returns `true` iff `(category, type)` is one of the 32 defined combos.
    pub fn is_known_combination(&self, category: &str, type_name: &str) -> bool {
        self.find_entry(category, type_name).is_some()
    }

    /// Sorted list of every field name defined by the type schema for
    /// `(category, type)` and any base schemas it `$ref`s. Returns `None`
    /// for unknown combinations.
    pub fn type_known_fields(&self, category: &str, type_name: &str) -> Option<&[String]> {
        self.find_entry(category, type_name)
            .map(|e| e.known_fields.as_ref())
    }

    /// Optional/recommended fields declared by the type schema for
    /// `(category, type)` with description + recommendation flag, in
    /// schema-declaration order.
    pub fn type_optional_fields(&self, category: &str, type_name: &str) -> Option<&[FieldMeta]> {
        self.find_entry(category, type_name)
            .map(|e| e.optional_fields.as_ref())
    }

    /// Core schema property names, in iteration order.
    pub fn core_property_names_slice(&self) -> &[String] {
        &self.core_property_names
    }

    /// Core schema required-field names.
    pub fn core_required_slice(&self) -> &[String] {
        &self.core_required
    }

    /// Core schema optional fields with description + recommendation flag.
    pub fn core_optional_fields(&self) -> &[FieldMeta] {
        &self.core_optional
    }
}

/// Process-wide schema registry built lazily on first access.
pub fn registry() -> &'static SchemaRegistry {
    static REGISTRY: LazyLock<SchemaRegistry> =
        LazyLock::new(|| SchemaRegistry::build().expect("bundled XARF schemas must parse"));
    &REGISTRY
}

// ---------------------------------------------------------------------------
// Helpers for field-metadata extraction
// ---------------------------------------------------------------------------

/// Property names defined directly on the core schema (excludes `$defs`).
pub fn core_property_names() -> Vec<String> {
    registry().core_property_names_slice().to_vec()
}

/// Walk a JSON-Schema node and, for any `properties` object, add to its
/// parent's `required` array every property whose value carries
/// `"x-recommended": true`. Recurses into all standard schema keywords.
///
/// This is the strict-mode transform described in the XARF v4 implementer's
/// guide and is identical to the Python `_promote_recommended_to_required`.
pub(crate) fn promote_recommended_to_required(node: &mut Value) {
    let Value::Object(map) = node else {
        return;
    };

    // Promote x-recommended properties on this node into `required`.
    let mut to_add: Vec<String> = Vec::new();
    if let Some(Value::Object(props)) = map.get("properties") {
        for (k, v) in props {
            if let Value::Object(pmap) = v {
                if pmap.get("x-recommended") == Some(&Value::Bool(true)) {
                    to_add.push(k.clone());
                }
            }
        }
    }
    if !to_add.is_empty() {
        let mut existing: Vec<String> = match map.get("required") {
            Some(Value::Array(arr)) => arr
                .iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect(),
            _ => Vec::new(),
        };
        for name in to_add {
            if !existing.contains(&name) {
                existing.push(name);
            }
        }
        map.insert(
            "required".to_string(),
            Value::Array(existing.into_iter().map(Value::String).collect()),
        );
    }

    // Recurse into object-valued sub-schemas.
    for key in ["properties", "$defs"] {
        if let Some(Value::Object(sub)) = map.get_mut(key) {
            for value in sub.values_mut() {
                promote_recommended_to_required(value);
            }
        }
    }

    // Recurse into list-valued schema combinators.
    for key in ["allOf", "anyOf", "oneOf"] {
        if let Some(Value::Array(arr)) = map.get_mut(key) {
            for item in arr.iter_mut() {
                promote_recommended_to_required(item);
            }
        }
    }

    // Recurse into single-schema keywords.
    for key in ["items", "if", "then", "else", "not", "additionalProperties"] {
        if let Some(child) = map.get_mut(key) {
            if child.is_object() {
                promote_recommended_to_required(child);
            }
        }
    }
}

/// Required property names from the core schema's top-level `required` array.
pub fn core_required_fields() -> Vec<String> {
    registry().core_required_slice().to_vec()
}

// ---------------------------------------------------------------------------
// Field extraction helpers (called once at registry build).
// ---------------------------------------------------------------------------

const CORE_FIELD_NAMES: &[&str] = &[
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

/// Build the sorted, deduplicated set of field names declared by `schema`
/// (including names pulled in from `content-base` via `$ref`).
fn build_known_fields(schema: &Value, content_base: &Value) -> Box<[String]> {
    let mut acc: Vec<String> = Vec::new();
    collect_property_names(schema, content_base, &mut acc);
    for &name in CORE_FIELD_NAMES {
        if !acc.iter().any(|n| n == name) {
            acc.push(name.to_string());
        }
    }
    acc.sort();
    acc.into_boxed_slice()
}

fn collect_property_names(schema: &Value, content_base: &Value, out: &mut Vec<String>) {
    if let Some(Value::Object(props)) = schema.get("properties") {
        for k in props.keys() {
            if !out.iter().any(|n| n == k) {
                out.push(k.clone());
            }
        }
    }
    if let Some(Value::Array(all_of)) = schema.get("allOf") {
        for sub in all_of {
            if let Some(Value::String(href)) = sub.get("$ref") {
                if href.contains("../xarf-core.json") {
                    continue;
                }
                if href.contains("content-base.json") {
                    collect_property_names(content_base, content_base, out);
                }
                continue;
            }
            collect_property_names(sub, content_base, out);
        }
    }
}

fn build_core_optional(core_schema: &Value, required: &[String]) -> Box<[FieldMeta]> {
    let Some(props) = core_schema.get("properties").and_then(Value::as_object) else {
        return Box::default();
    };
    let mut out: Vec<FieldMeta> = props
        .iter()
        .filter_map(|(name, v)| {
            if required.iter().any(|r| r == name) || name == "_internal" {
                return None;
            }
            let pmap = v.as_object()?;
            let recommended = pmap.get("x-recommended") == Some(&Value::Bool(true));
            let description = pmap
                .get("description")
                .and_then(Value::as_str)
                .map(String::from)
                .unwrap_or_else(|| format!("Optional field: {name}"));
            Some(FieldMeta {
                name: name.clone(),
                description,
                recommended,
            })
        })
        .collect();
    out.sort_by(|a, b| a.name.cmp(&b.name));
    out.into_boxed_slice()
}

fn build_optional_fields(schema: &Value, content_base: &Value) -> Box<[FieldMeta]> {
    use std::collections::BTreeSet;
    let mut out: Vec<FieldMeta> = Vec::new();
    let mut seen: BTreeSet<String> = BTreeSet::new();
    collect_type_optional(
        schema,
        content_base,
        &BTreeSet::new(),
        &mut out,
        &mut seen,
    );
    out.into_boxed_slice()
}

fn collect_type_optional(
    schema: &Value,
    content_base: &Value,
    accumulated_required: &std::collections::BTreeSet<String>,
    out: &mut Vec<FieldMeta>,
    seen: &mut std::collections::BTreeSet<String>,
) {
    use std::collections::BTreeSet;

    let core: BTreeSet<&str> = CORE_FIELD_NAMES.iter().copied().collect();
    const SKIP: &[&str] = &["category", "type", "_internal"];

    let mut required = accumulated_required.clone();
    if let Some(Value::Array(arr)) = schema.get("required") {
        for v in arr {
            if let Some(s) = v.as_str() {
                required.insert(s.to_string());
            }
        }
    }

    if let Some(Value::Object(props)) = schema.get("properties") {
        for (k, v) in props {
            if core.contains(k.as_str()) || SKIP.iter().any(|s| *s == k.as_str()) {
                continue;
            }
            if required.contains(k) || !seen.insert(k.clone()) {
                continue;
            }
            let description = v
                .get("description")
                .and_then(Value::as_str)
                .map(String::from)
                .unwrap_or_else(|| format!("Optional field: {k}"));
            let recommended = v.get("x-recommended") == Some(&Value::Bool(true));
            out.push(FieldMeta {
                name: k.clone(),
                description,
                recommended,
            });
        }
    }

    if let Some(Value::Array(all_of)) = schema.get("allOf") {
        for sub in all_of {
            if let Some(Value::String(href)) = sub.get("$ref") {
                if !href.contains("-base.json") {
                    continue;
                }
                if href.contains("content-base.json") {
                    collect_type_optional(content_base, content_base, &required, out, seen);
                }
            } else {
                collect_type_optional(sub, content_base, &required, out, seen);
            }
        }
    }
}
