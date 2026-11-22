# Changelog

All notable changes to this crate are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- `#[derive(AggregateRoot)]` now accepts `alloc::vec::Vec<Event>` (and the
  `::alloc` form) as the event queue type, and the rejection error lists the
  accepted spellings and notes that type aliases are not supported (#67).
- Declared the minimum supported Rust version as 1.88 (`rust-version`), verified
  by a new CI job (#72).
- Added `#![forbid(unsafe_code)]` (#72).
- `#[derive(ValueObject)]` and `#[derive(Entity)]` now detect field and variant
  names that collide with generated methods (such as a `new` field, or an `Inner`
  enum variant) and report an error pointing at the field or variant instead of
  an `E0592` in generated code (#62).
- `#[value_object(no_eq)]` skips the generated `Eq` and `Hash` impls and
  `#[value_object(no_hash)]` skips only `Hash`, so field types such as `f64` can
  use `ValueObject` together with `validate`, `error`, and `serde` (#61). Both flags work on every shape, may be combined (same as
  `no_eq`), and cannot be combined with `trait_only`. Default output is unchanged.

### Changed

- `#[derive(ValueObject)]` now reports unsatisfied `Debug`/`Clone`/`PartialEq`/`Eq`/`Hash`
  bounds, and `#[derive(Entity)]` unsatisfied `PartialEq`/`Eq`/`Hash` bounds on the id
  type, at the offending field type instead of the derive attribute (#81).
- **Breaking:** `#[derive(ValueObject)]` on an enum now rejects `validate` and
  `error` with a compile error, because enum variants are always public and
  `Pct::Value(200)` bypassed `try_new_<variant>` validation (#60). Enums without
  `validate` are unchanged. To migrate, keep the enum private, derive
  `ValueObject` (and `serde`, with any `#[serde(..)]` attributes) on it without
  `validate`, wrap it in a single-field struct with a non-`pub` field, and move
  `validate`/`error` (and `serde`) to the struct. `Enum::try_new_<variant>(..)`
  becomes `Wrapper::try_new(Enum::new_<variant>(..))`; the wire format is
  unchanged because a single-field struct serializes as its inner value.
- **Breaking:** `#[derive(ValueObject)]` on an enum now builds the
  `new_<variant>`/`is_<variant>`/`as_<variant>` suffix in snake_case instead of
  fully lowercasing the variant name (#64). `PendingPayment` now generates
  `new_pending_payment`, `is_pending_payment`, and `as_pending_payment`; acronym
  runs stay together (`HTTPError` becomes `http_error`) and digits attach to the
  preceding word (`Http2Server` becomes `http2_server`). The old names are
  removed with no deprecated aliases. To migrate, rename call sites, for example
  `new_pendingpayment` to `new_pending_payment` and `is_httperror` to
  `is_http_error`. Single-word variants (`Active` gives `is_active`) are
  unchanged. Variants that collide after conversion (`FooBar` and `Foo_Bar`) are
  reported as a duplicate-name error.
- **Breaking:** `#[derive(ValueObject)]` on a single-field struct now generates a
  `Debug` that follows the definition shape and includes the type name
  (`Email("a")` for a tuple struct, `Email { value: "a" }` for a named one) instead
  of printing only the inner value (#66). To keep the old output, format the inner
  value, for example `{:?}` of `value.as_inner()`, or use `trait_only` and write your
  own `Debug`.
- **Breaking:** `Secret<T>` no longer implements `Clone`, so `#[derive(ValueObject)]` types
  that contain a `Secret<T>` no longer compile. This follows the core change (#70).
- **Breaking:** the `#[value_object(validate = ..)]` validator of a single-field struct
  now receives `&Self` instead of `&Inner`, so every shape (single-field, multi-field)
  passes the constructed value object, as `Entity` validators already do (#65). `try_new`
  and the serde-generated deserialization build the value first and then validate it.
  To migrate, change the parameter type from the field type to the struct and read the
  field through it: `fn check(value: &String)` becomes `fn check(value: &Email)` using
  `value.0` (`value.<field>` for a named field). A validator written in the old form fails
  to compile with a type mismatch pointing at the `validate = ..` path. Multi-field
  structs and `Entity` are unchanged.

### Fixed

- `#[derive(ValueObject)]` now omits the `r#` prefix of raw identifiers (type, variant,
  and field names) in the generated `Debug` for multi-field structs and enums, matching
  `#[derive(Debug)]` (#93).
- `Entity` getter conflict check now catches a raw-identifier `r#id` field.
- `ValueObject` enum duplicate-name check after snake_case conversion now also
  catches raw identifiers (for example `r#Foo` and `FOO`), which previously
  failed with an `E0592` in generated code instead of a macro error.
- Attribute parsers of `Entity`, `ValueObject`, and `AggregateRoot` now name the
  unknown option and list the valid ones. `Entity` options placed on the wrong
  target (for example `skip_getter` on the struct, or `getters` on a field) get a
  placement message instead of a misleading one, and a repeated `events` in
  `#[aggregate(...)]` reports `duplicate events attribute`. `#[aggregate(events,)]`
  (trailing comma) is now accepted, and `#[aggregate(events, other)]` now reports
  the unknown option instead of "events is a flag and takes no value" (#63).

## [0.5.5] - 2026-11-08

### Added

- Opt-in `#[entity(getters)]` generates borrowed public field accessors;
  `#[entity(skip_getter)]` excludes individual fields. Preserve the existing
  `id()` accessor and default behavior, support validation/serde, and reject
  combinations with `trait_only` (#56).

## [0.5.4] - 2026-11-05

### Added

- Opt-in `#[entity(validate = check, error = Error, serde)]` support. Validation
  and serde can be enabled independently; `try_new` and deserialization validate
  state without generating IDs or creation events. Preserve supplied IDs and
  identity-based equality/hashing; reject combinations with `trait_only` (#52).

### Fixed

- Place generic `where` clauses before shared named-struct serde helper bodies
  and serialize raw identifier field names without the `r#` prefix (#52).
- Delegate ValueObject and Entity `#[serde(...)]` wire-format customization to
  Serde derive, including all naming rules, aliases, field defaults/skips,
  flattening, tags, borrowing, bounds, and field codecs. Borrowed serialization
  avoids cloning; owned deserialization preserves domain validation and supplied
  Entity IDs. Reject unsupported attributes explicitly (including container
  conversions/default and crate overrides); document the allowlist and
  limitations in `docs/macro-serde.md` (#51).

## [0.5.3] - 2026-11-03

### Fixed

- Bring parent-scope imports into ValueObject serde helper modules so imported
  field types and generic bounds resolve for multi-field structs and enums.
  Use value-object-specific helper type names to avoid shadowing user types
  named `Fields` or `Enum`, and fix the serializer call for multi-field tuple
  structs. Facade consumers cover all helper shapes and imported types in
  Entity, AggregateRoot, and DomainEvent derives (#48).

## [0.5.2] - 2026-11-02

### Added

- Opt-in `#[value_object(trait_only)]` and `#[entity(trait_only)]` derive modes
  generate only the corresponding domain trait, with no constructors, inherent
  methods, standard trait implementations, or extra bounds. Existing handwritten
  APIs and equality semantics can be retained; default derive behavior is unchanged
  (#34).
- Facade-only consumer coverage for handwritten APIs, renamed dependencies, and
  absent generated constructors, `Debug`, and `Clone`, plus English/Japanese
  documentation of the modes and their invariant-protection limitations (#34).

## [0.5.1] - 2026-11-01

### Fixed

- Generate shorthand fields for same-name bindings in named struct and enum
  constructors, including validation and single fields named `inner`, without
  changing argument order or APIs. Preserve explicit fields for different bindings.
  Facade consumers pass strict Clippy without redundant-field-names allowances (#22).

### Added

- Core compatibility contract (>=0.4.1, <0.5.0), exact released-minimum and
  current-core consumer coverage with dependency renames and serde on/off (#14).
- Independent missing-serde-feature regression tests and feature troubleshooting
  for direct-core and facade consumers (#13).

## [0.5.0] - 2026-11-01

### Fixed

- Resolve direct core dependencies first, falling back to facade re-exports;
  preserve both dependency renames and library self-references. Diagnostics now
  name `ddd-toolkit-core` and `ddd-toolkit` (#11).
- Single-field tuple enum serialization now matches its generated deserializer;
  scalar and container fields round-trip, and validation is preserved (#21).

### Changed

- **Breaking (0.5.0):** single-field tuple enum variants call
  `serialize_newtype_variant` rather than `serialize_tuple_variant`.
  `{"Code":[7]}` becomes `{"Code":7}`; `{"Codes":[[7,8]]}` becomes
  `{"Codes":[7,8]}`. Migrate affected saved records using schema/version metadata
  by removing exactly one outer tuple wrapper. No automatic fallback is provided
  because container fields can make formats ambiguous. Coordinate readers and
  writers or pin macro 0.4 / facade 0.3; see the bilingual README migration notes.
- **破壊的変更 (0.5.0):** 単一フィールドタプルバリアントを newtype 形式に変更します。
  保存済みの対象レコードの外側の配列を1段除去し、送受信側を調整して更新してください。
  コンテナーとの曖昧さを避けるため旧形式への自動フォールバックはありません。

## [0.4.1] - 2026-11-01

### Added

- Japanese README covering all four derives, validation, serialization, and aggregate events.

### Changed

- Clarified that testing targets the latest stable Rust without a specified MSRV
  and removed the unverified Rust 1.85 minimum declaration.
- Placed rustdoc language labels on separate lines from their text.
- Simplified two nested conditions to satisfy stable Clippy after removing the
  MSRV declaration; macro behavior is unchanged.

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

[Unreleased]: https://github.com/ryoshrimp/ddd-toolkit/commits/main
[0.4.1]: https://crates.io/crates/ddd-toolkit-macro/0.4.1
[0.4.0]: https://github.com/ryoshrimp/ddd-toolkit-macro/compare/v0.3.0...v0.4.0
[0.3.0]: https://github.com/ryoshrimp/ddd-toolkit-macro/compare/v0.2.4...v0.3.0
[0.2.4]: https://github.com/ryoshrimp/ddd-toolkit-macro/compare/v0.2.3...v0.2.4
[0.2.3]: https://github.com/ryoshrimp/ddd-toolkit-macro/compare/v0.2.2...v0.2.3
[0.2.2]: https://github.com/ryoshrimp/ddd-toolkit-macro/compare/v0.2.1...v0.2.2
[0.2.1]: https://github.com/ryoshrimp/ddd-toolkit-macro/compare/v0.2.0...v0.2.1
[0.2.0]: https://github.com/ryoshrimp/ddd-toolkit-macro/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/ryoshrimp/ddd-toolkit-macro/releases/tag/v0.1.0
