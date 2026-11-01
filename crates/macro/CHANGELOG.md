# Changelog

All notable changes to this crate are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Japanese README covering all four derives, validation, serialization, and aggregate events.

### Changed

- Clarified that testing targets the latest stable Rust without a specified MSRV
  and removed the unverified Rust 1.85 minimum declaration.
- Placed rustdoc language labels on separate lines from their text.

## [0.4.0] - 2026-11-01

### Added

- `Entity` derive with an identifier selected by `#[entity(id)]`, an `id()`
  accessor, and identity-based equality and hashing.
- `DomainEvent` marker derive for structs and enums.
- `AggregateRoot` derive with a `#[aggregate(events)]` vector, ordered event
  recording, and draining retrieval.
- Named and multi-field tuple structs and data-carrying enums for `ValueObject`,
  with constructors and borrowed field or variant accessors.
- Bilingual English/Japanese rustdoc, runnable examples, and compile-time tests.

### Changed

- **Breaking:** `ValueObject` implements core's `ValueObject` trait with
  `as_inner` and `into_inner`, plus `Debug`, `Clone`, `PartialEq`, `Eq`, and `Hash`.
  Construction uses `new`, or `try_new` when validation is configured, instead
  of the previous conversion-based API.
- **Breaking:** validation uses paired `validate = path` and `error = ErrorType`
  options in `#[value_object(...)]` rather than `#[vo(...)]`.
- **Breaking:** serde generation is selected per value object with
  `#[value_object(serde)]` and requires the core dependency's `serde` feature.
  Deserialization validates when configured; enums use externally tagged variants.
- **Breaking:** generated crate paths require a direct `ddd-toolkit-core`
  dependency; renamed dependencies work, but facade-only dependencies do not.
- Tests use `ddd-toolkit-core` with its serde support enabled. CI no longer
  requests removed macro-crate features.
- README and package metadata describe the current four derives.

### Removed

- **Breaking:** `EntityId`, `SecretVo`, and `EnumVo` derives, `#[vo(...)]`
  attributes, and macro-crate `zeroize` and `serde` features.
- Generated `Display`, `TryFrom`, and inner-value `From` implementations from
  the previous value-object API.

## [0.3.0] - 2026-07-21

### Added

- `serde` feature: `#[derive(ValueObject)]`/`#[derive(EntityId)]`/
  `#[derive(EnumVo)]` gain `Serialize`/`Deserialize`. Wrapped types
  serialize as their inner value and re-run the inner value through
  `TryFrom` on deserialize, so an invalid value is rejected the same way
  either path constructs one; `EnumVo` serializes as its `Display`/
  `FromStr` string form. `SecretVo` is intentionally excluded (a
  `Serialize` impl on a secret defeats the point of the type).

## [0.2.4] - 2026-07-21

### Added

- `CONTRIBUTING.md` and issue templates (bug report / feature request),
  covering dev setup, the checks CI runs, and the PR-based workflow.
- `main` is now branch-protected: direct pushes are disabled and PRs must
  pass CI to merge. No review is required yet (single-maintainer project).
  Repo-only; doesn't affect the published package.
- CI: a `coverage` job using `cargo-llvm-cov` (`--features zeroize`) uploads
  a report to Codecov; added the resulting badge to the README.

## [0.2.3] - 2026-07-21

### Added

- Full rustdoc for all four `proc_macro_derive` functions: compile-time
  requirements, accepted `#[vo(...)]` attributes (and why others are
  rejected), and runnable `Examples` exercised as doctests. Crate root
  `#![deny(missing_docs)]` locks the now fully-covered public surface
  against regressions.
- README: badges, an Installation section, an `EntityId` usage example, and
  a Changelog pointer, mirroring `ddd-toolkit-core`'s README structure.

### Changed

- The `ddd-toolkit-core` dev-dependency now points at the published v0.2.2
  crates.io release instead of a git tag. Dev-only; doesn't affect the
  published package.

## [0.2.2] - 2026-07-20

### Added

- Crates.io metadata (`license`, `description`, `repository`, `documentation`,
  `readme`, `keywords`, `categories`), `LICENSE-MIT`/`LICENSE-APACHE`, and a
  `README.md` documenting every derive and `#[vo(...)]` attribute, in
  preparation for the first crates.io release.
- Crate-level and per-derive docs closing every `missing_docs` gap on the
  public API surface.
- `[package.metadata.docs.rs] all-features = true` so docs.rs renders the
  `zeroize`-gated derive support instead of just the default build.
