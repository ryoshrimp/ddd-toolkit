# Changelog

All notable changes to this crate are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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

[Unreleased]: https://github.com/ryoshrimp/ddd-toolkit/compare/v0.1.1...HEAD
[0.1.1]: https://github.com/ryoshrimp/ddd-toolkit/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/ryoshrimp/ddd-toolkit/releases/tag/v0.1.0
