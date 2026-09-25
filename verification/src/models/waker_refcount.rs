//! Abstract refcount conservation model for RawWaker clone/wake/drop families.

use crate::{invariant_model, kani_sequence_proof, proof_family, ProofState};

const MAX_WAKERS: u8 = 7;

invariant_model! {
    #[derive(Clone, Debug, PartialEq, Eq, ProofState)]
    pub struct WakerRefcountModel {
        task_ref: u8,
        owned_wakers: u8,
    }
    invariants {
        task_lives => |s: &WakerRefcountModel| s.task_ref >= 1,
        conservation => |s: &WakerRefcountModel| s.task_ref == 1 + s.owned_wakers,
        bounded => |s: &WakerRefcountModel| s.owned_wakers <= MAX_WAKERS,
    }
}

impl WakerRefcountModel {
    pub const fn new() -> Self {
        Self {
            task_ref: 1,
            owned_wakers: 0,
        }
    }

    #[proof_family(
        id = "raw-waker-refcount-family",
        backend = "kani",
        assurance = "abstract-model"
    )]
    pub fn apply(&mut self, operation: u8) {
        match operation % 4 {
            0 => self.clone_waker(),
            1 => self.wake_by_ref(),
            2 => self.wake_by_val(),
            _ => self.drop_waker(),
        }
    }

    fn clone_waker(&mut self) {
        if self.owned_wakers < MAX_WAKERS {
            self.owned_wakers += 1;
            self.task_ref += 1;
        }
    }

    fn wake_by_ref(&mut self) {}

    fn wake_by_val(&mut self) {
        self.consume_owned_waker();
    }

    fn drop_waker(&mut self) {
        self.consume_owned_waker();
    }

    fn consume_owned_waker(&mut self) {
        if self.owned_wakers > 0 {
            self.owned_wakers -= 1;
            self.task_ref -= 1;
        }
    }
}

impl Default for WakerRefcountModel {
    fn default() -> Self {
        Self::new()
    }
}

// TOKIO_PROOF: waker-refcount-model-kani
kani_sequence_proof!(
    id = "waker-refcount-model-kani",
    harness = waker_refcount_sequence_preserves_invariant,
    steps = 8,
    init = WakerRefcountModel::new()
);
