# Contributing to ddd-toolkit

Thanks for your interest in contributing!

This repo is the facade over
[`ddd-toolkit-core`](https://github.com/ryoshrimp/ddd-toolkit-core) and
[`ddd-toolkit-macro`](https://github.com/ryoshrimp/ddd-toolkit-macro), pulled
in here as regular crates.io dependencies (see `Cargo.toml`). Changes to core
domain/port logic or to the derive macros belong in those repos, not here -
open an issue/PR there instead if that's what you're touching. This repo's
own contributions are about the facade re-exports, the `facade-tests`
integration suite, and docs.

## Reporting bugs / requesting features

Please use the issue templates (bug report / feature request) when opening an
issue - they help make sure we get the context needed to act on it quickly.

## Development setup

- Rust 1.85+ (Edition 2024)
- A plain `git clone` is enough - `ddd-toolkit-core` / `ddd-toolkit-macro` are
  pulled in as ordinary crates.io dependencies, not submodules.

## Running checks locally

Before opening a PR, run the same checks CI runs (see
[`.github/workflows/ci.yml`](.github/workflows/ci.yml)):

```sh
cargo test --workspace --all-features
./scripts/check-feature-matrix.sh

cargo fmt --check
cargo clippy --workspace --all-features --all-targets -- -D warnings
```

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
