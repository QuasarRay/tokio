#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
backend="${1:-}"
shift || true

case "$backend" in
  meta)
    cargo run --manifest-path "$repo_root/verification/Cargo.toml" --bin tokio-metaverify -- check "$repo_root"
    ;;
  test)
    cargo test --manifest-path "$repo_root/verification/Cargo.toml" "$@"
    ;;
  kani)
    (cd "$repo_root" && cargo kani -p tokio-verification "$@")
    ;;
  verus)
    verus_bin="${VERUS_BIN:-verus}"
    "$verus_bin" "$repo_root/verification/verus/smoke.rs" "$@"
    ;;
  loom)
    export RUSTFLAGS="${RUSTFLAGS:--Dwarnings --cfg loom --cfg tokio_unstable -C debug_assertions}"
    export LOOM_MAX_PREEMPTIONS="${LOOM_MAX_PREEMPTIONS:-2}"
    export LOOM_MAX_BRANCHES="${LOOM_MAX_BRANCHES:-10000}"
    (cd "$repo_root/tokio" && cargo test --lib --release --features full -- "$@")
    ;;
  *)
    echo "usage: $0 {meta|test|kani|verus|loom} [backend args...]" >&2
    exit 2
    ;;
esac
