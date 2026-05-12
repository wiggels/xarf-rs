//! # xarf — XARF v4 parser, validator, and generator
//!
//! The [eXtended Abuse Reporting Format](https://xarf.org/) (XARF) is a JSON
//! schema for describing abuse incidents — spam, DDoS, phishing, compromised
//! infrastructure, copyright violations, and so on. This crate is a Rust
//! implementation of the v4 spec.
//!
//! ## Quick start
//!
//! ```rust,no_run
//! use xarf::{parse, Contact, ReportBuilder, create_evidence};
//! use serde_json::json;
//!
//! // Parse incoming JSON.
//! let result = parse(r#"{"xarf_version": "4.2.0", ... }"#).unwrap();
//! if result.errors.is_empty() {
//!     println!("category = {}", result.report.unwrap().category.as_str());
//! }
//!
//! // Build a new report programmatically.
//! let evidence = create_evidence("text/plain", b"log line");
//! let result = ReportBuilder::new("messaging", "spam", "192.0.2.1")
//!     .reporter(Contact::new("Acme", "abuse@acme.example", "acme.example"))
//!     .sender(Contact::new("Acme", "abuse@acme.example", "acme.example"))
//!     .extra("protocol", json!("smtp"))
//!     .extra("smtp_from", json!("spam@bad.example"))
//!     .add_evidence(evidence)
//!     .build()
//!     .unwrap();
//! ```
//!
//! ## Modules
//!
//! - [`model`] — the [`Report`], [`Contact`], [`Evidence`], and [`Category`]
//!   types that make up the typed view.
//! - [`parser`] — [`parse`] and [`parse_value`] for ingesting JSON.
//! - [`generator`] — [`ReportBuilder`], [`create_report`], [`create_evidence`]
//!   for emitting new reports.
//! - [`validator`] — schema validation primitives behind the higher-level
//!   entry points.
//! - [`v3_compat`] — automatic XARF v3 → v4 conversion.

#![doc(html_root_url = "https://docs.rs/xarf-rs/0.1.0")]
#![warn(missing_debug_implementations)]

pub mod error;
pub mod generator;
mod hex;
pub mod model;
pub mod parser;
pub mod schemas;
pub mod v3_compat;
pub mod validator;

// ---------------------------------------------------------------------------
// Public re-exports — the convenience API.
// ---------------------------------------------------------------------------

pub use error::{Result, ValidationError, ValidationInfo, ValidationWarning, XarfError};
pub use generator::{
    CreateReportOptions, EvidenceOptions, HashAlgorithm, ReportBuilder, SPEC_VERSION,
    create_evidence, create_evidence_with_options, create_report,
};
pub use model::{Category, Contact, Evidence, Report};
pub use parser::{ParseOptions, ParseResult, parse, parse_value, parse_with_options};
pub use v3_compat::{convert_v3_to_v4, deprecation_warning, is_v3_report};
pub use validator::{ValidateOptions, ValidationResult, quick_errors, validate};
