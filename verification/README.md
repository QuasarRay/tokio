# Tokio verification infrastructure

This directory is a **nested Cargo workspace** containing reusable verification
and metaverification tooling. It is deliberately outside Tokio's root Cargo
workspace so adding proof tooling cannot change Tokio's production dependency
graph, MSRV, feature resolution, or binaries.

This first stacked change provides only the backend-independent foundation:

- `invariant_model!` generates invariant checking boilerplate once.
- `proof_obligation!` gives generated proofs stable IDs.
- `obligations.txt` is a machine-readable proof registry.
- `tokio-metaverify` rejects duplicate IDs, stale source anchors, missing
  harnesses, duplicate/missing proof markers, and repository-escaping paths.
- the metaverifier regression-tests itself.

Run:

```sh
cargo test --manifest-path verification/Cargo.toml
cargo run --manifest-path verification/Cargo.toml --bin tokio-metaverify -- check .
```

Subsequent stacked PRs add source-pattern regression diagnostics, Kani proof
families, Verus adapters, Lambars-powered metageneration, and binary-identity
enforcement without modifying Tokio's production sources.


## Pattern regression layer

This stacked layer adds a repository-wide proof-pattern scanner:

```sh
cargo run --manifest-path verification/Cargo.toml --bin tokio-proof-patterns -- report .
cargo run --manifest-path verification/Cargo.toml --bin tokio-proof-patterns -- regression .
```

The regression format follows Kani's own `expected` testing principle: stable
diagnostic fragments must remain present, while volatile match counts may grow.
Every diagnosed implementation family is mapped to a reusable metaprogramming
strategy before backend-specific proofs are introduced.


## Generated Kani proof families

This stack adds:

- a procedural macro crate with `#[derive(ProofState)]` and
  `#[proof_family(...)]`
- declarative `kani_harness!` and `kani_sequence_proof!` generators
- invariant delegation via `delegate_invariant!`
- one-to-one mapping from diagnosed implementation patterns to proof-family
  mechanisms
- generated abstract models for intrusive lists, task-state transitions, and
  RawWaker refcount conservation
- Kani expected-output regressions under
  `verification/regression/kani-harnesses.expected`

Run:

```sh
cargo test --manifest-path verification/Cargo.toml
cargo kani --manifest-path verification/Cargo.toml
bash verification/scripts/run-kani-expected.sh
```

The generated model proofs are explicitly registered as abstract-model
assurance; they do not claim equivalence to Tokio's unsafe implementation.


## Lambars metageneration

The Lambars layer lives in its own `verification/lambars-meta` workspace
because Lambars requires Rust 1.92. It consumes the shared pattern/proof-family
registry and performs immutable proof-plan construction, functional
transformation, aggregation, Writer-based diagnostic accumulation, pipeline
composition, and lens-based metadata updates without entering Tokio's runtime
dependency graph.
