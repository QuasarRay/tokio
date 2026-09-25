# Representation-only change gate

The verification stack treats production representation changes differently
from proof-only changes.

For every modified Rust file under the published Tokio crates:

1. the file must be mapped to a Kani regression harness in
   `verification/representation-map.txt`;
2. that harness must report Kani `SUCCESS`;
3. the ordinary Kani expected-output regression suite must still pass;
4. the original Tokio merge-base and the verification head are built from the
   same checkout path, with the same toolchain and release flags;
5. the resulting Tokio release artifacts are SHA-256 compared byte-for-byte.

The current verification stack changes no production Rust source, so the
representation map is intentionally empty.

## Build matrix

The identity script compares two configurations:

- minimal `tokio` release build;
- full release build of the published Tokio crates with `tokio/full`.

The GitHub workflow runs the comparison on Linux, macOS, and Windows.

A byte-identical result is stronger than a behavioral test for those exact
toolchain/platform/feature combinations. It does not by itself prove semantic
equivalence for arbitrary compilers or targets, which is why production source
changes additionally require a mapped Kani regression proof.
