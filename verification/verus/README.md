# Verus backend

This directory is intentionally outside both Tokio's production workspace and
the nested Cargo verification workspace. Verus has its own toolchain and can
therefore evolve independently without changing Tokio's MSRV or dependency
resolution.

The backend uses Verus's own state-machine metaprogramming to encode recurring
Tokio proof shapes rather than duplicating transition/inductiveness boilerplate.

Current abstract-model obligations:

- task lifecycle/refcount transition invariants
- RawWaker refcount conservation

These proofs are source-anchored in the shared obligation registry. They do not
claim code-level refinement to Tokio's unsafe implementation.

Run with an installed Verus binary:

    VERUS_BIN=/path/to/verus bash verification/scripts/run-verus.sh
