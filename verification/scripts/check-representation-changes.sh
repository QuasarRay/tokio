#!/usr/bin/env bash
set -euo pipefail

repo_root="$(git rev-parse --show-toplevel)"
head_sha="${HEAD_SHA:-$(git rev-parse HEAD)}"
git fetch -q origin master
base_sha="${BASE_SHA:-$(git merge-base "$head_sha" origin/master)}"
map_file="$repo_root/verification/representation-map.txt"

mapfile -t changed < <(
  git diff --name-only "$base_sha..$head_sha" -- |
    grep -E '^(tokio|tokio-macros|tokio-stream|tokio-test|tokio-util)/src/.*\.rs$' || true
)

if (( ${#changed[@]} == 0 )); then
  echo "representation guard: no production Rust source changes"
  exit 0
fi

declare -A harnesses=()
for path in "${changed[@]}"; do
  mapping="$(awk -F'|' -v path="$path" '$1 == path { print; exit }' "$map_file")"
  if [[ -z "$mapping" ]]; then
    echo "representation guard: $path changed without a Kani regression mapping" >&2
    exit 1
  fi

  IFS='|' read -r mapped_path harness reason <<<"$mapping"
  if [[ -z "$harness" || -z "$reason" ]]; then
    echo "representation guard: incomplete mapping for $path" >&2
    exit 1
  fi
  harnesses["$harness"]=1
  echo "representation guard: $mapped_path -> $harness ($reason)"
done

for harness in "${!harnesses[@]}"; do
  echo "=== representation Kani regression: $harness ==="
  output="$(cargo kani --manifest-path "$repo_root/verification/Cargo.toml" --harness "$harness" 2>&1)"
  printf '%s\n' "$output"
  grep -Fq "SUCCESS" <<<"$output" || {
    echo "representation guard: Kani harness $harness did not report SUCCESS" >&2
    exit 1
  }
done
