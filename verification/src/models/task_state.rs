//! Abstract model for Tokio's recurring atomic bitfield/RMW transition shape.
//!
//! This captures state-machine invariants, not memory-ordering refinement.

use crate::{invariant_model, kani_sequence_proof, proof_family, ProofState};

const MAX_REFS: u8 = 8;

invariant_model! {
    #[derive(Clone, Debug, PartialEq, Eq, ProofState)]
    pub struct TaskStateModel {
        running: bool,
        complete: bool,
        notified: bool,
        cancelled: bool,
        refs: u8,
    }
    invariants {
        refs_nonzero => |s: &TaskStateModel| s.refs > 0 && s.refs <= MAX_REFS,
        lifecycle => |s: &TaskStateModel| !(s.running && s.complete),
        complete_not_running => |s: &TaskStateModel| !s.complete || !s.running,
    }
}

impl TaskStateModel {
    pub const fn new() -> Self {
        Self {
            running: false,
            complete: false,
            notified: true,
            cancelled: false,
            refs: 3,
        }
    }

    #[proof_family(
        id = "task-state-rmw-family",
        backend = "kani",
        assurance = "abstract-model"
    )]
    pub fn apply(&mut self, operation: u8) {
        match operation % 6 {
            0 => self.notify(),
            1 => self.start(),
            2 => self.idle(),
            3 => self.complete(),
            4 => self.cancel(),
            _ => self.drop_reference(),
        }
    }

    fn notify(&mut self) {
        if !self.complete && !self.notified && self.refs < MAX_REFS {
            self.notified = true;
            self.refs += 1;
        }
    }

    fn start(&mut self) {
        if !self.complete && !self.running && self.notified {
            self.running = true;
            self.notified = false;
        }
    }

    fn idle(&mut self) {
        if self.running && !self.complete {
            self.running = false;
            if !self.notified && self.refs > 1 {
                self.refs -= 1;
            }
        }
    }

    fn complete(&mut self) {
        if self.running {
            self.running = false;
            self.complete = true;
        }
    }

    fn cancel(&mut self) {
        if !self.complete {
            self.cancelled = true;
            self.notify();
        }
    }

    fn drop_reference(&mut self) {
        if self.refs > 1 {
            self.refs -= 1;
        }
    }
}

impl Default for TaskStateModel {
    fn default() -> Self {
        Self::new()
    }
}

// TOKIO_PROOF: task-state-model-kani
kani_sequence_proof!(
    id = "task-state-model-kani",
    harness = task_state_sequence_preserves_invariant,
    steps = 8,
    init = TaskStateModel::new()
);
