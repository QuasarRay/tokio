#![forbid(unsafe_code)]
//! Reusable verification and metaverification primitives for Tokio.
//!
//! This is a nested workspace on purpose: Tokio's production workspace and
//! production dependency graph remain unchanged.

extern crate self as tokio_verification;

pub mod registry;
pub mod patterns;
pub mod models;
pub mod proof_families;

pub use tokio_verification_macros::{proof_family, ProofState};

/// Metadata implemented by the `ProofState` derive macro.
pub trait ProofState {
    const PROOF_TYPE_NAME: &'static str;
}

/// A common trait for models whose invariant can be delegated through wrappers.
pub trait InvariantCheck {
    fn invariant_holds(&self) -> bool;
}

/// Define an abstract state model and generate invariant boilerplate once.
///
/// Backends added by later stacked PRs reuse these generated methods rather
/// than duplicating invariant definitions.
#[macro_export]
macro_rules! invariant_model {
    (
        $(#[$meta:meta])*
        $vis:vis struct $name:ident {
            $($field_vis:vis $field:ident : $ty:ty),* $(,)?
        }
        invariants {
            $($inv:ident => $predicate:expr),+ $(,)?
        }
    ) => {
        $(#[$meta])*
        $vis struct $name {
            $($field_vis $field: $ty),*
        }

        impl $name {
            pub fn invariant_results(&self) -> ::std::vec::Vec<(&'static str, bool)> {
                ::std::vec![
                    $((::std::stringify!($inv), ($predicate)(self))),+
                ]
            }

            pub fn is_valid(&self) -> bool {
                true $(&& ($predicate)(self))+
            }

            pub fn assert_valid(&self) {
                $(assert!(
                    ($predicate)(self),
                    "verification invariant `{}` failed",
                    ::std::stringify!($inv)
                );)+
            }
        }

        impl $crate::InvariantCheck for $name {
            fn invariant_holds(&self) -> bool {
                self.is_valid()
            }
        }
    };
}

/// Embed a stable proof-obligation identifier in generated proof code.
#[macro_export]
macro_rules! proof_obligation {
    ($id:literal) => {
        const _: &str = $id;
    };
}

#[cfg(test)]
mod tests {
    invariant_model! {
        struct CounterModel {
            value: usize,
            limit: usize,
        }
        invariants {
            bounded => |s: &CounterModel| s.value <= s.limit,
        }
    }

    #[test]
    fn invariant_macro_generates_shared_checker() {
        let valid = CounterModel { value: 2, limit: 3 };
        let invalid = CounterModel { value: 4, limit: 3 };
        assert!(valid.is_valid());
        valid.assert_valid();
        assert!(!invalid.is_valid());
    }
}


/// Delegate an invariant implementation to an inner field.
///
/// This keeps wrapper/refinement models concise without duplicating predicates.
#[macro_export]
macro_rules! delegate_invariant {
    ($outer:ty => $field:ident) => {
        impl $crate::InvariantCheck for $outer {
            fn invariant_holds(&self) -> bool {
                $crate::InvariantCheck::invariant_holds(&self.$field)
            }
        }
    };
}

/// Generate a Kani harness while ordinary Rust builds see no Kani dependency.
#[macro_export]
macro_rules! kani_harness {
    ($id:literal, $name:ident, $body:block) => {
        #[cfg(kani)]
        #[kani::proof]
        fn $name() {
            $crate::proof_obligation!($id);
            $body
        }
    };
}

/// Generate a bounded symbolic operation-sequence proof.
///
/// The model supplies a single `apply(u8)` operation dispatcher and a shared
/// invariant. This turns repeated "arbitrary operations preserve invariant"
/// proofs into one declarative invocation.
#[macro_export]
macro_rules! kani_sequence_proof {
    (
        id = $id:literal,
        harness = $name:ident,
        steps = $steps:literal,
        init = $init:expr
    ) => {
        $crate::kani_harness!($id, $name, {
            let mut model = $init;
            let operations: [u8; $steps] = kani::any();
            let mut index = 0usize;
            while index < operations.len() {
                model.apply(operations[index]);
                assert!($crate::InvariantCheck::invariant_holds(&model));
                index += 1;
            }
        });
    };
}
