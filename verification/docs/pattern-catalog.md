# Proof-pattern regression catalog

The scanner diagnoses recurring implementation shapes under `tokio/src`. The
catalog is deliberately about *proof shape*, not algorithms. Each diagnosis is
mapped to a reusable generation mechanism.

| Pattern | Generated mechanism | Intended proof family |
|---|---|---|
| UnsafeCell protocols | attribute + declarative macros | protected access/exclusivity |
| unsafe Send/Sync | derive + attribute macros | thread-safety witnesses |
| NonNull/raw provenance | attribute + delegation | ownership/provenance contracts |
| unchecked Pin | procedural attribute | address-stability contracts |
| ManuallyDrop | derive + delegation | exactly-once destruction |
| atomic RMW loops | declarative state machine | linearizable transition proofs |
| atomic bitfields | derive + declarative macros | legal-state/refcount invariants |
| intrusive lists | derive + declarative + delegation | link/reachability/pinning |
| RawWaker vtables | procedural attribute + delegation | refcount conservation |
| repr(C) layout | derive + const proofs | layout/offset agreement |
| unsafe function vtables | attribute + delegation | contract families |
| ptr::write | attribute + declarative macros | initialization/ownership transfer |
| MaybeUninit | derive + attribute | initialization typestate |
| unsafe-op lint scopes | procedural attribute + metaverifier | named safety-contract coverage |

The regression file uses the same *expected-fragment* principle documented by
Kani's `expected` suite: every stable line in `patterns.expected` must appear
in diagnostics, while counts may increase as Tokio evolves.
