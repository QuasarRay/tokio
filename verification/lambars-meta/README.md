# Lambars metageneration workspace

This is a deliberately separate Rust 1.92 / edition-2024 workspace because the
pinned Lambars revision requires Rust 1.92, while Tokio itself supports Rust
1.71.

Lambars is used here as a metaverification and proof-plan layer, not as a Tokio
runtime dependency.

The tool uses:

- PersistentVector for immutable proof-plan snapshots and structural sharing
- FunctorMut to transform diagnosed source patterns into proof-plan entries
- Foldable to aggregate generated-proof coverage
- Writer to accumulate metaverification diagnostics
- pipe! for deterministic transformation pipelines
- Lambars derive-generated Lenses to normalize/update metadata immutably

It consumes the backend-independent Tokio verification registry and emits a
deterministic plan.

Commands:

    cargo +1.92.0 run --manifest-path verification/lambars-meta/Cargo.toml -- check
    cargo +1.92.0 run --manifest-path verification/lambars-meta/Cargo.toml -- render

The Lambars git dependency is pinned to commit
7cd362e3995eb6e33f07101b604fd006e9551a8b.