- CI: a GitHub Actions workflow testing with default and `--features
  zeroize`, `cargo fmt --check`, and `cargo clippy -D warnings`.

### Fixed

- The `ddd-toolkit-core` git dev-dependency now uses HTTPS instead of SSH, so
  CI (and anyone building this crate's own tests without an SSH key) can
  fetch it.

### Changed

- Moved the `ddd-toolkit-core` dependency from `[dependencies]` to
  `[dev-dependencies]`: nothing in `src/` needs it compiled in (the derives
  only reference it via generated token paths resolved in the *calling*
  crate). This also means this crate no longer needs a `version` alongside
  its `git` dependency to pass `cargo publish`, and can be published
  independently of `ddd-toolkit-core`'s publish status.
- The `ddd-toolkit-core` dev-dependency now pins tag `v0.2.1` (previously
  `v0.2.0`), matching its currently published version. Dev-only; doesn't
  affect the published package.
- Pinned `rust-version = "1.85"` (this crate's edition 2024 floor) and
  excluded the `.github` workflow from the published package.

## [0.2.1] - 2026-07-20

### Fixed

- Every derive macro emitted a hardcoded `ddd_toolkit_core::domain::...` path
  in its generated code, which only resolved if the invoking crate depended
  on `ddd-toolkit-core` directly. A downstream user depending only on the
  `ddd-toolkit` facade got `error[E0433]: cannot find module or crate
  ddd_toolkit_core` on the first use of any derive. `core_crate_path()` now
  resolves, per macro invocation, whether the calling crate depends on
  `ddd-toolkit-core` directly or only through the facade, and emits the
  matching path.

## [0.2.0] - 2026-07-20

### Fixed

- Removed an invalid `pub use zeroize;` re-export from this proc-macro
  crate's `lib.rs` — proc-macro crates cannot export anything besides their
  proc-macro entry points, so enabling the `zeroize` feature failed to
  compile the crate at all.
- `#[vo(zeroize)]` is now gated behind this crate's own `zeroize` Cargo
  feature. Previously it always emitted a `zeroize::Zeroize::zeroize` call
  regardless of the feature flag, producing a confusing "unresolved crate"
  error for consumers who used the attribute without adding `zeroize` as
  their own direct dependency.
- `ValueObject`/`EntityId`/`SecretVo` now reject a publicly-visible wrapped
  field at derive time. A `pub` field lets external code bypass
  `TryFrom`/`try_new` validation entirely by constructing the type directly;
  the macro can't fix the field, but it can refuse to derive for it.

## [0.1.0] - 2026-07-20

### Added

- `#[derive(ValueObject)]`, `#[derive(EntityId)]`, `#[derive(SecretVo)]`,
  `#[derive(EnumVo)]`, implementing `ddd-toolkit-core`'s corresponding
  traits.
- Unit tests and macro-expansion integration tests for all four derives.

### Fixed

- Several `#[vo(...)]` attributes were previously accepted silently but had
  no effect on macros they weren't wired up for (`validate` and
  `no_display` on `EnumVo`; `rename_all` on
  `ValueObject`/`EntityId`/`SecretVo`) — `validate` in particular was a
  silent validation bypass. `EnumVo` now rejects `validate` (variant
  membership is already exhaustive) and properly supports `no_display`; the
  other three macros now reject `rename_all`.

[Unreleased]: https://github.com/ryoshrimp/ddd-toolkit-macro/compare/v0.4.0...HEAD
[0.4.0]: https://github.com/ryoshrimp/ddd-toolkit-macro/compare/v0.3.0...v0.4.0
[0.3.0]: https://github.com/ryoshrimp/ddd-toolkit-macro/compare/v0.2.4...v0.3.0
[0.2.4]: https://github.com/ryoshrimp/ddd-toolkit-macro/compare/v0.2.3...v0.2.4
[0.2.3]: https://github.com/ryoshrimp/ddd-toolkit-macro/compare/v0.2.2...v0.2.3
[0.2.2]: https://github.com/ryoshrimp/ddd-toolkit-macro/compare/v0.2.1...v0.2.2
[0.2.1]: https://github.com/ryoshrimp/ddd-toolkit-macro/compare/v0.2.0...v0.2.1
[0.2.0]: https://github.com/ryoshrimp/ddd-toolkit-macro/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/ryoshrimp/ddd-toolkit-macro/releases/tag/v0.1.0
