# Contributing to ddd-toolkit

Thanks for your interest in contributing!

This monorepo contains three published crates in one Cargo workspace:

- `ddd-toolkit` (root): public facade and feature flags.
- `ddd-toolkit-core` (`crates/core`): domain traits and helpers.
- `ddd-toolkit-macro` (`crates/macro`): derive macros.

Changes to all three crates belong here and can be delivered in one PR.
Workspace dependencies use local paths during development and registry versions
when published. `facade-tests` is a non-published integration suite depending
only on the facade. `scripts/test_macro_consumers.py` creates isolated Cargo
workspaces for facade/core dependencies, renames, self-references, priority, and
missing-dependency diagnostics so feature unification cannot mask regressions.
It runs with the Python tests in CI. Test-local adapters live in
`facade-tests/tests/support`; they are not part of the facade's public API.

## Reporting bugs / requesting features

Please use the issue templates (bug report / feature request) when opening an
issue - they help make sure we get the context needed to act on it quickly.

## Development setup

- Use the latest stable Rust (edition 2024), as in CI. No minimum supported Rust
  version (MSRV) is specified. Functional tests do not require `rust-src`.
  UI diagnostic snapshots additionally require matching `rust-src`
  (`rustup component add rust-src`).
- A plain `git clone` includes core and macro; no sibling repos or submodules
  are needed.

## Running checks locally

Before opening a PR, run the same checks CI runs (see
[`.github/workflows/ci.yml`](.github/workflows/ci.yml)):

```sh
# Functional tests, including doctests; no rust-src or snapshot matching needed.
cargo test --workspace --all-features --locked
# UI diagnostics (compile-fail AND compile-pass); stable + rust-src required.
./scripts/check-ui-tests.sh
cargo build -p ddd-toolkit --examples --all-features --locked
cargo run -p ddd-toolkit --example getting_started --all-features --locked
cargo run -p ddd-toolkit --example dispatch_retry --all-features --locked
cargo run -p ddd-toolkit --example cqrs --all-features --locked
cargo run -p ddd-toolkit --example event_sourcing --all-features --locked
cargo run -p ddd-toolkit --example reliable_delivery --all-features --locked
./scripts/check-feature-matrix.sh
python3 -m unittest discover -s scripts -p 'test_*.py'

RUSTDOCFLAGS="-D warnings" cargo doc --workspace --all-features --no-deps --locked
cargo fmt --all --check
cargo clippy --workspace --all-features --all-targets -- -D warnings
```

The rustdoc check builds documentation for all three published crates with all
features enabled and treats documentation warnings (including broken intra-doc
links) as errors. `--no-deps` excludes dependency documentation. This check is
separate from the doctests run by the workspace tests and feature matrix.

### Test execution paths

- `cargo test --workspace --all-features --locked` runs workspace functional
  tests and doctests. The trybuild harnesses are explicitly `#[ignore]` and print
  the UI command; they are not deleted or weakened. Plain root `cargo test` still
  targets only the facade by default, not the whole workspace.
- `./scripts/check-feature-matrix.sh` checks the facade's six feature combinations
  and doctests, plus core functional tests/doctests without serde and with serde.
  It does not need `rust-src` and does not compare diagnostic snapshots.
- `./scripts/check-ui-tests.sh` runs **all ignored test harnesses** in core
  without serde, core with serde, and macro (including compile-pass cases).
  It first checks the active toolchain and `rust-src`. This preserves the UI
  coverage formerly provided by workspace tests and the feature matrix.
- CI runs `functional-tests` and `ui-diagnostics` independently. The existing
  required `test` check succeeds only if **both** succeed, including when either
  dependency fails or is skipped. Release verification runs both paths and the
  feature matrix before any publishing job can start.

Do not use `--include-ignored` for the functional-only path: it opts back into UI
snapshots. New trybuild harnesses must use the same `#[ignore]` annotation so they
are selected by the UI script. Do not mark functional tests ignored.

## Diagnostic snapshots (core and macro)

All trybuild suites use the latest stable Rust **with matching `rust-src`**,
in CI's dedicated UI job and Release verification. There is no separate
pinned UI toolchain: functional tests and UI snapshots are separate gates
on stable. The last reviewed baseline is rustc 1.99.1 with `rust-src`, on Linux
x86_64; CI logs `rustc -Vv`, `cargo -Vv`, and checks the active sysroot's sources
via `scripts/check-rust-test-environment.sh`. Install sources for the active
stable toolchain with `rustup component add rust-src --toolchain stable`.

When a stable update changes diagnostics without a source change:

1. Reproduce with `rustup update stable`, `rustup component add rust-src --toolchain
   stable`, and `rustup run stable ./scripts/check-ui-tests.sh`.
   Record `rustc +stable -Vv` and `cargo +stable -Vv` in the PR.
2. Inspect the failure and trybuild's actual output. Distinguish an intended
   diagnostic/rendering change from a real contract regression. Prefer a
   targeted `syn::Error` for contracts the derive can inspect syntactically.
   For example, AggregateRoot checks queue path/argument shape itself, but
   rustc still rejects malformed `Vec` declarations; their standard-library
   excerpts must remain in the full snapshots.
3. Only after inspection, regenerate the **affected suite**, for example
   `TRYBUILD=overwrite cargo +stable test -p ddd-toolkit-macro --test compile_fail -- --ignored`
   (use the affected core test target for core snapshots). Never set overwrite
   globally, in CI, or as a normal test invocation.
4. Review `git diff -- '*.stderr'` against the actual errors: preserve expected
   spans, messages and all compiler output, including `$RUST` excerpts. Do not
   delete unstable-looking lines or weaken/disable tests to get a green run.
5. Rerun the full locked workspace tests, feature matrix, and
   `./scripts/check-ui-tests.sh` **without** overwrite.
   Commit the reviewed snapshots, record the new baseline here, and explain
   compiler-only changes separately from intentional contract changes in the PR.

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

## Core/macro compatibility

See [the compatibility contract](docs/macro-compatibility.md) for supported
versions, generated-code API stability and isolated minimum/current-core tests.
Changing a hidden helper used by generated code requires the same compatibility
review as changing a public trait.

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
