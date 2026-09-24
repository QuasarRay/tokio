#!/usr/bin/env bash
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
manifest="$root/verification/Cargo.toml"

run_expected() {
  local harness="$1"
  local expected="$2"
  local output
  output="$(mktemp)"

  if ! cargo kani --manifest-path "$manifest" --harness "$harness" 2>&1 | tee "$output"; then
    rm -f "$output"
    exit 1
  fi

  while IFS= read -r fragment; do
    [[ -z "$fragment" || "$fragment" == #* ]] && continue
    if ! grep -F -- "$fragment" "$output" >/dev/null; then
      echo "missing expected Kani regression fragment: $fragment" >&2
      rm -f "$output"
      exit 1
    fi
  done < "$expected"

  rm -f "$output"
}

run_expected   tokio_verification::models::intrusive_list::intrusive_list_operations_preserve_invariants   "$root/verification/regression/kani/intrusive-list.expected"

run_expected   tokio_verification::models::task_state::task_state_transitions_preserve_partition   "$root/verification/regression/kani/task-state.expected"
