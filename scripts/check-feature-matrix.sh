#!/usr/bin/env bash
# Checks that the bare `ddd-toolkit` facade crate compiles under every
# meaningful feature combination. `facade-tests` locks in all features at
# once to exercise every derive/adapter, so it can't catch a combination
# that's individually broken (e.g. `chrono` without `uuid`, or no features
# at all) - this script is what actually covers that.
set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/.."

combos=(
  "--no-default-features"
  ""
  "--no-default-features --features derive"
  "--no-default-features --features chrono"
  "--no-default-features --features uuid"
  "--no-default-features --features derive,zeroize"
  "--no-default-features --features derive,serde"
  "--all-features"
)

for combo in "${combos[@]}"; do
  echo "==> cargo check -p ddd-toolkit ${combo}"
  # shellcheck disable=SC2086
  cargo check -p ddd-toolkit ${combo}
done

echo "All feature combinations checked successfully."
