# Tokio verification infrastructure

This directory is a reusable verification layer for Tokio. It is intentionally
separate from production code so formal tooling can evolve without changing
Tokio's runtime dependencies or MSRV.

## Assurance levels

The registry records an explicit assurance level for every obligation. A proof
of an `abstract-model` is **not** a proof of the corresponding unsafe Rust. It
establishes properties of the model and provides a reusable target for later
refinement/equivalence proofs. `meta` obligations test the verification
infrastructure itself; `infrastructure` obligations only validate a backend.

## Components

- `invariant_model!` generates invariant checking boilerplate once for reuse by
  tests and formal backends.
- `kani_harness!` generates Kani harnesses without affecting ordinary Rust
  builds.
- `obligations.txt` is the machine-readable proof registry.
- `tokio-metaverify` rejects duplicate IDs, missing files, stale source anchors,
  missing/duplicated proof markers, and paths that escape the repository.
- `scripts/run-backend.sh` provides one stable entry point for Kani, Verus,
  Loom, ordinary tests, and metaverification.
- `models/intrusive_list.rs` is the first bounded abstract model, targeting the
  unsafe intrusive-list state machine used by Tokio internals.

## Commands

```sh
bash verification/scripts/run-backend.sh meta
bash verification/scripts/run-backend.sh test
bash verification/scripts/run-backend.sh kani
VERUS_BIN=/path/to/verus bash verification/scripts/run-backend.sh verus
bash verification/scripts/run-backend.sh loom sync::tests
```

To print the registry as a Markdown matrix:

```sh
cargo run -p tokio-verification --bin tokio-metaverify -- matrix .
```

## Adding a proof

1. Add or reuse an abstract model/specification.
2. Put a `// TOKIO_PROOF: <id>` marker in its harness.
3. Add exactly one matching row to `obligations.txt` with source file, stable
   source anchor, backend, assurance level, harness, and a narrowly worded claim.
4. Run `meta` and the relevant backend.
5. If a model is claimed to prove production code, add a separate refinement or
   equivalence obligation rather than silently upgrading the assurance level.

This structure is designed to keep proof claims auditable as Tokio changes and
to prevent model proofs from being mistaken for code-level verification.
