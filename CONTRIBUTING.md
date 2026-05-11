# Contributing

Thanks for considering a contribution to `xarf-rs`. This document covers the
project conventions; if anything here is unclear, open an issue and ask.

## Development setup

```sh
# Install rustup if you haven't (https://rustup.rs/).
git clone https://github.com/wiggels/xarf-rs
cd xarf-rs

# Confirm the toolchain is installed and the test suite is green.
cargo test
```

The MSRV is **1.74**. CI verifies the crate still builds on that version,
so don't reach for `let-else` features newer than that without bumping the
MSRV (which is a breaking change for downstream consumers).

## Project layout

```txt
src/
  lib.rs          Public re-exports; doc landing page.
  error.rs        XarfError, ValidationError, ValidationWarning, ValidationInfo.
  model.rs        Report, Contact, Evidence, Category.
  parser.rs       parse, parse_value, parse_with_options.
  validator.rs    validate, ValidateOptions, ValidationResult.
  generator.rs    ReportBuilder, create_report, create_evidence.
  v3_compat.rs    is_v3_report, convert_v3_to_v4.
  schemas.rs      Embedded JSON Schemas + lazily-compiled validators.

tests/            Integration + snapshot + async-compat + perf-budget tests.
benches/          Criterion regression benchmarks.
schemas/v4/       Bundled XARF v4 JSON Schemas (included via `include_str!`).
.github/          CI, release, audit, semver, coverage, dependabot configs.
```

## Pull request checklist

Before opening a PR, run locally:

```sh
cargo fmt --all
cargo clippy --all-targets -- -D warnings
cargo test --all-targets --locked
cargo test --release --test perf_budgets -- --nocapture
cargo doc --no-deps --all-features
```

The same checks run in CI. PRs need every one of them green to merge.

### Commit messages

Use [Conventional Commits](https://www.conventionalcommits.org/). `release-plz`
parses commit messages to generate changelog entries and version bumps, so:

* `feat: ...` → minor bump, listed under "Added"
* `fix: ...` → patch bump, listed under "Fixed"
* `perf: ...` → patch bump, listed under "Performance"
* `docs: ...` → no version change, listed under "Documentation"
* `chore: ...` / `ci: ...` → no version change, skipped from changelog
* Anything with `BREAKING CHANGE:` in the body → major bump

### Snapshot tests

We use [`insta`](https://insta.rs/) for snapshot tests of error output,
v3 conversion shape, and the typed `Report` layout. When you change
something that affects output, expect a snapshot to fail. Review the diff
in your terminal, decide whether the change is intentional, and accept it:

```sh
cargo install cargo-insta   # one time
cargo insta review          # walks each pending snapshot interactively
```

The accepted `.snap` files get committed alongside the code change.

### Benchmarks

The repo has three layers of perf regression defense:

1. **`tests/perf_budgets.rs`** — wall-clock budgets enforced by `cargo test`.
   Catastrophic regressions fail there.
2. **`benches/xarf.rs`** — criterion micro-benchmarks. Run locally with
   `cargo bench --bench xarf -- --save-baseline main` before changes,
   then `cargo bench --bench xarf -- --baseline main` after to compare.
3. **CI bench workflow** — runs against `main` on every PR and fails the
   build if anything regresses by more than 10%.

If you're optimising, please include before/after criterion output in the
PR description.

## Release process

Releases are automated via [`release-plz`](https://release-plz.dev/). The
flow:

1. Commits land on `main` using Conventional Commits.
2. `release-plz` opens a "Release v0.x.y" PR with a version bump and a
   draft changelog entry. The maintainer reviews and merges it.
3. The merge triggers `release-plz release`, which tags the commit,
   publishes to crates.io, and creates a GitHub Release.

Don't bump versions or write `CHANGELOG.md` entries by hand — let
release-plz manage them. If a version bump needs manual adjustment
(e.g. you want to skip a planned 0.x.0 and go straight to 1.0.0), do it
on the release PR, not in a separate commit.

## License

By contributing, you agree that your contributions will be licensed under
the same MIT licence that covers the rest of the project.
