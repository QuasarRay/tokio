//! Abstract bitfield/refcount representation model for Tokio task state.

use crate::{invariant_model, InvariantCheck};
use tokio_verification_macros::{kani_proof, ProofFamily};

const RUNNING: usize = 0b000001;
const COMPLETE: usize = 0b000010;
const NOTIFIED: usize = 0b000100;
const JOIN_INTEREST: usize = 0b001000;
const JOIN_WAKER: usize = 0b010000;
const CANCELLED: usize = 0b100000;
const FLAGS_MASK: usize =
    RUNNING | COMPLETE | NOTIFIED | JOIN_INTEREST | JOIN_WAKER | CANCELLED;
const REF_SHIFT: usize = 6;

invariant_model! {
    #[derive(Clone, Copy, Debug, PartialEq, Eq, ProofFamily)]
    #[proof_family(name = "task-state-bitfield")]
    pub struct TaskStateModel {
        pub lifecycle: u8,
        pub notified: bool,
        pub join_interest: bool,
        pub join_waker: bool,
        pub cancelled: bool,
        pub refs: u8,
    }
    invariants {
        lifecycle_range => |s: &TaskStateModel| s.lifecycle <= 3,
        refcount_range => |s: &TaskStateModel| s.refs <= 31,
        encoded_partition => |s: &TaskStateModel| {
            let raw = s.encode();
            (raw & FLAGS_MASK) < (1usize << REF_SHIFT)
                && (raw >> REF_SHIFT) == s.refs as usize
        },
    }
}

impl TaskStateModel {
    pub fn encode(self) -> usize {
        let mut raw = (self.refs as usize) << REF_SHIFT;
        raw |= (self.lifecycle as usize) & (RUNNING | COMPLETE);
        if self.notified { raw |= NOTIFIED; }
        if self.join_interest { raw |= JOIN_INTEREST; }
        if self.join_waker { raw |= JOIN_WAKER; }
        if self.cancelled { raw |= CANCELLED; }
        raw
    }

    pub fn decode(raw: usize) -> Self {
        Self {
            lifecycle: (raw & (RUNNING | COMPLETE)) as u8,
            notified: raw & NOTIFIED != 0,
            join_interest: raw & JOIN_INTEREST != 0,
            join_waker: raw & JOIN_WAKER != 0,
            cancelled: raw & CANCELLED != 0,
            refs: (raw >> REF_SHIFT) as u8,
        }
    }

    fn apply(&mut self, op: u8) {
        match op % 8 {
            0 => self.lifecycle = 0,
            1 => self.lifecycle = 1,
            2 => self.lifecycle = 2,
            3 => self.lifecycle = 3,
            4 => self.notified = !self.notified,
            5 => self.cancelled = !self.cancelled,
            6 if self.refs < 31 => self.refs += 1,
            7 if self.refs > 0 => self.refs -= 1,
            _ => {}
        }
    }
}

// TOKIO_PROOF: task-state-model-kani
#[kani_proof(id = "task-state-model-kani", unwind = 12)]
pub fn task_state_transitions_preserve_partition() {
    let lifecycle: u8 = kani::any();
    let refs: u8 = kani::any();
    kani::assume(lifecycle <= 3);
    kani::assume(refs <= 31);

    let mut model = TaskStateModel {
        lifecycle,
        notified: kani::any(),
        join_interest: kani::any(),
        join_waker: kani::any(),
        cancelled: kani::any(),
        refs,
    };

    let operations: [u8; 6] = kani::any();
    let mut i = 0;
    while i < operations.len() {
        assert!(model.invariant_ok());
        let encoded = model.encode();
        assert_eq!(TaskStateModel::decode(encoded), model);
        model.apply(operations[i]);
        i += 1;
    }

    assert!(model.invariant_ok());
    assert_eq!(TaskStateModel::decode(model.encode()), model);
}
