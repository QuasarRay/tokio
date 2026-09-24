//! One-to-one mapping from diagnosed source patterns to reusable proof families.

use crate::patterns::PATTERNS;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BackendSet {
    Kani,
    Verus,
    Both,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Coverage {
    /// A generated backend proof exists in this stack.
    Generated,
    /// The family is diagnosed and has reusable infrastructure, but code-level
    /// proof work remains. This avoids overstating assurance.
    Infrastructure,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ProofFamily {
    pub pattern_id: &'static str,
    pub backend: BackendSet,
    pub mechanism: &'static str,
    pub coverage: Coverage,
}

macro_rules! declare_proof_families {
    (
        $(
            $id:literal => {
                backend: $backend:ident,
                mechanism: $mechanism:literal,
                coverage: $coverage:ident $(,)?
            }
        ),+ $(,)?
    ) => {
        pub const PROOF_FAMILIES: &[ProofFamily] = &[
            $(
                ProofFamily {
                    pattern_id: $id,
                    backend: BackendSet::$backend,
                    mechanism: $mechanism,
                    coverage: Coverage::$coverage,
                }
            ),+
        ];
    };
}

declare_proof_families! {
    "unsafe-cell-protocol" => {
        backend: Both,
        mechanism: "proof_family attribute + invariant_model",
        coverage: Infrastructure,
    },
    "unsafe-send-sync" => {
        backend: Both,
        mechanism: "ProofState derive + contract attribute",
        coverage: Infrastructure,
    },
    "nonnull-provenance" => {
        backend: Both,
        mechanism: "proof_family attribute + delegated ownership predicate",
        coverage: Infrastructure,
    },
    "pin-address-stability" => {
        backend: Both,
        mechanism: "procedural contract attribute",
        coverage: Infrastructure,
    },
    "manual-drop" => {
        backend: Both,
        mechanism: "ProofState derive + delegated refcount model",
        coverage: Generated,
    },
    "atomic-rmw-loop" => {
        backend: Both,
        mechanism: "kani_sequence_proof + state transition model",
        coverage: Generated,
    },
    "atomic-bitfield-state" => {
        backend: Both,
        mechanism: "ProofState derive + invariant_model",
        coverage: Generated,
    },
    "intrusive-list" => {
        backend: Both,
        mechanism: "invariant_model + kani_sequence_proof + delegation",
        coverage: Generated,
    },
    "raw-waker-vtable" => {
        backend: Both,
        mechanism: "proof_family attribute + delegated refcount model",
        coverage: Generated,
    },
    "repr-c-layout" => {
        backend: Both,
        mechanism: "ProofState derive + const/refinement proof",
        coverage: Infrastructure,
    },
    "unsafe-vtable-functions" => {
        backend: Both,
        mechanism: "proof_family attribute + contract delegation",
        coverage: Infrastructure,
    },
    "ptr-write" => {
        backend: Kani,
        mechanism: "generated initialization-state harness",
        coverage: Infrastructure,
    },
    "maybe-uninit" => {
        backend: Kani,
        mechanism: "ProofState derive + initialization typestate",
        coverage: Infrastructure,
    },
    "unsafe-op-scope" => {
        backend: Both,
        mechanism: "proof_family attribute + metaverifier",
        coverage: Infrastructure,
    }
}

pub fn validate_pattern_coverage() -> Vec<String> {
    let mut problems = Vec::new();

    for pattern in PATTERNS {
        let matches = PROOF_FAMILIES
            .iter()
            .filter(|family| family.pattern_id == pattern.id)
            .count();
        if matches != 1 {
            problems.push(format!(
                "pattern {} maps to {matches} proof families; expected exactly one",
                pattern.id
            ));
        }
    }

    for family in PROOF_FAMILIES {
        if !PATTERNS.iter().any(|pattern| pattern.id == family.pattern_id) {
            problems.push(format!(
                "proof family {} has no diagnosed source pattern",
                family.pattern_id
            ));
        }
        if family.mechanism.trim().is_empty() {
            problems.push(format!(
                "proof family {} has no metaprogramming mechanism",
                family.pattern_id
            ));
        }
    }

    problems
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_diagnosed_pattern_has_exactly_one_family() {
        let problems = validate_pattern_coverage();
        assert!(problems.is_empty(), "{problems:#?}");
    }
}
