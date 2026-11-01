#!/usr/bin/env bash
# Check the facade in isolation: facade-tests enables derive + serde together,
# so workspace-wide checks alone cannot detect broken individual combinations.
set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/.."

combos=(
  "--no-default-features"
  ""
  "--no-default-features --features derive"
  "--no-default-features --features serde"
  "--no-default-features --features derive,serde"
  "--all-features"
)

for combo in "${combos[@]}"; do
  echo "==> cargo check -p ddd-toolkit ${combo}"
  # shellcheck disable=SC2086
  cargo check -p ddd-toolkit ${combo}
  echo "==> cargo test -p ddd-toolkit --doc ${combo}"
  # shellcheck disable=SC2086
  cargo test -p ddd-toolkit --doc ${combo}
done

echo "All feature combinations checked successfully."
