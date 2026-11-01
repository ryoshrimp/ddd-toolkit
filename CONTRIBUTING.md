# Contributing to ddd-toolkit

Thanks for your interest in contributing!

This monorepo contains three published crates in one Cargo workspace:

- `ddd-toolkit` (root): public facade and feature flags.
- `ddd-toolkit-core` (`crates/core`): domain traits and helpers.
- `ddd-toolkit-macro` (`crates/macro`): derive macros.

Changes to all three crates belong here and can be delivered in one PR.
Workspace dependencies use local paths during development and registry versions
when published. `facade-tests` is a non-published consumer integration suite. The consumer test package deliberately depends on
core 0.4 directly: the upstream macros no longer resolve a facade-only
dependency. Test-local adapters live in `facade-tests/tests/support`; they are
not part of the facade's public API.

## Reporting bugs / requesting features

Please use the issue templates (bug report / feature request) when opening an
issue - they help make sure we get the context needed to act on it quickly.

## Development setup

- Rust 1.85+ (Edition 2024). Diagnostic snapshots are verified on current stable
  Rust with `rust-src` installed (`rustup component add rust-src`), as in CI.
- A plain `git clone` includes core and macro; no sibling repos or submodules
  are needed.

## Running checks locally

Before opening a PR, run the same checks CI runs (see
[`.github/workflows/ci.yml`](.github/workflows/ci.yml)):

```sh
cargo test --workspace --all-features
./scripts/check-feature-matrix.sh

cargo fmt --all --check
cargo clippy --workspace --all-features --all-targets -- -D warnings
```

## Releasing

See [docs/releasing.md](docs/releasing.md) for the single-operation release
workflow, version updates, and partial-release recovery. Publishing remains
separate per crate, but does not require three manual release operations.

## Submitting changes

`main` is protected: direct pushes are disabled and all changes go through a
pull request that must pass CI before merging. This project doesn't require a
mandatory review at this stage (it's solo-maintained), but please describe the
change and the reasoning behind it in the PR description - that's what gets
read during merge.

## License

By contributing, you agree that your contributions will be dual-licensed
under the [MIT](LICENSE-MIT) and [Apache-2.0](LICENSE-APACHE) licenses, same
as the rest of this project.
