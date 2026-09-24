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
