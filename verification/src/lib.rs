#![forbid(unsafe_code)]
//! Reusable verification support for Tokio.
//!
//! This crate intentionally stays dependency-free and separate from Tokio's
//! production code. It provides small metaprogramming primitives for abstract
//! state models plus a registry validator that keeps proof obligations tied to
//! their source anchors and harnesses.

pub mod models;
pub mod registry;

/// Define an abstract state model and generate invariant boilerplate.
///
/// The generated `invariant_results`, `is_valid`, and `assert_valid` methods are
/// shared by ordinary tests and formal-verification harnesses so that the same
/// invariant definitions are not duplicated across backends.
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
    };
}

/// Embed a proof-obligation identifier in a harness without runtime cost.
///
/// `tokio-metaverify` also requires the matching `TOKIO_PROOF:` marker so it
/// can validate non-Rust backends and harnesses without parsing Rust syntax.
#[macro_export]
macro_rules! proof_obligation {
    ($id:literal) => {
        const _: &str = $id;
    };
}

/// Generate a Kani proof harness while leaving stable `rustc` builds untouched.
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
