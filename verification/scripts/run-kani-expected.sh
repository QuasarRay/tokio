#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
manifest="$repo_root/verification/Cargo.toml"
expected="$repo_root/verification/regression/kani-harnesses.expected"

while IFS='|' read -r harness fragment; do
  [[ -z "$harness" ]] && continue
  [[ "$harness" == \#* ]] && continue

  echo "=== Kani regression: $harness ==="
  output="$(cargo kani --manifest-path "$manifest" --harness "$harness" 2>&1)"
  printf '%s\n' "$output"

  if ! grep -Fq "$fragment" <<<"$output"; then
    echo "missing expected Kani output fragment '$fragment' for $harness" >&2
    exit 1
  fi
done < "$expected"
