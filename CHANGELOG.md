# Changelog

All notable changes to this crate are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.4.1] - 2026-11-01

### Fixed

- Upgrade to core 0.4.3 for serde-feature diagnostics and macro 0.5.1 for
  warning-free named value-object constructors, including validated constructors
  and enum variants. Consumers no longer need a redundant-field-names allow (#13, #22).

### Added

- Stable/rust-src diagnostic snapshot update and review procedure shared by CI
  and Release checks (#12).
- Documented generated-code compatibility contract and isolated minimum/current
  core consumer tests, covering direct/facade, renames and serde feature boundaries (#14).

## [0.4.0] - 2026-11-01

### Fixed

- Derives now support facade-only and renamed facade dependencies, preferring
  direct core dependencies when present. Missing-dependency errors use current
  package names (#11). Isolated Cargo consumer tests cover resolution and serde.
- Single-field tuple enum value objects now round-trip scalar and container
  fields using Serde's newtype variant format; deserialization still validates (#21).

### Changed

- **Breaking:** upgraded to macro 0.5.0 (core remains 0.4.2).
  `Code(7)` now serializes as `{"Code":7}`, not `{"Code":[7]}`. Migrate saved
  single-field variants by removing exactly one outer tuple wrapper, including
  `{"Codes":[[7,8]]}` → `{"Codes":[7,8]}`. Identify old records by schema/version;
  there is no ambiguous container-format fallback. Coordinate reader/writer
  upgrades or pin facade 0.3 / macro 0.4. See the README migration notes.
- **破壊的変更:** 単一フィールドタプルバリアントは newtype 形式になります。
  保存済みの対象データは外側の配列を1段除去し、送受信側を調整して更新してください。
  コンテナーとの曖昧さを避けるため旧形式への自動フォールバックはありません。

## [0.3.1] - 2026-11-01

### Added

- Japanese README with installation, examples, feature requirements, and migration notes.

### Changed

- Clarified that testing targets the latest stable Rust without a specified MSRV;
  removed the unverified Rust 1.85 minimum declaration and MSRV badges.
- Grouped rustdoc prose by language and placed language labels on separate lines.
- Re-exported APIs inherit their original documentation without duplicate language sections.
- Consolidated core, derive macros, and facade tests into one Cargo workspace.
  Crate names, versions, and public APIs are unchanged.
- Added a single release workflow that publishes missing crate versions in
  dependency order; see `docs/releasing.md`.

## [0.3.0] - 2026-11-01

### Changed

- **Breaking:** upgraded `ddd-toolkit-core` and `ddd-toolkit-macro` to 0.4.0.
  Core traits and helpers are now re-exported at the crate root instead of
  under `domain`/`port`/`adapter`/`mock`/`application`.
- **Breaking:** re-exported derives are now `ValueObject`, `Entity`,
  `DomainEvent`, and `AggregateRoot`. Macro 0.4 requires consumers to depend
  directly on `ddd-toolkit-core`, even when using derives via this facade.
- **Breaking:** value objects use `new` / `try_new` and
  `#[value_object(validate = path, error = ErrorType)]`, not generated
  conversions or `#[vo(...)]`. Do not duplicate generated standard derives.
- `serde` enables core's serde feature and serde's derive support required
  by the generated deserializers. Serialization is opt-in per value object
  with `#[value_object(serde)]`; consumers do not need a direct serde dependency.
- Replaced legacy integration tests and examples with current repository,
  sync/async publication, multi-aggregate, value-object, and secret usage.
  The feature matrix also runs doctests for each supported combination.

### Known upstream limitations

- Macro 0.4.0 requires the `serde` flag to appear last when sharing an attribute
  with validation options. Single-field tuple enum variants also have
  mismatched JSON serialization/deserialization shapes; prefer named variants
  for round trips. These constraints are documented and covered by tests.
- Clippy 1.99 flags redundant field names emitted by macro 0.4.0 for named
  value-object constructors. The affected integration test crates allow only
  `clippy::redundant_field_names` until the upstream generator is fixed;
  CI continues to deny other warnings.

### Removed

- **Breaking:** `chrono`, `uuid`, and `zeroize` features. Core no longer
  supplies storage/clock/ID adapters or test doubles; `Secret<T>` provides
  unconditional redaction and zeroization instead of `SecretVo`.
- **Breaking:** legacy exports including `EntityId`, `Wrapped`, `SecretVo`,
  `EnumVo`, `ValidationError`, `UseCase`, `Load`/`Save`/`Delete`,
  `EventDispatcher`, and `PortError`. See README for the migration guide.

## [0.2.0] - 2026-07-21

### Added

- `serde` feature, forwarding `ddd-toolkit-core`/`ddd-toolkit-macro`'s new
  `serde` features: `Serialize`/`Deserialize` on `ValueObject`/`EntityId`/
  `EnumVo` derives, not `SecretVo`. Requires depending on the `serde` crate
  directly, same as `zeroize`.

### Changed

- Bumped the `ddd-toolkit-core`/`ddd-toolkit-macro` dependency requirements
  to `0.3.0` (the versions that add the `serde` feature this crate now
  forwards).

## [0.1.5] - 2026-07-21

### Added

- `facade-tests/tests/multi_aggregate_flow.rs`: a `place_order` example
  coordinating two aggregates (`Order` and `Inventory`) through only
  `ddd_toolkit::` paths, covering a pre-mutation guard, saving both
  aggregates in one use case, and a `DispatchError` surfaced after both are
  already persisted.

### Fixed

- `scripts/check-feature-matrix.sh` was being included in the published
  crate (it's a CI/dev-only script, not part of the workspace's nested
  `facade-tests` crate, so it wasn't auto-excluded the way that is).
  Excluded it explicitly, matching `.github`.
- Removed the `coverage` CI job, its Codecov upload, and the README badge.
  `cargo llvm-cov --workspace` only measures this workspace's own members:
  `ddd-toolkit`'s `src/lib.rs` is just re-exports (near-zero coverable
  lines), and `facade-tests` has no library source of its own - the actual
  logic lives in `ddd-toolkit-core`/`ddd-toolkit-macro`, separate repos not
  part of this workspace. The job could only ever report ~0% with nothing
  to regress against, so it was pure CI cost with no signal; those two
  repos' own coverage (98%/97%) are the meaningful numbers.

## [0.1.4] - 2026-07-21

### Added

- `CONTRIBUTING.md` and issue templates (bug report / feature request),
  covering dev setup, the checks CI runs, and the PR-based workflow.
- `main` is now branch-protected: direct pushes are disabled and PRs must
  pass CI to merge. No review is required yet (single-maintainer project).
  Repo-only; doesn't affect the published package.
- CI: a `coverage` job using `cargo-llvm-cov` (workspace, all-features)
  uploads a report to Codecov; added the resulting badge to the README.

## [0.1.3] - 2026-07-21

### Changed

- Dropped the `ddd-toolkit-core`/`ddd-toolkit-macro` git submodules and the
  workspace `[patch]` that redirected to them; both are now plain
  crates.io path-free dependencies (`ddd-toolkit-core = "0.2.2"`,
  `ddd-toolkit-macro = "0.2.3"`). CI no longer checks out submodules or
  needs the trybuild git-cache warmup step that worked around the patch.
- Rewrote the crate-root doc comment and README as a fuller landing page:
  a module-by-module tour (`domain`/`port`/`adapter`/`mock`/`application`),
  a features table, badges, an `AggregateRoot` + `InMemoryStore` example
  alongside the existing `SecretVo` one, and a link to this changelog.

## [0.1.2] - 2026-07-20

### Fixed

- CI: `cargo test --workspace --all-features` had been failing on every run
  since CI was added. `ddd-toolkit-macro`'s trybuild UI tests build each
  fixture in an isolated, non-workspace crate with cargo's `--offline` flag
  hardcoded, but this workspace's `[patch]` on `ddd-toolkit-core`'s git URL
  meant the patched build here never fetched that git dependency for real,
  so trybuild's offline build had nothing to find. Added a CI step that
  fetches it for real first, from a `git worktree` outside this workspace
  (so the patch doesn't apply), to warm cargo's git cache. Repo/CI-only;
  doesn't affect the published package.

## [0.1.1] - 2026-07-20

### Added

- Crates.io metadata (`license`, `description`, `repository`, `documentation`,
  `readme`, `keywords`, `categories`), `LICENSE-MIT`/`LICENSE-APACHE`, and a
  `README.md` with a verified-working example, in preparation for the first
  crates.io release.
- `[package.metadata.docs.rs] all-features = true`, and a crate-level
  `Example` doctest mirroring the README's, now checked by
  `cargo test --doc`.
- CI: a GitHub Actions workflow running `cargo test --workspace
  --all-features`, `scripts/check-feature-matrix.sh`, `cargo fmt --check`,
  and `cargo clippy -D warnings`, checking out submodules recursively.

### Fixed

- `.gitmodules` (and the `[patch]` key in `Cargo.toml`) pointed at
  `ssh://git@github.com/...` - fine for local dev, but a fresh checkout
  running `git submodule update --init` would need an SSH key just to read
  these public repos. Switched to HTTPS.

### Changed

- Pinned `rust-version = "1.85"` (this crate's edition 2024 floor) and
  excluded the `.github` workflow from the published package.
- Bumped the `ddd-toolkit-core` dependency requirement to `0.2.1` and
  `ddd-toolkit-macro` to `0.2.2`, matching their now-published crates.io
  versions.

## [0.1.0] - 2026-07-20

### Added

- Initial release: a single-dependency facade that re-exports
  [`ddd-toolkit-core`](https://docs.rs/ddd-toolkit-core) in full, and, behind
  the `derive` feature (on by default),
  [`ddd-toolkit-macro`](https://docs.rs/ddd-toolkit-macro)'s derive macros.
- `chrono` and `uuid` features pass through to `ddd-toolkit-core`; `zeroize`
  passes through to `ddd-toolkit-macro`.
- `facade-tests`, a workspace-only test crate depending on nothing but this
  facade, simulating a real downstream consumer. It caught (and now
  regression-tests) an `E0433` crate-resolution bug in the derive macros
  that only reproduced through facade-only dependency paths, plus a
  `zeroize`-feature compile break. `scripts/check-feature-matrix.sh` checks
  the facade compiles under every individual feature combination.

[Unreleased]: https://github.com/ryoshrimp/ddd-toolkit/commits/main
[0.3.1]: https://crates.io/crates/ddd-toolkit/0.3.1
[0.3.0]: https://github.com/ryoshrimp/ddd-toolkit/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/ryoshrimp/ddd-toolkit/compare/v0.1.5...v0.2.0
[0.1.5]: https://github.com/ryoshrimp/ddd-toolkit/compare/v0.1.4...v0.1.5
[0.1.4]: https://github.com/ryoshrimp/ddd-toolkit/compare/v0.1.3...v0.1.4
[0.1.3]: https://github.com/ryoshrimp/ddd-toolkit/compare/v0.1.2...v0.1.3
[0.1.2]: https://github.com/ryoshrimp/ddd-toolkit/compare/v0.1.1...v0.1.2
[0.1.1]: https://github.com/ryoshrimp/ddd-toolkit/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/ryoshrimp/ddd-toolkit/releases/tag/v0.1.0
