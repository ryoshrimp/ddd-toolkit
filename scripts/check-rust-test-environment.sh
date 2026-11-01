#!/usr/bin/env bash
# UI-only preflight (called by check-ui-tests.sh), not needed for functional tests.
# Diagnostics depend on both rustc and rust-src.
set -euo pipefail
rustc --version --verbose
cargo --version --verbose
sysroot="$(rustc --print sysroot)"
if [[ ! -f "$sysroot/lib/rustlib/src/rust/library/alloc/src/vec/mod.rs" ]]; then
  echo 'error: diagnostic snapshots require rust-src for the active toolchain; run rustup component add rust-src' >&2
  exit 1
fi
echo 'rust-src: present for the active rustc sysroot'
