#![allow(unused_imports)]

use verus_builtin::*;
use vstd::prelude::*;
use vstd::{pervasive::*, *};
use verus_state_machines_macros::state_machine;

// TOKIO_PROOF: task-state-model-verus
verus! {

state_machine!(
    TaskStateProof {
        fields {
            pub running: bool,
            pub complete: bool,
            pub notified: bool,
            pub cancelled: bool,
            pub refs: nat,
        }

        init! {
            initialize() {
                init running = false;
                init complete = false;
                init notified = true;
                init cancelled = false;
                init refs = 3;
            }
        }

        transition! {
            notify() {
                require !pre.complete;
                require !pre.notified;
                update notified = true;
                update refs = pre.refs + 1;
            }
        }

        transition! {
            start() {
                require !pre.complete;
                require !pre.running;
                require pre.notified;
                update running = true;
                update notified = false;
            }
        }

        transition! {
            idle() {
                require pre.running;
                require !pre.complete;
                update running = false;
            }
        }

        transition! {
            complete() {
                require pre.running;
                update running = false;
                update complete = true;
            }
        }

        transition! {
            cancel() {
                require !pre.complete;
                update cancelled = true;
            }
        }

        #[invariant]
        pub fn references_remain_live(&self) -> bool {
            self.refs > 0
        }

        #[invariant]
        pub fn lifecycle_is_exclusive(&self) -> bool {
            !(self.running && self.complete)
        }

        #[inductive(initialize)]
        fn initialize_inductive(post: Self) {}

        #[inductive(notify)]
        fn notify_inductive(pre: Self, post: Self) {}

        #[inductive(start)]
        fn start_inductive(pre: Self, post: Self) {}

        #[inductive(idle)]
        fn idle_inductive(pre: Self, post: Self) {}

        #[inductive(complete)]
        fn complete_inductive(pre: Self, post: Self) {}

        #[inductive(cancel)]
        fn cancel_inductive(pre: Self, post: Self) {}
    }
);

} // verus!

fn main() {}
