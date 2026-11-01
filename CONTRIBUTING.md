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

- Use the latest stable Rust (edition 2024), as in CI. No minimum supported Rust
  version (MSRV) is specified. Diagnostic snapshots are verified with `rust-src`
  installed (`rustup component add rust-src`).
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

## Documentation

Keep each rustdoc block in language order: all English prose, then all Japanese
prose. Put `**English:**<br>` and `**日本語:**<br>` on their own doc-comment
lines, with the body starting on the next line. Use `<br>` rather than a blank
line after the label: rustdoc uses the first paragraph as the API-list summary,
so a label-only paragraph would hide the summary text. Do not alternate languages
paragraph by paragraph. Put shared code examples after both language sections,
under `# Example / 使用例`, without duplicating the code.

Let re-exported items inherit their original API documentation. Adding bilingual
docs to both a `pub use` and its target makes rustdoc concatenate the language
sections and repeat them on the rendered API page.

Each published crate has `README.md` (English) and `README.ja.md` (Japanese),
with links between them. Keep API descriptions, feature requirements, limitations,
and code examples in sync when updating either version.

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
