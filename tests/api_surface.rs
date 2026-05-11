//! Sanity checks on the public API surface — proves the crate re-exports
//! everything callers need from the root namespace, and that the headline
//! examples in the docs compile and run.

use serde_json::json;

#[test]
fn root_namespace_exposes_canonical_types() {
    // If any of these resolves fail to compile, the README is also broken.
    let _: xarf::Report;
    let _: xarf::Contact;
    let _: xarf::Evidence;
    let _: xarf::Category;
    let _: xarf::ParseResult;
    let _: xarf::ValidationResult;
    let _: xarf::ValidationError;
    let _: xarf::ValidationWarning;
    let _: xarf::ValidationInfo;
    let _: xarf::XarfError;
    let _: xarf::HashAlgorithm;
    let _: xarf::EvidenceOptions;
    let _: xarf::CreateReportOptions;
    let _: xarf::ParseOptions;
    let _: xarf::ValidateOptions;
    let _: xarf::ReportBuilder;
}

#[test]
fn doc_example_from_readme_compiles_and_runs() {
    let evidence = xarf::create_evidence("text/plain", b"log line");
    let result = xarf::ReportBuilder::new("messaging", "spam", "192.0.2.1")
        .reporter(xarf::Contact::new(
            "Acme",
            "abuse@acme.example",
            "acme.example",
        ))
        .sender(xarf::Contact::new(
            "Acme",
            "abuse@acme.example",
            "acme.example",
        ))
        .source_port(25)
        .extra("protocol", json!("smtp"))
        .extra("smtp_from", json!("spam@bad.example"))
        .add_evidence(evidence)
        .build()
        .unwrap();
    assert!(result.errors.is_empty(), "errors: {:?}", result.errors);
    assert!(result.report.is_some());
}

#[test]
fn quick_errors_returns_just_errors() {
    let value = json!({"xarf_version": "4.2.0"});
    let errors = xarf::quick_errors(&value, false).unwrap();
    assert!(!errors.is_empty());
}

#[test]
fn spec_version_constant_is_4_2_0() {
    assert_eq!(xarf::SPEC_VERSION, "4.2.0");
}
