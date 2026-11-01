#!/usr/bin/env bash
# Check the facade in isolation: facade-tests enables derive + serde together,
# so workspace-wide checks alone cannot detect broken individual combinations.
set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/.."
# Functional checks only; scripts/check-ui-tests.sh covers UI feature variants.

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
  cargo check -p ddd-toolkit --locked ${combo}
  echo "==> cargo test -p ddd-toolkit --doc ${combo}"
  # shellcheck disable=SC2086
  cargo test -p ddd-toolkit --doc --locked ${combo}
done

# Core must also work without serde, independently of macro's dev-dependencies.
cargo test -p ddd-toolkit-core --no-default-features --locked
cargo test -p ddd-toolkit-core --features serde --locked

echo "All feature combinations checked successfully."
