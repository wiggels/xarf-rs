# xarf-rs

[![CI](https://github.com/wiggels/xarf-rs/actions/workflows/ci.yml/badge.svg)](https://github.com/wiggels/xarf-rs/actions/workflows/ci.yml)
[![Audit](https://github.com/wiggels/xarf-rs/actions/workflows/audit.yml/badge.svg)](https://github.com/wiggels/xarf-rs/actions/workflows/audit.yml)
[![Benchmarks](https://github.com/wiggels/xarf-rs/actions/workflows/bench.yml/badge.svg)](https://github.com/wiggels/xarf-rs/actions/workflows/bench.yml)
[![Coverage](https://codecov.io/gh/wiggels/xarf-rs/branch/main/graph/badge.svg)](https://codecov.io/gh/wiggels/xarf-rs)
[![Crates.io](https://img.shields.io/crates/v/xarf-rs.svg)](https://crates.io/crates/xarf-rs)
[![Docs.rs](https://docs.rs/xarf-rs/badge.svg)](https://docs.rs/xarf-rs)
[![MSRV](https://img.shields.io/badge/rustc-1.86+-blue.svg)](https://blog.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

A Rust implementation of [XARF v4](https://xarf.org/) — the eXtended Abuse
Reporting Format. Parses, validates, and generates abuse reports with a
small, dependency-light API. Compatible with XARF v3 input via automatic
conversion.

```toml
[dependencies]
xarf-rs = "0.1"
```

The crate publishes as `xarf-rs` on crates.io but imports as `xarf`:

```rust
use xarf::{parse, ReportBuilder, Contact, create_evidence};
```

## What XARF is

XARF is a JSON standard for describing abuse incidents (spam, DDoS,
phishing, copyright violations, compromised infrastructure, and so on) in a
machine-readable, schema-validated form. The full v4 specification lives at
[github.com/xarf/xarf-spec](https://github.com/xarf/xarf-spec). This crate
ships the canonical JSON Schemas embedded at compile time — no network or
filesystem access is required at runtime.

## Quick start

```rust
use serde_json::json;
use xarf::{create_evidence, parse, Contact, ReportBuilder};

// Parse an incoming report.
let result = parse(r#"{"xarf_version": "4.2.0", "...": "..."}"#).unwrap();
if result.errors.is_empty() {
    let report = result.report.unwrap();
    println!("{}/{}", report.category.as_str(), report.type_);
}

// Build a new report programmatically.
let evidence = create_evidence("text/plain", b"original spam payload");
let result = ReportBuilder::new("messaging", "spam", "192.0.2.1")
    .reporter(Contact::new("Acme", "abuse@acme.example", "acme.example"))
    .sender(Contact::new("Acme", "abuse@acme.example", "acme.example"))
    .source_port(25)
    .extra("protocol", json!("smtp"))
    .extra("smtp_from", json!("spam@bad.example"))
    .add_evidence(evidence)
    .build()
    .unwrap();
```

## API surface

| Function / type           | Purpose                                                            |
| ------------------------- | ------------------------------------------------------------------ |
| `parse(json)`             | Parse a JSON string. Auto-converts v3 input.                       |
| `parse_value(value, ...)` | Same, but starts from a `serde_json::Value`.                       |
| `validate(value, ...)`    | Validate without typed deserialization.                            |
| `ReportBuilder`           | Build a report programmatically; auto-fills `report_id`/timestamp. |
| `create_report(...)`      | Functional shorthand for `ReportBuilder`.                          |
| `create_evidence(...)`    | Compute hash + base64 + size from raw bytes.                       |
| `is_v3_report(value)`     | Detect XARF v3 format.                                             |
| `convert_v3_to_v4(...)`   | Convert v3 → v4 JSON.                                              |
| `Report`, `Contact`,      |                                                                    |
| `Evidence`, `Category`    | Typed views over a parsed report.                                  |

## Validation modes

* **Standard** (default): required fields enforced; recommended fields
  ignored; unknown fields surface as warnings.
* **Strict** (`ParseOptions { strict: true, .. }`): recommended fields
  promoted to required; unknown fields become errors.
* **Show missing optional** (`show_missing_optional: true`): populate
  `info` with every absent recommended/optional field — useful for review
  tools.

## XARF v3 backward compatibility

`parse()` automatically detects v3 input (`Version: "3"/"3.0"/"3.0.0"` plus
`ReporterInfo` and `Report`), converts it to v4, and surfaces a
deprecation warning. Behavioural parity is held with the reference
[Python implementation](https://github.com/xarf/xarf-python)'s
`v3_compat` module.

## Architecture

* Core fields on `Report` are strongly typed (`Contact`, `Evidence`,
  `Category`); category-specific fields flow through a sorted
  `BTreeMap<String, serde_json::Value>` for lossless round-tripping and
  forward compatibility.
* The 33 type schemas plus the master schema and core schema are bundled
  into the binary with `include_str!`.
* Strict-mode validation deep-copies the master schema once at startup and
  promotes every `x-recommended: true` property to its parent's `required`
  array — matching the algorithm in the v4 implementer's guide.
* Schema reference resolution (`$ref` between type schemas and the core
  schema) is satisfied entirely from the embedded documents through a
  custom `jsonschema::Retrieve` retriever.

## Performance

The hot paths (parse, validate, build) all run at single-digit µs on
commodity hardware, well inside the XARF v4 spec's `<1ms` typical-report
target:

| Operation                       | Time     | Throughput     |
| ------------------------------- | -------- | -------------- |
| `parse` — small spam (~500 B)   | ~9 µs    | ~68 MiB/s      |
| `parse` — DDoS report (~700 B)  | ~8 µs    | ~68 MiB/s      |
| `parse` — large bulk (~40 KB)   | ~23 µs   | ~1.7 GiB/s     |
| `parse` (strict mode)           | ~10 µs   | ~60 MiB/s      |
| `validate` only                 | ~7 µs    | —              |
| `convert_v3_to_v4`              | ~5 µs    | ~100 MiB/s     |
| `is_v3_report` detection        | ~30 ns   | ~16 GiB/s      |
| `create_evidence` (SHA-256, 64 KB) | ~39 µs | ~1.6 GiB/s    |
| `create_evidence` (MD5, 64 KB)  | ~85 µs   | ~730 MiB/s     |

The compiled `jsonschema::Validator` for the master schema is built once
on first use and cached for the lifetime of the process — concurrent
parses from many tasks share a single immutable validator.

### Regression detection

The crate ships with **three layers** of defense against perf regressions:

1. **Committed perf-budget tests** (`tests/perf_budgets.rs`) — wall-clock
   bounds with 5–180× headroom, run as part of `cargo test` on any
   contributor's machine. They catch catastrophic regressions (e.g. an
   accidentally-uncached schema validator) without any CI setup. Run:

   ```sh
   cargo test --release --test perf_budgets -- --nocapture
   ```

2. **CI-side benchmark comparison** (`.github/workflows/bench.yml`) — runs
   `cargo bench` on every PR, compares against the `main` baseline stored
   on a `gh-pages` branch, and **fails the build** if any benchmark
   regresses by more than 10%. Posts a comment on the PR with the diff.
   Powered by
   [`benchmark-action/github-action-benchmark`](https://github.com/benchmark-action/github-action-benchmark).

3. **Local criterion baselines** (`benches/xarf.rs`) — for fine-grained
   work on a single machine:

   ```sh
   cargo bench --bench xarf -- --save-baseline main
   # ... make changes ...
   cargo bench --bench xarf -- --baseline main
   ```

   Criterion writes reports to `target/criterion/` (gitignored, so these
   stay local) and flags any operation that regresses by more than 5%.

The committed tests are the universal fallback; the CI workflow is the
authoritative regression gate; the local baselines are for development.

## Sync and async

The public API is synchronous and performs **zero I/O at runtime** — all 34
JSON schemas are embedded into the binary with `include_str!`, so there is
no filesystem or network access to await on. That means:

* Call `parse`, `validate`, `ReportBuilder::build`, etc. directly from any
  context — synchronous CLI tools, `tokio` tasks, `async-std`, threads,
  anywhere.
* Every public type is `Send + Sync` (verified at compile time with
  `static_assertions`), so reports flow across `tokio::spawn` boundaries
  and channels without wrapping.
* For very large batches you can push the work off the runtime with
  `tokio::task::spawn_blocking(move || xarf::parse(...))` — but for normal
  message sizes the parse is microseconds and you don't need to.

See `tests/async_compat.rs` for working examples of multi-threaded tokio
runtimes, `spawn`, `spawn_blocking`, and concurrent registry use.

## Testing

The crate ships with **115 tests** across **eleven test binaries**:

* `golden_parser_tests` — exercises every sample from the shared
  [`xarf-parser-tests`](https://github.com/xarf/xarf-parser-tests) corpus
  (44 valid + 5 invalid) plus the 32 canonical samples from `xarf-spec`.
* `snapshots` — `insta` snapshot tests covering v3→v4 conversion output,
  validation error shape, strict-mode promotion, `show_missing_optional`
  output, generator output, and round-trip stability. Run
  `cargo install cargo-insta && cargo insta review` to inspect pending
  changes.
* `async_compat` — `Send + Sync` compile-time assertions plus `tokio`
  integration tests (single + multi-thread runtimes, spawn boundary,
  concurrent registry use, `spawn_blocking` batches).
* `parser_tests` — JSON parsing, strict mode, info mode, unknown-field
  handling, evidence and tag round-trips.
* `validator_tests` — schema enforcement for every category, format
  validation (UUID, email, hostname, datetime), `x-recommended` promotion
  in strict mode.
* `generator_tests` — `ReportBuilder` auto-metadata, hash algorithms,
  evidence creation, strict-mode generation errors.
* `v3_compat_tests` — detection, conversion, every documented v3
  ReportType, error paths.
* `model_tests` — `Category` (un)known, contact/evidence (de)serialization,
  internal stripping.
* `round_trip` — parse → mutate → serialize → re-parse for every spec
  sample, plus extension-field preservation.
* `smoke`, `api_surface` — sanity checks.

Run them all:

```sh
cargo test
```

## License

MIT — see [LICENSE](LICENSE).
