# Changelog

All notable changes to this crate are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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

[Unreleased]: https://github.com/ryoshrimp/ddd-toolkit/compare/v0.1.4...HEAD
[0.1.4]: https://github.com/ryoshrimp/ddd-toolkit/compare/v0.1.3...v0.1.4
[0.1.3]: https://github.com/ryoshrimp/ddd-toolkit/compare/v0.1.2...v0.1.3
[0.1.2]: https://github.com/ryoshrimp/ddd-toolkit/compare/v0.1.1...v0.1.2
[0.1.1]: https://github.com/ryoshrimp/ddd-toolkit/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/ryoshrimp/ddd-toolkit/releases/tag/v0.1.0
