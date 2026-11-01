# Changelog

All notable changes to this crate are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.4.5] - 2026-11-02

### Changed

- Clarify synchronous/asynchronous dispatch restoration on publisher panic and
  async future cancellation in English and Japanese rustdoc, including unpolled
  futures, irreversible publication side effects, duplicate retries, and limits
  for `panic=abort` or panics during restoration. Runtime behavior and public APIs
  are unchanged (#29).
- Separate functional tests from ignored trybuild UI suites; the dedicated UI
  script and CI/Release gates retain all diagnostic checks (#32).

## [0.4.4] - 2026-11-01

### Changed

- Document boxed `Send` future allocation/dynamic-dispatch trade-offs, the intent
  of type erasure and explicit bounds, and the exclusion of `!Send` futures even
  on single-threaded executors, in English and Japanese (#15).
- Replace `Secret` reference/zeroization pointer casts with safe `ManuallyDrop`
  dereferencing; retain and explain the necessary unsafe take/drop operations.
  Regression tests verify zeroize-before-drop exactly once and `into_inner`
  without wrapper zeroization or double drop (#16).
- Share the synchronous/asynchronous event restoration guard as a private module
  without changing APIs, FIFO order, successful consumption, or restoration on
  error, panic, and async cancellation. Add async publisher-future panic coverage (#17).

## [0.4.3] - 2026-11-01

### Fixed

- Keep the generated-code module available without serde so missing-feature
  diagnostics identify the gated `serde` re-export rather than `__private` (#13).
  Independent core/facade consumers verify serde opt-in and plain derives.

## [0.4.2] - 2026-11-01

### Fixed

- Alias the core library to itself so generated absolute crate paths also work
  inside library targets (#11).

## [0.4.1] - 2026-11-01

### Changed

- Clarified that testing targets the latest stable Rust without a specified MSRV;
  removed the unverified Rust 1.85 minimum declaration and MSRV badges.
- Grouped rustdoc prose by language and placed language labels on separate lines.
- Re-exported APIs inherit their original documentation without duplicate language sections.
- Simplified two test-helper conditions to satisfy stable Clippy after removing
  the MSRV declaration; behavior is unchanged.

## [0.4.0] - 2026-11-01

### Changed

- **Breaking:** replaced the previous `domain`, `port`, `adapter`, `mock`, and
  `application` module API with traits and helpers re-exported from the crate
  root. Existing implementations and imports must be migrated; this is not a
  drop-in update from 0.3.x.
- Domain types implement `Entity`, `ValueObject`, `AggregateRoot`, and
  `DomainEvent` directly. Aggregate event queues use `record_event` and
  `pull_events`; repositories and event publishers have separate contracts.
- Async ports return boxed `Send` futures without choosing an async runtime.
  Storage, concurrency, transaction, retry, and delivery guarantees belong to
  adapters and applications.
- Replaced the README examples with current API examples and added a Japanese
  README and bilingual English/Japanese API documentation.

### Added

- Synchronous and asynchronous repository, event publishing, dispatch, CQRS,
  and lifecycle contracts, with error and cancellation contract tests.
- Event-sourcing history/revision types, event stores, aggregate replay,
  snapshots, and optional-snapshot replay helpers.
- Event consumption, delivery metadata, transactional outbox ports and
  single-attempt lifecycle contexts, and checkpoint/idempotent projection ports.
- `Specification` composition, domain service/policy/factory contracts,
  process-manager transitions, unit-of-work contracts, and event upcaster chains.
- `Secret<T>` with explicit plaintext access, redacted formatting, and
  `zeroize`-based cleanup, plus the `SecretMetadata` marker.

### Removed

- **Breaking:** legacy traits and helpers such as `EntityId`, `Wrapped`,
  `SecretVo`, `EnumVo`, `ValidationError`, `UseCase`, `Load`/`Save`/`Delete`,
  `EventDispatcher`, `Clock`, `IdGenerator`, and `PortError`, along with the
  built-in storage, clock, and ID-generator adapters and test doubles.
- **Breaking:** `chrono` and `uuid` features and dependencies. The remaining
  optional `serde` feature exposes serde for generated implementations; it no
  longer derives serialization for the removed `ValidationError` type.
- The `trait-variant` dependency.

## [0.3.0] - 2026-07-21

### Added

- `serde` feature: derives `Serialize` for `domain::ValidationError`. Not
  `Deserialize` - `type_name` is `&'static str`, and there is no sound way
  to produce that from arbitrary deserialized input without leaking memory.

## [0.2.3] - 2026-07-21

### Added

- `CONTRIBUTING.md` and issue templates (bug report / feature request),
  covering dev setup, the checks CI runs, and the PR-based workflow.
- `main` is now branch-protected: direct pushes are disabled and PRs must
  pass CI to merge. No review is required yet (single-maintainer project).
  Repo-only; doesn't affect the published package.
- CI: a `coverage` job using `cargo-llvm-cov` uploads a report to Codecov;
  added the resulting badge to the README.

## [0.2.2] - 2026-07-21

### Added

- `# Examples` sections (compiled/run as doctests) on every public trait,
  struct, and enum outside the crate root.
- README: crates.io/docs.rs/CI/MSRV/license badges, an `Installation`
  section with per-feature guidance, a second example showing
  `AggregateRoot` + `InMemoryStore` save/load, and a link to this
  changelog.

### Fixed

- Moved `domain`'s module doc from an outer `///` on `lib.rs`'s `pub mod
  domain;` into a `//!` in `domain/mod.rs`, matching every other module.

## [0.2.1] - 2026-07-20

### Added

- Crates.io metadata (`license`, `description`, `repository`, `documentation`,
  `readme`, `keywords`, `categories`), `LICENSE-MIT`/`LICENSE-APACHE`, and a
  `README.md` with a verified-working example, in preparation for the first
  crates.io release.
- Crate-level and per-item docs closing every `missing_docs` gap on the
  public API surface.
- `[package.metadata.docs.rs] all-features = true` so docs.rs renders the
  `chrono`/`uuid`-gated items instead of just the default build.
- A crate-level `Example` doctest mirroring the README's, now checked by
  `cargo test --doc`.
- CI: a GitHub Actions workflow testing the feature matrix
  (none/default/`chrono`/`uuid`/all), `cargo fmt --check`, and
  `cargo clippy -D warnings`.

### Fixed

- Fixed a broken intra-doc link (`application::UseCase` ->
  `application::usecase::UseCase`).

### Changed

- Pinned `rust-version = "1.85"` (this crate's edition 2024 floor) and
  excluded the `.github` workflow from the published package.

## [0.2.0] - 2026-07-20

### Fixed

- `EventDispatcher::dispatch` now reports undelivered events via
  `DispatchError<E>` when dispatch fails partway through, instead of
  silently dropping them.
- `testing::block_on` now times out instead of spinning forever when a
  future never completes.

### Changed

- `UuidV4Generator`/`UuidV7Generator` now share their generation logic
  instead of duplicating it.

### Documentation

- Clarified the `Delete` port's idempotent-on-missing-id contract.
- Clarified the `Save` port's `Conflict` contract.
- Clarified identity equality vs. derived `PartialEq` on `Entity`.
- Documented the `type_name` invariant on `ValidationError`.
- Documented that redaction is a `Debug`-impl responsibility on `SecretVo`.
- Documented the `variants()`/`from_str` consistency expectation on `EnumVo`.

## [0.1.0] - 2026-07-20

### Added

- `domain`: `Entity`, `AggregateRoot`, `ValueObject`, `EntityId`, `SecretVo`,
  `EnumVo`, `DomainEvent`, `ValidationError`.
- `port`: `Load`/`Save`/`Delete` repository traits, `EventDispatcher`,
  `Clock`, `IdGenerator`, `PortError`.
- `adapter` (behind the `chrono`/`uuid` features): `SystemClock`,
  `UuidV4Generator`, `UuidV7Generator`.
- `mock`: `InMemoryStore`, `FixedClock`, `FixedIdGenerator`.
- `application`: `UseCase`.

### Fixed

- `UseCase` regained its `Sync` bound.
- `InMemoryStore::save` is now atomic across events and aggregates.
- Poisoned mutexes in `mock` are recovered from instead of panicking forever.
- `SystemClock` tests no longer assert wall-clock monotonicity.

[Unreleased]: https://github.com/ryoshrimp/ddd-toolkit/commits/main
[0.4.1]: https://crates.io/crates/ddd-toolkit-core/0.4.1
[0.4.0]: https://github.com/ryoshrimp/ddd-toolkit-core/compare/v0.3.0...v0.4.0
[0.3.0]: https://github.com/ryoshrimp/ddd-toolkit-core/compare/v0.2.3...v0.3.0
[0.2.3]: https://github.com/ryoshrimp/ddd-toolkit-core/compare/v0.2.2...v0.2.3
[0.2.2]: https://github.com/ryoshrimp/ddd-toolkit-core/compare/v0.2.1...v0.2.2
[0.2.1]: https://github.com/ryoshrimp/ddd-toolkit-core/compare/v0.2.0...v0.2.1
[0.2.0]: https://github.com/ryoshrimp/ddd-toolkit-core/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/ryoshrimp/ddd-toolkit-core/releases/tag/v0.1.0
