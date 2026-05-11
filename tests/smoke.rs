//! Quick smoke check that the basic parse → validate → typed-report path
//! works against one canonical sample. Helps catch the catastrophic case
//! where the bundled schemas don't load or category lookup is broken.

use std::fs;

#[test]
fn parses_messaging_spam_sample() {
    let path = "tests/parser_test_samples/valid/v4/messaging/spam_user_complaint_sample.json";
    let json = fs::read_to_string(path).unwrap();
    let result = xarf::parse(&json).unwrap();
    assert!(
        result.errors.is_empty(),
        "expected no errors, got: {:?}",
        result.errors
    );
    let report = result.report.unwrap();
    assert_eq!(report.category.as_str(), "messaging");
    assert_eq!(report.type_, "spam");
}

#[test]
fn detects_missing_xarf_version() {
    let path = "tests/parser_test_samples/invalid/schema_violations/missing_xarf_version.json";
    let json = fs::read_to_string(path).unwrap();
    let result = xarf::parse(&json).unwrap();
    assert!(
        !result.errors.is_empty(),
        "expected validation errors for missing xarf_version"
    );
}
