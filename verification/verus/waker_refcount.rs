#![allow(unused_imports)]

use verus_builtin::*;
use vstd::prelude::*;
use vstd::{pervasive::*, *};
use verus_state_machines_macros::state_machine;

// TOKIO_PROOF: waker-refcount-model-verus
verus! {

state_machine!(
    WakerRefcountProof {
        fields {
            pub task_ref: nat,
            pub owned_wakers: nat,
        }

        init! {
            initialize() {
                init task_ref = 1;
                init owned_wakers = 0;
            }
        }

        transition! {
            clone_waker() {
                update task_ref = pre.task_ref + 1;
                update owned_wakers = pre.owned_wakers + 1;
            }
        }

        transition! {
            wake_by_ref() {
                update task_ref = pre.task_ref;
                update owned_wakers = pre.owned_wakers;
            }
        }

        transition! {
            consume_owned_waker() {
                require pre.owned_wakers > 0;
                update task_ref = (pre.task_ref - 1) as nat;
                update owned_wakers = (pre.owned_wakers - 1) as nat;
            }
        }

        #[invariant]
        pub fn refcount_conservation(&self) -> bool {
            self.task_ref == self.owned_wakers + 1
        }

        #[inductive(initialize)]
        fn initialize_inductive(post: Self) {}

        #[inductive(clone_waker)]
        fn clone_waker_inductive(pre: Self, post: Self) {}

        #[inductive(wake_by_ref)]
        fn wake_by_ref_inductive(pre: Self, post: Self) {}

        #[inductive(consume_owned_waker)]
        fn consume_owned_waker_inductive(pre: Self, post: Self) {}
    }
);

} // verus!

fn main() {}
