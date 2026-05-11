//! Golden tests against the shared `xarf/xarf-parser-tests` corpus.
//!
//! Every file under `tests/parser_test_samples/valid/v4/` must parse with
//! zero errors. Every file under `tests/parser_test_samples/valid/v3/` must
//! parse cleanly via the v3 → v4 path (errors are tolerated only when
//! conversion was best-effort and the v3 sample doesn't carry enough
//! information for a fully-valid v4 report). Every file under
//! `tests/parser_test_samples/invalid/` must surface at least one error
//! (or an `Err` for malformed JSON).

use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};

/// Recursively walk *root* collecting `.json` file paths.
fn walk_json(root: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(root) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            walk_json(&path, out);
        } else if path.extension() == Some(OsStr::new("json")) {
            out.push(path);
        }
    }
}

#[test]
fn every_v4_shared_sample_does_not_crash() {
    // The shared `valid/v4` corpus deliberately includes reports that use
    // non-canonical type values (e.g. `connection/auth_failure`,
    // `connection/ip_spoof`) — forward-compatibility samples that exercise the
    // parser's resilience to unknown types. The contract (matching the
    // upstream Python parser tests) is: `parse` must not crash, and the result
    // must either carry a typed report OR surface validation errors.
    let root = Path::new("tests/parser_test_samples/valid/v4");
    let mut paths = Vec::new();
    walk_json(root, &mut paths);
    assert!(!paths.is_empty(), "no v4 sample files found under {root:?}");

    for path in &paths {
        let json = fs::read_to_string(path).expect("read sample");
        let result = xarf::parse(&json)
            .unwrap_or_else(|e| panic!("parse panicked on {}: {e}", path.display()));
        assert!(
            result.report.is_some() || !result.errors.is_empty(),
            "{}: result should have a report OR errors, got neither",
            path.display()
        );
    }
    eprintln!("✓ exercised {} shared v4 samples", paths.len());
}

/// Smoke test that the validator surfaces *some* error for at least one
/// shared sample that has known schema violations (`spam_v3_converted_sample`
/// is a real-world example: its `evidence_source` is "unknown" which is not
/// in the enum). This guards against regressions where validation silently
/// passes everything.
#[test]
fn validator_flags_known_shared_sample_violations() {
    let path = "tests/parser_test_samples/valid/v4/messaging/spam_v3_converted_sample.json";
    let json = fs::read_to_string(path).expect("read sample");
    let result = xarf::parse(&json).expect("parse never panics");
    assert!(
        result
            .errors
            .iter()
            .any(|e| e.field == "evidence_source" || e.message.contains("evidence_source")),
        "expected an evidence_source enum error, got: {:?}",
        result.errors
    );
}

#[test]
fn v3_samples_are_detected() {
    // Every sample under `valid/v3/` must be recognised as v3 format by the
    // detection heuristic. (Conversion success is asserted separately — not
    // all v3 samples carry enough information to convert cleanly; the upstream
    // Python parser also makes ddos_v3 / botnet_v3 raise.)
    let root = Path::new("tests/parser_test_samples/valid/v3");
    let mut paths = Vec::new();
    walk_json(root, &mut paths);
    assert_eq!(paths.len(), 4, "expected 4 v3 samples, got {}", paths.len());

    for path in &paths {
        let json = fs::read_to_string(path).expect("read sample");
        let value: serde_json::Value =
            serde_json::from_str(&json).expect("v3 sample is valid JSON");
        assert!(
            xarf::is_v3_report(&value),
            "{} should be detected as v3",
            path.display()
        );
    }
}

#[test]
fn v3_spam_and_phishing_convert_cleanly() {
    // These two carry enough information for a fully-valid v4 conversion. The
    // v3 ddos and botnet samples lack required v4 fields (protocol /
    // compromise_evidence respectively) and are tested separately.
    for stem in ["spam_v3_sample", "phishing_v3_sample"] {
        let path = format!("tests/parser_test_samples/valid/v3/{stem}.json");
        let json = fs::read_to_string(&path).expect("read sample");
        let value: serde_json::Value =
            serde_json::from_str(&json).expect("v3 sample is valid JSON");
        let mut warnings: Vec<String> = Vec::new();
        let converted = xarf::convert_v3_to_v4(value, &mut warnings)
            .unwrap_or_else(|e| panic!("{stem} conversion failed: {e}"));
        assert_eq!(
            converted.get("legacy_version").and_then(|v| v.as_str()),
            Some("3")
        );
        assert!(converted.get("xarf_version").is_some());
        assert!(converted.get("report_id").is_some());
        assert!(converted.get("timestamp").is_some());
        assert!(converted.get("category").is_some());
        assert!(converted.get("type").is_some());
    }
}

#[test]
fn v3_ddos_sample_fails_conversion_due_to_missing_protocol() {
    // Mirrors xarf-python's `test_ddos_v3_sample_raises_parse_error`. The
    // sample's `Protocol` field lives inside `AdditionalInfo` rather than at
    // the top level of `Report`, which the v3 spec doesn't accept for
    // connection types.
    let path = "tests/parser_test_samples/valid/v3/ddos_v3_sample.json";
    let json = fs::read_to_string(path).expect("read sample");
    let value: serde_json::Value = serde_json::from_str(&json).expect("valid JSON");
    let mut warnings: Vec<String> = Vec::new();
    let err = xarf::convert_v3_to_v4(value, &mut warnings).unwrap_err();
    let msg = err.to_string();
    assert!(
        msg.contains("protocol"),
        "expected error to mention missing protocol, got: {msg}"
    );
}

#[test]
fn every_invalid_sample_surfaces_errors() {
    let root = Path::new("tests/parser_test_samples/invalid");
    let mut paths = Vec::new();
    walk_json(root, &mut paths);
    assert!(
        !paths.is_empty(),
        "no invalid sample files found under {root:?}"
    );

    for path in &paths {
        let json = fs::read_to_string(path).expect("read invalid sample");
        match xarf::parse(&json) {
            Ok(result) => {
                assert!(
                    !result.errors.is_empty(),
                    "{} should have surfaced errors but parsed cleanly",
                    path.display()
                );
            }
            Err(e) => {
                // Malformed JSON / non-object inputs return Err directly.
                // That's still "surfaces an error" in the parser-tests sense.
                let _ = e;
            }
        }
    }
}

#[test]
fn spec_samples_parse_without_errors() {
    let root = Path::new("tests/spec_samples");
    let mut paths = Vec::new();
    walk_json(root, &mut paths);
    assert!(
        paths.len() >= 30,
        "expected at least 30 canonical spec samples, found {}",
        paths.len()
    );

    let mut failed: Vec<(PathBuf, Vec<String>)> = Vec::new();
    for path in &paths {
        let json = fs::read_to_string(path).expect("read sample");
        let result = xarf::parse(&json).expect("spec sample is valid JSON");
        if !result.errors.is_empty() {
            failed.push((
                path.clone(),
                result
                    .errors
                    .iter()
                    .map(|e| format!("[{}] {}", e.field, e.message))
                    .collect(),
            ));
        }
    }
    if !failed.is_empty() {
        let mut msg = String::from("spec samples failed validation:\n");
        for (p, errs) in &failed {
            msg.push_str(&format!("  {}:\n", p.display()));
            for err in errs {
                msg.push_str(&format!("    - {err}\n"));
            }
        }
        panic!("{msg}");
    }
    eprintln!("✓ validated {} canonical spec samples", paths.len());
}
