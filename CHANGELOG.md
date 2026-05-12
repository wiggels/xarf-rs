# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.4](https://github.com/wiggels/xarf-rs/compare/v0.1.3...v0.1.4) - 2026-05-12

### Changed

- Deprecation_warning returns &'static str
- Replace once_cell::Lazy with std::sync::LazyLock

### Fixed

- Satisfy rustfmt, clippy, and rustdoc lints

### Performance

- Eliminate unnecessary clones in v3 conversion path
- Cache per-type field metadata in schema registry
- Zero-alloc schema registry lookups
- Vectorize hex encoding for evidence hashing

## [0.1.3](https://github.com/wiggels/xarf-rs/compare/v0.1.2...v0.1.3) - 2026-05-12

### Fixed

- Raise bench alert threshold to 125% for shared runner noise

## [0.1.2](https://github.com/wiggels/xarf-rs/compare/v0.1.1...v0.1.2) - 2026-05-11

### Fixed

- Bootstrap gh-pages
- Remove windows ci job
- Update github workflows

## [0.1.1](https://github.com/wiggels/xarf-rs/compare/v0.1.0...v0.1.1) - 2026-05-11

### Fixed

- Rustfmt lints
- Edition version to 2024

## [0.1.0] - 2026-05-11

### Added

- Initial release. XARF v4 parser, validator, and generator with v3
  backward compatibility.
- Core API: `parse`, `parse_value`, `parse_with_options`, `validate`,
  `ReportBuilder`, `create_report`, `create_evidence`, `convert_v3_to_v4`,
  `is_v3_report`.
- Typed model: `Report`, `Contact`, `Evidence`, `Category` — with a
  forward-compatible `BTreeMap` for category-specific extras.
- Validation modes: standard, strict (recommended → required), and
  `show_missing_optional` for review tools.
- 34 bundled JSON schemas (core + master + 33 type-specific), embedded
  via `include_str!` — no runtime I/O.
- 127 tests across 11 binaries: parser/validator/generator unit tests,
  insta snapshot tests, async-compat / Send+Sync compile-time assertions,
  golden tests against the shared `xarf-parser-tests` corpus, and
  committed wall-clock perf budgets.
- 33 criterion benchmarks across 7 groups (parse, validate, generate,
  create_evidence, v3, registry, round_trip).
- CI workflows: test matrix (Linux/macOS/Windows × stable/beta), MSRV
  (1.86, edition 2024), clippy, rustfmt, cargo-deny, cargo-audit,
  cargo-semver-checks, release-plz, benchmark regression detection,
  llvm-cov coverage.

[Unreleased]: https://github.com/wiggels/xarf-rs/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/wiggels/xarf-rs/releases/tag/v0.1.0
