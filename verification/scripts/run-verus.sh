#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
verus_bin="${VERUS_BIN:-verus}"

for proof in   "$repo_root/verification/verus/task_state.rs"   "$repo_root/verification/verus/waker_refcount.rs"
do
  echo "=== Verus: ${proof#$repo_root/} ==="
  "$verus_bin" "$proof"
done
