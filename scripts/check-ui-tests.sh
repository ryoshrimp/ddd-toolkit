#!/usr/bin/env bash
# Run every ignored trybuild suite, including core's isolated feature variants.
set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/.."
./scripts/check-rust-test-environment.sh

# --tests includes every integration harness, including compile-pass coverage.
# Keep core isolated from macro's serde-enabled dev-dependency.
cargo test -p ddd-toolkit-core --no-default-features --tests --locked -- --ignored
cargo test -p ddd-toolkit-core --features serde --tests --locked -- --ignored
cargo test -p ddd-toolkit-core --features serde_with --tests --locked -- --ignored
cargo test -p ddd-toolkit-macro --tests --locked -- --ignored
