#!/usr/bin/env bash
set -euo pipefail

repo_root="$(git rev-parse --show-toplevel)"
head_sha="${HEAD_SHA:-$(git rev-parse HEAD)}"
git fetch -q origin master
base_sha="${BASE_SHA:-$(git merge-base "$head_sha" origin/master)}"
tmp_root="${RUNNER_TEMP:-/tmp}/tokio-binary-identity"
target_dir="$tmp_root/target"

rm -rf "$tmp_root"
mkdir -p "$tmp_root"

restore_head() {
  git checkout -q --detach "$head_sha" || true
}
trap restore_head EXIT

hash_file() {
  if command -v sha256sum >/dev/null 2>&1; then
    sha256sum "$1" | awk '{print $1}'
  else
    shasum -a 256 "$1" | awk '{print $1}'
  fi
}

collect_manifest() {
  local output="$1"
  : > "$output"

  local files=()
  while IFS= read -r file; do
    files+=("$file")
  done < <(
    find "$target_dir/release/deps" -maxdepth 1 -type f \
      \( -name 'libtokio*.rlib' -o -name 'libtokio*.rmeta' \
         -o -name 'libtokio*.so' -o -name 'libtokio*.dylib' \
         -o -name 'tokio*.dll' \) | sort
  )

  if (( ${#files[@]} == 0 )); then
    echo "binary identity: no Tokio artifacts found" >&2
    exit 1
  fi

  for file in "${files[@]}"; do
    printf '%s  %s\n' "$(basename "$file")" "$(hash_file "$file")" >> "$output"
  done
}

build_mode() {
  local ref="$1"
  local label="$2"
  local mode="$3"

  git checkout -q --detach "$ref"
  rm -rf "$target_dir"
  export CARGO_TARGET_DIR="$target_dir"
  export CARGO_INCREMENTAL=0

  case "$mode" in
    minimal)
      cargo build --release -p tokio
      ;;
    full)
      cargo build --release \
        -p tokio -p tokio-macros -p tokio-stream -p tokio-test -p tokio-util \
        --features tokio/full
      ;;
    *)
      echo "unknown identity mode: $mode" >&2
      exit 2
      ;;
  esac

  collect_manifest "$tmp_root/$label-$mode.manifest"
}

echo "binary identity base: $base_sha"
echo "binary identity head: $head_sha"

for mode in minimal full; do
  build_mode "$base_sha" base "$mode"
  build_mode "$head_sha" head "$mode"

  if ! diff -u "$tmp_root/base-$mode.manifest" "$tmp_root/head-$mode.manifest"; then
    echo "binary identity FAILED for mode=$mode" >&2
    exit 1
  fi

  echo "binary identity passed for mode=$mode"
done
