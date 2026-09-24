use tokio_verification::{invariant_model, InvariantCheck, ProofFamily};
use tokio_verification_macros::{DelegateInvariant, ProofFamily};

invariant_model! {
    #[derive(ProofFamily)]
    #[proof_family(name = "sample")]
    struct Sample {
        value: usize,
    }
    invariants {
        small => |s: &Sample| s.value < 4,
    }
}

#[derive(DelegateInvariant)]
struct Wrapper {
    #[invariant_delegate]
    inner: Sample,
    label: &'static str,
}

#[test]
fn derive_and_delegation_macros_share_the_invariant_contract() {
    let good = Wrapper {
        inner: Sample { value: 2 },
        label: Sample::PROOF_FAMILY,
    };
    let bad = Wrapper {
        inner: Sample { value: 9 },
        label: Sample::PROOF_FAMILY,
    };

    assert!(good.invariant_ok());
    assert!(!bad.invariant_ok());
    assert_eq!(good.label, "sample");
}
